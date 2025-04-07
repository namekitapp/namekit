use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use output::{OutputMode, display_results};

mod api;
mod config;
mod domain;
mod output;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "namekit")]
#[command(version = VERSION)]
#[command(about = "A command line toolkit for quickly exploring domain names available for registration", long_about = None)]
struct Cli {
    /// Output format: 'list' for single line, 'grid' for terminal-width grid, or 'json' for JSON array output
    #[arg(short, long, default_value = "grid")]
    output: String,

    /// Show taken domains (by default only available domains are shown)
    #[arg(long)]
    show_taken: bool,

    /// Hide premium domains (by default premium domains are shown)
    #[arg(long)]
    hide_premium: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for domain names
    Search {
        #[command(subcommand)]
        mode: SearchMode,
    },

    /// Configure the application
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum SearchMode {
    /// Search for domains using AI-powered suggestions
    AI {
        /// Terms to use for domain search
        #[arg(required = true)]
        terms: Vec<String>,
    },

    /// Search for a specific domain name with different TLDs
    Tld {
        /// Domain name to check with different TLDs
        #[arg(required = true)]
        query: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set the API token for accessing the domain API
    SetToken {
        /// The API token to set
        token: String,
    },

    /// Set the API server URL
    SetApiServer {
        /// The API server URL to use (default: https://api.namedrop.dev)
        server: String,
    },

    /// Show the current configuration
    Show,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Determine output mode
    let output_mode = match cli.output.to_lowercase().as_str() {
        "grid" => OutputMode::Grid,
        "json" => OutputMode::Json,
        _ => OutputMode::List,
    };

    match &cli.command {
        Commands::Search { mode } => {
            match mode {
                SearchMode::AI { terms } => {
                    // Load config to get the API token
                    let config = config::Config::load()?;
                    let token = match config.get_token() {
                        Ok(token) => token,
                        Err(_) => {
                            eprintln!(
                                "API token not set. Please set a token with 'namekit config set-token <TOKEN>'"
                            );
                            return Ok(());
                        }
                    };

                    match api::stream_domains(&terms.join(" "), "ai", &token).await {
                        Ok(domain_stream) => {
                            // Filter the stream based on flags
                            let filtered_stream = domain_stream
                                .filter(move |domain| {
                                    let show = (domain.available || cli.show_taken)
                                        && (!domain.premium || !cli.hide_premium);
                                    async move { show }
                                })
                                .boxed(); // Box the stream to make it Unpin

                            // Display the filtered results
                            display_results(filtered_stream, output_mode).await?;
                        }
                        Err(e) => {
                            eprintln!("Error fetching domain results: {}", e);
                        }
                    }
                }
                SearchMode::Tld { query } => {
                    // Load config to get the API token
                    let config = config::Config::load()?;
                    let token = match config.get_token() {
                        Ok(token) => token,
                        Err(_) => {
                            eprintln!(
                                "API token not set. Please set a token with 'namekit config set-token <TOKEN>'"
                            );
                            return Ok(());
                        }
                    };

                    match api::stream_domains(query, "tld", &token).await {
                        Ok(domain_stream) => {
                            // Filter the stream based on flags
                            let filtered_stream = domain_stream
                                .filter(move |domain| {
                                    let show = (domain.available || cli.show_taken)
                                        && (!domain.premium || !cli.hide_premium);
                                    async move { show }
                                })
                                .boxed(); // Box the stream to make it Unpin

                            // Display the filtered results
                            display_results(filtered_stream, output_mode).await?;
                        }
                        Err(e) => {
                            eprintln!("Error fetching domain results: {}", e);
                        }
                    }
                }
            }
        }
        Commands::Config { action } => {
            match action {
                ConfigCommands::SetToken { token } => {
                    let mut config = config::Config::load()?;
                    config.set_token(token.clone())?;
                    println!("API token set successfully");

                    // Show the config file path for reference
                    let path = config::get_config_path();
                    println!("Configuration saved to: {}", path.display());
                }
                ConfigCommands::SetApiServer { server } => {
                    let mut config = config::Config::load()?;
                    config.set_api_server(server.clone())?;
                    println!("API server set to: {}", server);

                    // Show the config file path for reference
                    let path = config::get_config_path();
                    println!("Configuration saved to: {}", path.display());
                }
                ConfigCommands::Show => {
                    let config = config::Config::load()?;
                    println!("Current configuration:");

                    match config.get_token() {
                        Ok(token) => {
                            // Only show a masked version of the token for security
                            let masked_token = if token.len() > 8 {
                                format!("{}...{}", &token[0..4], &token[token.len() - 4..])
                            } else {
                                "****".to_string()
                            };
                            println!("API Token: {}", masked_token);
                        }
                        Err(_) => {
                            println!("API Token: Not set");
                        }
                    }

                    // Show the API server
                    println!("API Server: {}", config.get_api_server());

                    let path = config::get_config_path();
                    println!("Configuration file: {}", path.display());
                }
            }
        }
    }

    Ok(())
}
