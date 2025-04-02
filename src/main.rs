use clap::{Parser, Subcommand};
use domain::DomainResult;
use output::{OutputMode, display_results, generate_test_results};

mod api;
mod config;
mod domain;
mod output;

#[derive(Parser)]
#[command(name = "namekit")]
#[command(version = "0.0.1")]
#[command(about = "A command line toolkit for quickly exploring domain names available for registration", long_about = None)]
struct Cli {
    /// Output format: 'list' for single line or 'grid' for terminal-width grid
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
    Exact {
        term: String,
    },

    Search {
        #[arg(required = true)]
        terms: Vec<String>,
    },

    Prompt {
        /// Query to send to the domain prompt API
        query: String,
    },

    Config {
        #[command(subcommand)]
        action: ConfigCommands,
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
        _ => OutputMode::List,
    };

    match &cli.command {
        Commands::Exact { term } => {
            println!("Searching for exact domain: {}", term);

            // For demonstration, generate test results with the exact term
            let mut results = generate_test_results();
            // Add the exact term as first result
            results.insert(
                0,
                DomainResult::new(
                    format!("{}.com", term),
                    term.len() > 10, // Just a simple rule for demo purposes
                ),
            );

            // Filter results based on flags
            let filtered_results = filter_results(&results, cli.show_taken, cli.hide_premium);

            // Display the filtered results
            display_results(&filtered_results, output_mode)?;
        }
        Commands::Search { terms } => {
            println!("Searching for domains with terms: {:?}", terms);

            // For demonstration, generate test results
            let results = generate_test_results();

            // Filter results based on flags
            let filtered_results = filter_results(&results, cli.show_taken, cli.hide_premium);

            // Display the filtered results
            display_results(&filtered_results, output_mode)?;
        }
        Commands::Prompt { query } => {
            println!("Sending prompt to API: {}", query);

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

            // Call the API to get domain results
            println!("Connecting to API...");

            match api::prompt_domains(query, &token).await {
                Ok(results) => {
                    println!("\nReceived {} domain results", results.len());

                    // Filter results based on flags
                    let filtered_results =
                        filter_results(&results, cli.show_taken, cli.hide_premium);

                    // Display the filtered results
                    display_results(&filtered_results, output_mode)?;
                }
                Err(e) => {
                    eprintln!("Error fetching domain results: {}", e);
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

/// Filter domain results based on show_taken and hide_premium flags
fn filter_results(
    results: &[DomainResult],
    show_taken: bool,
    hide_premium: bool,
) -> Vec<DomainResult> {
    results
        .iter()
        .filter(|result| {
            // Filter out taken domains if show_taken is false
            if !show_taken && !result.available {
                return false;
            }

            // Filter out premium domains if hide_premium is true
            if hide_premium && result.premium {
                return false;
            }

            true
        })
        .cloned()
        .collect()
}
