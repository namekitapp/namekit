use crate::domain::DomainResult;
use crossterm::{
    execute,
    style::{Color as CrosstermColor, ResetColor, SetForegroundColor},
};
use std::io;

pub enum OutputMode {
    List,
    Grid,
}

pub fn display_list(results: &[DomainResult]) -> io::Result<()> {
    for result in results {
        let status = if result.available {
            "Available"
        } else {
            "Taken"
        };
        let premium_status = if result.premium { " (Premium)" } else { "" };

        let color = if result.premium {
            CrosstermColor::Yellow
        } else if result.available {
            CrosstermColor::Green
        } else {
            CrosstermColor::Red
        };

        execute!(io::stdout(), SetForegroundColor(color),)?;

        print!("{}", result.name);

        execute!(io::stdout(), ResetColor,)?;

        println!(" ({}){}", status, premium_status);
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
        let color = if result.premium {
            CrosstermColor::Yellow
        } else if result.available {
            CrosstermColor::Green
        } else {
            CrosstermColor::Red
        };

        execute!(io::stdout(), SetForegroundColor(color),)?;

        print!("{:<width$}", result.name, width = column_width);

        execute!(io::stdout(), ResetColor,)?;

        current_col += 1;

        if current_col >= num_columns {
            println!();
            current_col = 0;
        }
    }

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
        DomainResult::new("example.com".to_string(), false),
        DomainResult::new("example.net".to_string(), false),
        DomainResult::new("example.org".to_string(), false),
        DomainResult::new("example-site.com".to_string(), true),
        DomainResult::new("my-example.com".to_string(), true),
        DomainResult::new_with_premium("exampleapp.io".to_string(), true, true),
        DomainResult::new("example-store.com".to_string(), false),
        DomainResult::new_with_premium("best-example.net".to_string(), true, true),
        DomainResult::new("example-blog.com".to_string(), false),
        DomainResult::new("example-project.dev".to_string(), true),
        DomainResult::new("example-cloud.io".to_string(), true),
        DomainResult::new_with_premium("example-tech.co".to_string(), false, true),
        DomainResult::new("example-app.net".to_string(), true),
        DomainResult::new("example-service.org".to_string(), false),
        DomainResult::new("example-platform.com".to_string(), true),
    ]
}
