use crate::domain::DomainResult;
use reqwest::Client;
use std::error::Error;
use std::io::{self, BufRead};

pub async fn prompt_domains(query: &str, token: &str) -> Result<Vec<DomainResult>, Box<dyn Error>> {
    let client = Client::new();

    // Create the request body with the query parameter
    let body = serde_json::json!({
        "q": query.to_string(),
        "mode": "tld",
        "tlds": ["com"]
    });

    println!("Sending request to API...");

    // Make the POST request to the API with the token from config
    let response = client
        .post("https://api.namedrop.dev/domains/prompt")
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .await?;

    // Check if the request was successful
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    // Get the response body as bytes
    let bytes = response.bytes().await?;

    // Convert bytes to a reader for line-by-line processing
    let reader = io::BufReader::new(bytes.as_ref());
    let mut results = Vec::new();

    // Process each line as NDJSON
    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        // Parse the JSON line
        match serde_json::from_str::<serde_json::Value>(&line) {
            Ok(json) => {
                // Extract domain information from the JSON
                if let Some(domain) = json.get("domain").and_then(|d| d.as_str()) {
                    let available = json
                        .get("available")
                        .and_then(|a| a.as_bool())
                        .unwrap_or(false);
                    let premium = json
                        .get("premium")
                        .and_then(|p| p.as_bool())
                        .unwrap_or(false);

                    results.push(DomainResult::new_with_premium(
                        domain.to_string(),
                        available,
                        premium,
                    ));
                } else {
                    println!("Ignoring unexpected JSON: {}", json);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Error parsing JSON line: {}", e);
                continue;
            }
        }
    }

    println!(
        "Finished processing. Total domains received: {}",
        results.len()
    );

    Ok(results)
}
