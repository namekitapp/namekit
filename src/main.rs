use clap::{Parser, Subcommand};
use futures_util::StreamExt;
use output::{OutputMode, display_results};

mod api;
mod auth;
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

    /// Authenticate with OAuth providers
    Auth {
        #[command(subcommand)]
        action: AuthCommands,
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
    /// Set the API server URL
    SetApiServer {
        /// The API server URL to use (default: https://api.namekit.app)
        server: String,
    },

    /// Show the current configuration
    Show,
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Login with Google OAuth
    Google,

    /// Login with GitHub OAuth
    Github,

    /// Show current authentication status
    Status,

    /// Get user information from auth server
    Info,

    /// Logout and clear stored tokens
    Logout,
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
                    // Load config to get the OAuth token
                    let config = config::Config::load()?;
                    let token = match config.get_auth_token() {
                        Ok(token) => token,
                        Err(_) => {
                            eprintln!("Authentication required");
                            eprintln!(
                                "Run 'namekit auth google' or 'namekit auth github' to get started"
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
                            eprintln!("Failed to search domains: {}", e);
                        }
                    }
                }
                SearchMode::Tld { query } => {
                    // Load config to get the OAuth token
                    let config = config::Config::load()?;
                    let token = match config.get_auth_token() {
                        Ok(token) => token,
                        Err(_) => {
                            eprintln!("Authentication required");
                            eprintln!(
                                "Run 'namekit auth google' or 'namekit auth github' to get started"
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
                            eprintln!("Failed to search domains: {}", e);
                        }
                    }
                }
            }
        }
        Commands::Config { action } => match action {
            ConfigCommands::SetApiServer { server } => {
                let mut config = config::Config::load()?;
                config.set_api_server(server.clone())?;
                println!("API server updated to: {}", server);

                let path = config::get_config_path();
                println!("Configuration saved to: {}", path.display());
            }
            ConfigCommands::Show => {
                let config = config::Config::load()?;
                println!("Namekit Configuration");
                println!();

                // Show authentication status
                match config.get_auth_token() {
                    Ok(_) => println!("Authentication: Authenticated"),
                    Err(_) => println!("Authentication: Not authenticated"),
                }

                // Show the API server
                println!("API Server: {}", config.get_api_server());
                println!();

                let path = config::get_config_path();
                println!("Config file: {}", path.display());
            }
        },
        Commands::Auth { action } => match action {
            AuthCommands::Google => match auth::login(auth::AuthProvider::Google).await {
                Ok(token) => {
                    let mut config = config::Config::load()?;
                    config.set_auth_token(token.access_token)?;
                    println!("Successfully authenticated with Google");
                    println!("You can now search for domains!");
                }
                Err(e) => {
                    eprintln!("Google authentication failed: {}", e);
                }
            },
            AuthCommands::Github => match auth::login(auth::AuthProvider::GitHub).await {
                Ok(token) => {
                    let mut config = config::Config::load()?;
                    config.set_auth_token(token.access_token)?;
                    println!("Successfully authenticated with GitHub");
                    println!("You can now search for domains!");
                }
                Err(e) => {
                    eprintln!("GitHub authentication failed: {}", e);
                }
            },
            AuthCommands::Status => {
                let config = config::Config::load()?;
                match config.get_auth_token() {
                    Ok(_) => println!("Status: Authenticated"),
                    Err(_) => println!("Status: Not authenticated"),
                }
            }
            AuthCommands::Info => {
                let config = config::Config::load()?;
                match config.get_auth_token() {
                    Ok(token) => match auth::get_user_info(&token).await {
                        Ok(user_info) => {
                            println!("User Information");
                            println!();
                            println!("Email: {}", user_info.email);
                            println!("Tier: {}", user_info.tier);
                        }
                        Err(e) => {
                            eprintln!("Failed to get user info: {}", e);
                        }
                    },
                    Err(_) => {
                        eprintln!("Not authenticated");
                        eprintln!(
                            "Run 'namekit auth google' or 'namekit auth github' to get started"
                        );
                    }
                }
            }
            AuthCommands::Logout => {
                let mut config = config::Config::load()?;
                config.clear_auth_token()?;
                println!("Successfully logged out");
                println!("Your authentication token has been cleared");
            }
        },
    }

    Ok(())
}
