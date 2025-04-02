use crossterm::{
    execute,
    style::{Color as CrosstermColor, ResetColor, SetForegroundColor},
};
use std::io;

/// Represents a domain name and its availability status
pub struct DomainResult {
    pub name: String,
    pub available: bool,
}

/// Output mode for displaying results
pub enum OutputMode {
    /// Simple list with one domain per line
    List,
    /// Grid view that fills the terminal width
    Grid,
}

/// Display domain results in a simple list format, one domain per line
pub fn display_list(results: &[DomainResult]) -> io::Result<()> {
    for result in results {
        let status = if result.available {
            "Available"
        } else {
            "Taken"
        };
        let color = if result.available {
            CrosstermColor::Green
        } else {
            CrosstermColor::Red
        };

        // Use crossterm's coloring
        execute!(io::stdout(), SetForegroundColor(color),)?;

        print!("{}", result.name);

        execute!(io::stdout(), ResetColor,)?;

        println!(" ({})", status);
    }
    Ok(())
}

/// Display domain results in a grid view using standard terminal output
pub fn display_grid(results: &[DomainResult]) -> io::Result<()> {
    // Calculate terminal width
    let (width, _) = crossterm::terminal::size()?;

    // Calculate optimal number of columns based on terminal width
    let max_domain_length = results.iter().map(|r| r.name.len()).max().unwrap_or(20);

    let column_width = max_domain_length + 4; // Add some padding
    let num_columns = std::cmp::max(1, width as usize / column_width);

    // Print header
    println!("\n{}", "Domain Name Search Results".to_string());
    println!("{}\n", "=".repeat(width as usize - 2));

    // Print results in a grid
    let mut current_col = 0;

    for result in results {
        // Determine color based on availability
        let color = if result.available {
            CrosstermColor::Green
        } else {
            CrosstermColor::Red
        };

        // Use crossterm's coloring
        execute!(io::stdout(), SetForegroundColor(color),)?;

        // Print the domain with padding
        print!("{:<width$}", result.name, width = column_width);

        // Reset color
        execute!(io::stdout(), ResetColor,)?;

        current_col += 1;

        // Move to next line if we've filled the row
        if current_col >= num_columns {
            println!();
            current_col = 0;
        }
    }

    // Ensure we end with a newline
    if current_col > 0 {
        println!();
    }

    println!();

    Ok(())
}

/// Display domain search results based on the specified output mode
pub fn display_results(results: &[DomainResult], mode: OutputMode) -> io::Result<()> {
    match mode {
        OutputMode::List => display_list(results),
        OutputMode::Grid => display_grid(results),
    }
}

/// Generate test results for demonstration purposes
pub fn generate_test_results() -> Vec<DomainResult> {
    vec![
        DomainResult {
            name: "example.com".to_string(),
            available: false,
        },
        DomainResult {
            name: "example.net".to_string(),
            available: false,
        },
        DomainResult {
            name: "example.org".to_string(),
            available: false,
        },
        DomainResult {
            name: "example-site.com".to_string(),
            available: true,
        },
        DomainResult {
            name: "my-example.com".to_string(),
            available: true,
        },
        DomainResult {
            name: "exampleapp.io".to_string(),
            available: true,
        },
        DomainResult {
            name: "example-store.com".to_string(),
            available: false,
        },
        DomainResult {
            name: "best-example.net".to_string(),
            available: true,
        },
        DomainResult {
            name: "example-blog.com".to_string(),
            available: false,
        },
        DomainResult {
            name: "example-project.dev".to_string(),
            available: true,
        },
        DomainResult {
            name: "example-cloud.io".to_string(),
            available: true,
        },
        DomainResult {
            name: "example-tech.co".to_string(),
            available: false,
        },
        DomainResult {
            name: "example-app.net".to_string(),
            available: true,
        },
        DomainResult {
            name: "example-service.org".to_string(),
            available: false,
        },
        DomainResult {
            name: "example-platform.com".to_string(),
            available: true,
        },
    ]
}
