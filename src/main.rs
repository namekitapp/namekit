use clap::{Parser, Subcommand};
use domain::DomainResult;
use output::{OutputMode, display_results, generate_test_results};

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
}

fn main() -> std::io::Result<()> {
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

            // Display the results
            display_results(&results, output_mode)?;
        }
        Commands::Search { terms } => {
            println!("Searching for domains with terms: {:?}", terms);

            // For demonstration, generate test results
            let results = generate_test_results();

            // Display the results
            display_results(&results, output_mode)?;
        }
    }

    Ok(())
}
