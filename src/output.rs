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
        let color_code = if result.available { 32 } else { 31 }; // 32 for green, 31 for red
        println!("\x1b[{}m{}\x1b[0m ({})", color_code, result.name, status);
    }
    Ok(())
}

pub fn display_grid(results: &[DomainResult]) -> io::Result<()> {
    let (width, _) = crossterm::terminal::size()?;

    let max_domain_length = results.iter().map(|r| r.name.len()).max().unwrap_or(20);

    let column_width = max_domain_length + 4; // Add some padding
    let num_columns = std::cmp::max(1, width as usize / column_width);

    println!("\n{}", "Domain Name Search Results".to_string());
    println!("{}\n", "=".repeat(width as usize - 2));

    let mut current_col = 0;

    for result in results {
        let color_code = if result.available { 32 } else { 31 }; // 32 for green, 31 for red

        print!(
            "\x1b[{}m{:<width$}\x1b[0m",
            color_code,
            result.name,
            width = column_width
        );

        current_col += 1;

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

pub fn display_results(results: &[DomainResult], mode: OutputMode) -> io::Result<()> {
    match mode {
        OutputMode::List => display_list(results),
        OutputMode::Grid => display_grid(results),
    }
}

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
