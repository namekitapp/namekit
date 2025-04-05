# Namekit

A command line toolkit for quickly exploring domain names available for registration via **https://namekit.app**

![Namekit CLI demo](demo.gif)


## Installation

Globally install from `cargo`, installed by default to `~/.cargo/bin` -

```sh
$ cargo install namekit
```

## Usage

```sh
# Show help information
$ namekit --help

# Search for domain names using AI suggestions
$ namekit search ai tech startup saas

# Search for a specific domain name with different TLDs
$ namekit search tld example

# Search with list view instead of grid view
$ namekit --output list search ai tech startup

# Show all domains including taken ones
$ namekit --show-taken search tld mydomain

# Hide premium domains
$ namekit --hide-premium search ai business app

# Configure your API token
$ namekit config set-token YOUR_API_TOKEN

# Set a custom API server
$ namekit config set-api-server https://custom-api-server.com

# View your current configuration
$ namekit config show
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
  search   Search for domain names
  config   Configure the application
  help     Print this message or the help of the given subcommand(s)

Search Commands:
  ai       Search for domains using AI-powered suggestions
  tld      Search for a specific domain name with different TLDs

Config Commands:
  set-token       Set the API token for accessing the domain API
  set-api-server  Set the API server URL
  show            Show the current configuration
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