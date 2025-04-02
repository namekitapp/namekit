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

# Search for an exact domain name (default: grid view, only available domains)
$ namekit exact example

# Search for an exact domain name with list view
$ namekit --output list exact example

# Show all domains including taken ones
$ namekit --show-taken exact example

# Hide premium domains
$ namekit --hide-premium exact example

# Search for domain names with multiple terms
$ namekit search term1 term2 term3

# Search with list view and show all domains
$ namekit -o list --show-taken search term1 term2 term3
```

## Command Structure

```
namekit [OPTIONS] <COMMAND>

Options:
  -o, --output <OUTPUT>  Output format: 'list' for single line or 'grid' for terminal-width grid [default: grid]
      --show-taken       Show taken domains (by default only available domains are shown)
      --hide-premium     Hide premium domains (by default premium domains are shown)
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
- Yellow: Premium domains
- Green: Available domains
- Red: Taken domains (only shown with --show-taken flag)

### List Mode
Displays each domain on a single line with color coding:
- Yellow: Premium domains
- Green: Available domains
- Red: Taken domains (only shown with --show-taken flag)

## Domain Filtering

By default, Namekit only shows available domains. You can control which domains are displayed with these flags:

- `--show-taken`: Shows all domains, including those that are already taken
- `--hide-premium`: Hides premium domains from the results

## License

[GPLv3](LICENSE)