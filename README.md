# TAP - Transfer and Analyze Project

A high-performance file investigation and export tool that intelligently categorizes files for LLM analysis. Built with Rust and Nix for maximum reliability and reproducibility.

## Features

- **Smart File Categorization**: Automatically categorizes files based on their type and content
- **Interactive Device Selection**: Pick drives and devices through an intuitive TUI
- **Flexible Export Options**: Export files individually or as compressed archives
- **LLM-Optimized**: Organizes files in a structure ideal for AI analysis workflows
- **Async Performance**: Blazing-fast file operations using Tokio
- **Beautiful Terminal UI**: Color-themed interface with proper terminal size validation

## Installation

### Using Nix Flakes (Recommended)

```bash
# Clone the repository
git clone git@github.com:rottedfm/tap.git
cd tap

# Enter development shell
nix develop

# Build the project
nix build
```

### Using Cargo

```bash
cargo build --release
```

## Usage

### Inspect a Drive

Scan and categorize files on a drive:

```bash
tap inspect [DRIVE]
```

If no drive is specified, an interactive picker will be shown.

### Export Files

Export categorized files to a directory:

```bash
tap export [OPTIONS] [DRIVE]

Options:
  -o, --output-dir <PATH>  Output directory (default: ./export)
  -z, --zip                Create a zip archive of exported files
```

## Configuration

TAP can be configured via a TOML configuration file. See the default configuration for available options including UI theming and categorization rules.

## Development

This project uses Nix flakes for reproducible development environments:

```bash
# Enter development shell
nix develop

# Run directly via cargo
nix develop -c cargo run

# Build release binary
nix build

# Update dependencies
nix flake update
```

### Pre-commit Hooks

Pre-commit hooks are automatically setup in the Nix shell for code formatting and linting:

```bash
pre-commit run -a
```

### Available Commands

We provide a [`justfile`](https://just.systems/) for common development tasks:

```bash
just --list
```

## Project Structure

- `src/categories.rs` - File categorization logic
- `src/cli.rs` - Command-line argument parsing
- `src/config.rs` - Configuration management
- `src/device_picker.rs` - Interactive device selection
- `src/export.rs` - File export functionality
- `src/inspect.rs` - Drive inspection logic
- `src/scanner.rs` - File system scanning
- `src/tui.rs` - Terminal UI components
- `src/zip.rs` - Archive creation utilities

## Requirements

- Rust 1.85+
- Nix with flakes enabled (for Nix-based builds)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
