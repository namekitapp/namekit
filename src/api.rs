use crate::config;
use crate::domain::DomainResult;
use futures_core::stream::Stream;
use futures_util::StreamExt;
use reqwest::Client;
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
            let endpoint = format!("{}/domains/prompt", api_server);

            // Create the request body with the query parameter
            let body = serde_json::json!({
                "q": query,
                "mode": mode,
                "tlds": "com,dev,app",
            });

            // Make the POST request to the API with the token from config
            match client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {}", token))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    // Check if the request was successful
                    if !response.status().is_success() {
                        eprintln!("API request failed with status: {}", response.status());
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
                                                } else {
                                                    println!("Ignoring unexpected JSON: {}", json);
                                                }
                                            }
                                            Err(e) => {
                                                eprintln!("Error parsing JSON line: {}", e);
                                            }
                                        }
                                    }

                                    // Keep only the last part in the buffer
                                    buffer = parts[parts.len() - 1].to_string();
                                }
                            }
                            Err(e) => {
                                eprintln!("Error reading chunk: {}", e);
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
                            Err(e) => {
                                eprintln!("Error parsing final JSON: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error sending request: {}", e);
                }
            }
        } else {
            eprintln!("Error loading config");
        }

        // Channel will be closed when tx is dropped at the end of this function
    });

    // Convert the receiver into a stream
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    Ok(Box::pin(stream))
}
