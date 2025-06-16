# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```bash
# Build the project
cargo build

# Run with development profile
cargo run -- [ARGS]

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy

# Install locally for testing
cargo install --path .
```

## Architecture Overview

Namekit is a Rust CLI application for domain name discovery using the Namekit API. The architecture follows a modular design:

### Core Components

- `main.rs` - CLI argument parsing using clap, command routing, and main async runtime
- `api.rs` - Handles streaming HTTP requests to the Namekit API, processes server-sent events line by line
- `config.rs` - Manages user configuration (API token, server URL) stored in JSON format at `~/.config/namekit/config.json`
- `domain.rs` - Domain result data structures with availability and premium status
- `output.rs` - Three output modes: grid (terminal-width columns), list (single line), and JSON array

### Key Architecture Patterns

- **Async Streaming**: Uses tokio channels and streams to process domain results as they arrive from the API
- **Configuration Management**: Persistent config stored in user's config directory using the `dirs` crate
- **Modular Output**: Abstracted display logic supporting multiple output formats
- **Error Handling**: Custom error types with proper error propagation

### API Integration

The application connects to the Namekit API at `https://api.namekit.app` by default, using:
- Bearer token authentication
- Streaming endpoint `/domains/stream` with POST requests
- Server-sent events parsing for real-time domain results
- Rate limiting handling with user-friendly messages

### Data Flow

1. User provides search terms via CLI
2. Config loaded to get API token and server URL
3. Streaming request sent to API with query parameters
4. Results processed line-by-line as JSON objects
5. Domain objects filtered based on CLI flags (--show-taken, --hide-premium)
6. Results displayed in chosen format (grid/list/json)