use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use url::Url;

const AUTH_SERVER: &str = "https://auth.mgcmnd.net";
const CALLBACK_HOST: &str = "127.0.0.1";
const CALLBACK_PORT: u16 = 8080;

#[derive(Debug)]
pub enum AuthError {
    IoError(io::Error),
    HttpError(reqwest::Error),
    ServerError(String),
    CallbackError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::IoError(e) => write!(f, "IO error: {}", e),
            AuthError::HttpError(e) => write!(f, "HTTP error: {}", e),
            AuthError::ServerError(msg) => write!(f, "Server error: {}", msg),
            AuthError::CallbackError(msg) => write!(f, "Callback error: {}", msg),
        }
    }
}

impl From<io::Error> for AuthError {
    fn from(err: io::Error) -> Self {
        AuthError::IoError(err)
    }
}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> Self {
        AuthError::HttpError(err)
    }
}

impl Error for AuthError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub email: String,
    pub tier: String,
}

pub enum AuthProvider {
    Google,
    GitHub,
}

impl AuthProvider {
    fn endpoint(&self) -> &'static str {
        match self {
            AuthProvider::Google => "/google/login",
            AuthProvider::GitHub => "/github/login",
        }
    }

    fn name(&self) -> &'static str {
        match self {
            AuthProvider::Google => "Google",
            AuthProvider::GitHub => "GitHub",
        }
    }
}

pub async fn login(provider: AuthProvider) -> Result<AuthToken, AuthError> {
    let callback_url = format!("http://{}:{}/callback", CALLBACK_HOST, CALLBACK_PORT);

    // Start local HTTP server for OAuth callback
    let listener = TcpListener::bind(format!("{}:{}", CALLBACK_HOST, CALLBACK_PORT)).await?;
    println!("Started local callback server on {}", callback_url);

    // Build the OAuth initiation URL
    let auth_url = format!(
        "{}{}?redirect_uri={}",
        AUTH_SERVER,
        provider.endpoint(),
        urlencoding::encode(&callback_url)
    );

    println!("Opening {} OAuth flow...", provider.name());
    println!("Auth URL: {}", auth_url);
    println!("Callback URL: {}", callback_url);

    // Try to open the URL in the default browser
    if let Err(_) = open::that(&auth_url) {
        println!("Could not automatically open browser. Please manually visit the URL above.");
    }

    // Wait for the OAuth callback
    let (mut stream, _) = listener.accept().await?;

    let buf_reader = BufReader::new(&mut stream);
    let mut lines = buf_reader.lines();

    // Read the first line (HTTP request line)
    let request_line = lines
        .next_line()
        .await?
        .ok_or_else(|| AuthError::CallbackError("No request line received".to_string()))?;

    // Parse the request line to extract the callback parameters
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(AuthError::CallbackError("Invalid HTTP request".to_string()));
    }

    let path_and_query = parts[1];
    println!("Received callback: {}", path_and_query);

    let url = Url::parse(&format!("http://localhost{}", path_and_query))
        .map_err(|e| AuthError::CallbackError(format!("Invalid callback URL: {}", e)))?;

    println!("Parsed URL query pairs:");
    for (key, value) in url.query_pairs() {
        println!("  {} = {}", key, value);
    }

    // Check if we received a direct token (simplified flow) or authorization code
    if let Some(token) = url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string())
    {
        // Direct token flow - no need to exchange code
        let response = "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>Authentication successful!</h1><p>You can close this tab and return to the terminal.</p></body></html>";
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        return Ok(AuthToken {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: None,
        });
    }

    // Standard OAuth flow with authorization code
    let response = if url.query_pairs().any(|(key, _)| key == "code") {
        "HTTP/1.1 200 OK\r\n\r\n<html><body><h1>Authentication successful!</h1><p>You can close this tab and return to the terminal.</p></body></html>"
    } else {
        "HTTP/1.1 400 Bad Request\r\n\r\n<html><body><h1>Authentication failed!</h1><p>Please try again.</p></body></html>"
    };

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    // Extract authorization code from callback
    let auth_code = url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .ok_or_else(|| {
            // Check for error parameter
            let error = url
                .query_pairs()
                .find(|(key, _)| key == "error")
                .map(|(_, value)| value.to_string())
                .unwrap_or_else(|| "Unknown error".to_string());
            AuthError::CallbackError(format!("OAuth failed: {}", error))
        })?;

    // Exchange authorization code for access token
    exchange_code_for_token(&auth_code, &callback_url, &provider).await
}

async fn exchange_code_for_token(
    code: &str,
    redirect_uri: &str,
    provider: &AuthProvider,
) -> Result<AuthToken, AuthError> {
    let client = Client::new();
    let token_url = format!("{}/token", AUTH_SERVER);

    let provider_name = provider.name().to_lowercase();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("provider", provider_name.as_str()),
    ];

    println!("Exchanging code for token...");
    println!("Token URL: {}", token_url);
    println!("Request params: {:?}", params);

    let response = client.post(&token_url).form(&params).send().await?;

    println!("Token exchange response status: {}", response.status());

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        println!("Token exchange error response: {}", error_text);
        return Err(AuthError::ServerError(format!(
            "Token exchange failed: {}",
            error_text
        )));
    }

    let response_text = response.text().await?;
    println!("Token exchange response body: {}", response_text);

    let token: AuthToken = serde_json::from_str(&response_text)
        .map_err(|e| AuthError::ServerError(format!("Failed to parse token response: {}", e)))?;

    Ok(token)
}

pub async fn get_user_info(token: &str) -> Result<UserInfo, AuthError> {
    use crate::config;

    let client = Client::new();

    // Load config to get the API server URL
    let config = config::Config::load().map_err(|e| {
        AuthError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    let api_server = config.get_api_server();
    let info_url = format!("{}/auth/info", api_server);

    println!("Fetching user info from: {}", info_url);

    let response = client
        .get(&info_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    println!("User info response status: {}", response.status());

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        println!("User info response text: {}", response.text().await?);

        return Err(AuthError::ServerError(
            "Authentication failed - token may be expired or invalid".to_string(),
        ));
    }

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        println!("User info error response: {}", error_text);
        return Err(AuthError::ServerError(format!(
            "Failed to get user info: {}",
            error_text
        )));
    }

    let response_text = response.text().await?;
    println!("User info response body: {}", response_text);

    let user_info: UserInfo = serde_json::from_str(&response_text).map_err(|e| {
        AuthError::ServerError(format!("Failed to parse user info response: {}", e))
    })?;

    Ok(user_info)
}
