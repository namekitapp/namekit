use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

// Simple error enum without thiserror
#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    JsonError(serde_json::Error),
    TokenNotSet,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::JsonError(e) => write!(f, "JSON error: {}", e),
            ConfigError::TokenNotSet => write!(f, "Token not set"),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::JsonError(err)
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_token: Option<String>,
    pub api_server: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_token: None,
            api_server: Some("https://api.namekit.app".to_string()),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config_path = get_config_path();

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let mut file = fs::File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = get_config_path();

        // Ensure the directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(config_path)?;
        file.write_all(contents.as_bytes())?;

        Ok(())
    }

    pub fn set_token(&mut self, token: String) -> Result<(), ConfigError> {
        self.api_token = Some(token);
        self.save()?;
        Ok(())
    }

    pub fn get_token(&self) -> Result<String, ConfigError> {
        self.api_token.clone().ok_or(ConfigError::TokenNotSet)
    }

    pub fn set_api_server(&mut self, server: String) -> Result<(), ConfigError> {
        self.api_server = Some(server);
        self.save()?;
        Ok(())
    }

    pub fn get_api_server(&self) -> String {
        self.api_server
            .clone()
            .unwrap_or_else(|| "https://api.namedrop.dev".to_string())
    }
}

// Helper function to get the config path using dirs crate
pub fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("namekit");
    path.push("config.json");
    path
}
