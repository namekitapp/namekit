use crate::domain::DomainResult;
use crossterm::{
    execute,
    style::{Color as CrosstermColor, ResetColor, SetForegroundColor},
};
use futures_core::stream::Stream;
use futures_util::StreamExt;
use std::io;

pub enum OutputMode {
    List,
    Grid,
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

/// Display domain search results based on the specified output mode
pub async fn display_results<S>(stream: S, mode: OutputMode) -> io::Result<()>
where
    S: Stream<Item = DomainResult> + Unpin,
{
    match mode {
        OutputMode::List => display_list(stream).await,
        OutputMode::Grid => display_grid(stream).await,
    }
}
