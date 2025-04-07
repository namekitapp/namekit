use crate::domain::DomainResult;
use crossterm::{
    execute,
    style::{Color as CrosstermColor, ResetColor, SetForegroundColor},
};
use futures_core::stream::Stream;
use futures_util::StreamExt;
use std::io;
use serde_json;

pub enum OutputMode {
    List,
    Grid,
    Json,
}

pub async fn display_list<S>(stream: S) -> io::Result<()>
where
    S: Stream<Item = DomainResult> + Unpin,
{
    let mut stream = Box::pin(stream);

    while let Some(result) = stream.next().await {
        let color = if result.premium {
            CrosstermColor::Yellow
        } else if result.available {
            CrosstermColor::Green
        } else {
            CrosstermColor::Red
        };
        execute!(io::stdout(), SetForegroundColor(color))?;
        println!("{}", result.name);
        execute!(io::stdout(), ResetColor)?;
    }
    Ok(())
}

pub async fn display_grid<S>(stream: S) -> io::Result<()>
where
    S: Stream<Item = DomainResult> + Unpin,
{
    let (width, _) = crossterm::terminal::size()?;

    // Use static max domain length of 20 as specified
    let max_domain_length = 20;

    let column_width = max_domain_length + 4; // Add some padding
    let num_columns = std::cmp::max(1, width as usize / column_width);

    let mut current_col = 0;
    let mut stream = Box::pin(stream);

    while let Some(result) = stream.next().await {
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

/// Display domain search results in JSON format
pub async fn display_json<S>(stream: S) -> io::Result<()>
where
    S: Stream<Item = DomainResult> + Unpin,
{
    let mut stream = Box::pin(stream);
    
    // Print opening bracket for JSON array
    println!("[");
    
    let mut first = true;
    
    // Print each result as a JSON object on a single line
    while let Some(result) = stream.next().await {
        // Add comma after previous item
        if !first {
            println!(",");
        }
        
        // Convert domain result to JSON
        let json_str = serde_json::to_string(&result).unwrap_or_default();
        print!("{}", json_str);
        
        if first {
            first = false;
        }
    }
    
    // Add newline after the last item
    if !first {
        println!();
    }
    
    // Print closing bracket for JSON array
    println!("]");
    
    Ok(())
}

/// Display domain search results based on the specified output mode
pub async fn display_results<S>(stream: S, mode: OutputMode) -> io::Result<()>
where
    S: Stream<Item = DomainResult> + Unpin,
{
    match mode {
        OutputMode::List => display_list(stream).await,
        OutputMode::Grid => display_grid(stream).await,
        OutputMode::Json => display_json(stream).await,
    }
}
