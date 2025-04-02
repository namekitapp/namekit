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

# Search for an exact domain name
$ namekit exact example

# Search for domain names with multiple terms
$ namekit search term1 term2 term3
```

## Command Structure

```
namekit <COMMAND>

Commands:
  exact    Search for an exact domain name
  search   Search for domain names based on multiple terms
  help     Print this message or the help of the given subcommand(s)