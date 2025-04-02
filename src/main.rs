use clap::{Parser, Subcommand};
use domain::DomainResult;
use output::{OutputMode, display_results, generate_test_results};

mod domain;
mod output;
mod api;

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
            
            // Call the API to get domain results
            println!("Connecting to API at http://127.0.0.1:8000/domains/prompt...");
            println!("Streaming results (this may take a moment):");
            
            match api::prompt_domains(query).await {
                Ok(results) => {
                    println!("\nReceived {} domain results", results.len());
                    
                    // Filter results based on flags
                    let filtered_results = filter_results(&results, cli.show_taken, cli.hide_premium);
                    
                    // Display the filtered results
                    display_results(&filtered_results, output_mode)?;
                }
                Err(e) => {
                    eprintln!("Error fetching domain results: {}", e);
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
