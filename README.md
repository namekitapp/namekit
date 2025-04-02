# Namekit

A command line toolkit for quickly exploring domain names available for registration.

## Installation

Globally install from `cargo`, installed by default to `~/.cargo/bin` -

```sh
$ cargo install namekit
```

## Usage

```sh
# Show help information
$ namekit --help

# Search for an exact domain name (list view)
$ namekit --output list exact example

# Search for an exact domain name with grid view (default)
$ namekit exact example

# Search for domain names with multiple terms
$ namekit search term1 term2 term3

# Search with list view output
$ namekit -o list search term1 term2 term3
```

## Command Structure

```
namekit [OPTIONS] <COMMAND>

Options:
  -o, --output <OUTPUT>  Output format: 'list' for single line or 'grid' for terminal-width grid [default: grid]
  -h, --help             Print help
  -V, --version          Print version

Commands:
  exact    Search for an exact domain name
  search   Search for domain names based on multiple terms
  help     Print this message or the help of the given subcommand(s)
```

## Output Modes

Namekit supports two output modes:

### Grid Mode (default)
Displays domains in a grid that fills the terminal width, with color coding:
- Green: Available domains
- Red: Taken domains

### List Mode
Displays each domain on a single line with color coding:
- Green: Available domains
- Red: Taken domains

## License

[GPLv3](LICENSE)