use crate::VERSION;
use crate::config;
use crate::domain::DomainResult;
use futures_core::stream::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use std::env::consts::{ARCH, OS};
use std::error::Error;
use std::pin::Pin;
use tokio::sync::mpsc;

pub async fn stream_domains(
    query: &str,
    mode: &str,
    token: &str,
) -> Result<Pin<Box<dyn Stream<Item = DomainResult> + Send>>, Box<dyn Error>> {
    // Create a channel for sending domains as they're processed
    let (tx, rx) = mpsc::channel(32);

    // Clone values for the spawned task
    let query = query.to_string();
    let mode = mode.to_string();
    let token = token.to_string();

    // Spawn a task to process the API response
    tokio::spawn(async move {
        let client = Client::new();

        // Load config to get the API server URL
        if let Ok(config) = config::Config::load() {
            let api_server = config.get_api_server();

            // Create the endpoint URL
            let endpoint = format!("{}/domains/stream", api_server);

            // Create the request body with the query parameter
            let body = serde_json::json!({
                "q": query,
                "mode": mode,
                "tlds": "com,dev,app",
            });

            let user_agent = format!("NamekitCLI/{} ({}/{})", VERSION, OS, ARCH);

            // Make the POST request to the API with the token from config
            match client
                .post(&endpoint)
                .header("User-Agent", user_agent)
                .header("Authorization", format!("Bearer {}", token))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    // Check if the request was successful
                    if !response.status().is_success() {
                        // Handle rate limiting (429 Too Many Requests) specifically
                        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            eprintln!("Rate limit reached");
                            eprintln!(
                                "Visit https://namekit.app to upgrade your plan for unlimited searches"
                            );
                        } else if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                            eprintln!("Authentication failed");
                            eprintln!(
                                "Your token may have expired. Try logging in again with 'namekit auth google' or 'namekit auth github'"
                            );
                        } else {
                            eprintln!("API request failed: {}", response.status());
                        }
                        return;
                    }

                    // Process the streaming response
                    let mut buffer = String::new();
                    let mut stream = response.bytes_stream();

                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                // Add the new chunk to our buffer
                                buffer.push_str(&String::from_utf8_lossy(&chunk));

                                // Process complete lines by splitting on newlines
                                let parts: Vec<&str> = buffer.split('\n').collect();

                                // If we have multiple parts, process all but the last one
                                if parts.len() > 1 {
                                    // Process all complete lines (all but the last part)
                                    for line in &parts[0..parts.len() - 1] {
                                        if line.trim().is_empty() {
                                            continue;
                                        }

                                        // Parse the JSON line
                                        match serde_json::from_str::<serde_json::Value>(line) {
                                            Ok(json) => {
                                                // Extract domain information from the JSON
                                                if let Some(domain) =
                                                    json.get("domain").and_then(|d| d.as_str())
                                                {
                                                    let available = json
                                                        .get("available")
                                                        .and_then(|a| a.as_bool())
                                                        .unwrap_or(false);
                                                    let premium = json
                                                        .get("premium")
                                                        .and_then(|p| p.as_bool())
                                                        .unwrap_or(false);

                                                    // Send the domain through the channel
                                                    let domain_result =
                                                        DomainResult::new_with_premium(
                                                            domain.to_string(),
                                                            available,
                                                            premium,
                                                        );

                                                    if tx.send(domain_result).await.is_err() {
                                                        // Channel closed, receiver dropped
                                                        return;
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                // Silently ignore malformed JSON lines
                                            }
                                        }
                                    }

                                    // Keep only the last part in the buffer
                                    buffer = parts[parts.len() - 1].to_string();
                                }
                            }
                            Err(_) => {
                                // Connection error, stop streaming
                                break;
                            }
                        }
                    }

                    // Process any remaining data in the buffer
                    if !buffer.trim().is_empty() {
                        match serde_json::from_str::<serde_json::Value>(&buffer) {
                            Ok(json) => {
                                if let Some(domain) = json.get("domain").and_then(|d| d.as_str()) {
                                    let available = json
                                        .get("available")
                                        .and_then(|a| a.as_bool())
                                        .unwrap_or(false);
                                    let premium = json
                                        .get("premium")
                                        .and_then(|p| p.as_bool())
                                        .unwrap_or(false);

                                    let domain_result = DomainResult::new_with_premium(
                                        domain.to_string(),
                                        available,
                                        premium,
                                    );

                                    let _ = tx.send(domain_result).await;
                                }
                            }
                            Err(_) => {
                                // Silently ignore malformed final JSON
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Network error: {}", e);
                }
            }
        } else {
            eprintln!("Configuration error");
        }

        // Channel will be closed when tx is dropped at the end of this function
    });

    // Convert the receiver into a stream
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    Ok(Box::pin(stream))
}
