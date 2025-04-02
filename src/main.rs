use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "namekit")]
#[command(version = "0.0.1")]
#[command(about = "A command line toolkit for quickly exploring domain names available for registration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for an exact domain name
    Exact {
        /// The exact term to search for
        term: String,
    },

    /// Search for domain names based on multiple terms
    Search {
        /// Search terms
        #[arg(required = true)]
        terms: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Exact { term } => {
            println!("Searching for exact domain: {}", term);
            // Implement exact domain search logic here
        }
        Commands::Search { terms } => {
            println!("Searching for domains with terms: {:?}", terms);
            // Implement multi-term domain search logic here
        }
    }
}
