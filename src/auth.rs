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

    // Build the OAuth initiation URL
    let auth_url = format!(
        "{}{}?redirect_uri={}",
        AUTH_SERVER,
        provider.endpoint(),
        urlencoding::encode(&callback_url)
    );

    println!(
        "Opening {} authentication in your browser...",
        provider.name()
    );

    // Try to open the URL in the default browser
    if open::that(&auth_url).is_err() {
        println!("Could not open browser automatically.");
        println!("Please visit: {}", auth_url);
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

    let url = Url::parse(&format!("http://localhost{}", path_and_query))
        .map_err(|e| AuthError::CallbackError(format!("Invalid callback URL: {}", e)))?;

    // Check if we received a direct token (auth server uses this flow)
    if let Some(token) = url
        .query_pairs()
        .find(|(key, _)| key == "token")
        .map(|(_, value)| value.to_string())
    {
        // Direct token flow - auth server returns JWT directly
        let response = r#"HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8

<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Namekit - Authentication Successful</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="icon" href="https://namekit.app/images/icon512.png">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; text-align: center; margin: 0; padding: 40px 20px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; min-height: 90vh; display: flex; flex-direction: column; justify-content: center; }
        .container { max-width: 500px; margin: 0 auto; }
        .logo { width: 80px; height: 80px; margin: 0 auto 20px; border-radius: 16px; }
        h1 { font-size: 2.5em; margin-bottom: 10px; }
        p { font-size: 1.2em; opacity: 0.9; margin-bottom: 30px; }
        .success-icon { font-size: 4em; margin-bottom: 20px; color: #4ade80; }
    </style>
</head>
<body>
    <div class="container">
        <img src="https://namekit.app/images/icon512.png" alt="Namekit" class="logo">
        <h1>Authentication Successful!</h1>
        <p>You've successfully authenticated with Namekit.</p>
        <p>You can now close this window and return to the terminal.</p>
    </div>
</body>
</html>"#;
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;

        return Ok(AuthToken {
            access_token: token,
            token_type: "Bearer".to_string(),
            expires_in: None,
        });
    }

    // No token received - this is an error
    let error = url
        .query_pairs()
        .find(|(key, _)| key == "error")
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| "No token received".to_string());

    let response = r#"HTTP/1.1 400 Bad Request
Content-Type: text/html; charset=utf-8

<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Namekit - Authentication Failed</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="icon" href="https://namekit.app/images/icon512.png">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; text-align: center; margin: 0; padding: 40px 20px; background: linear-gradient(135deg, #ff6b6b 0%, #ee5a52 100%); color: white; min-height: 90vh; display: flex; flex-direction: column; justify-content: center; }
        .container { max-width: 500px; margin: 0 auto; }
        .logo { width: 80px; height: 80px; margin: 0 auto 20px; border-radius: 16px; }
        h1 { font-size: 2.5em; margin-bottom: 10px; }
        p { font-size: 1.2em; opacity: 0.9; margin-bottom: 30px; }
        .error-icon { font-size: 4em; margin-bottom: 20px; color: #f87171; }
    </style>
</head>
<body>
    <div class="container">
        <img src="https://namekit.app/images/icon512.png" alt="Namekit" class="logo">
        <div class="error-icon">❌</div>
        <h1>Authentication Failed</h1>
        <p>Something went wrong during authentication.</p>
        <p>Please close this window and try again.</p>
    </div>
</body>
</html>"#;

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    Err(AuthError::CallbackError(format!(
        "Authentication failed: {}",
        error
    )))
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

    let response = client
        .get(&info_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AuthError::ServerError(
            "Authentication failed - token may be expired or invalid".to_string(),
        ));
    }

    if !response.status().is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AuthError::ServerError(format!(
            "Failed to get user info: {}",
            error_text
        )));
    }

    let response_text = response.text().await?;
    let user_info: UserInfo = serde_json::from_str(&response_text).map_err(|e| {
        AuthError::ServerError(format!("Failed to parse user info response: {}", e))
    })?;

    Ok(user_info)
}
