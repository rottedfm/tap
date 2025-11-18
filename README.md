# TAP

**Transfer and Analyze Project** - A blazingly fast file investigation and export tool that intelligently categorizes files from mountable drives.

Built with Rust for maximum performance and reliability.

![TAP Help Screen](demos/help.gif)

## Features

- **Smart File Categorization** - Automatically organizes files by type and content (documents, images, videos, code, archives, etc.)
- **Interactive Device Selection** - Choose drives and devices through an intuitive TUI
- **Flexible Export Options** - Export categorized files individually or as compressed archives
- **LLM-Optimized Output** - Structures files in a format ideal for large language model analysis workflows
- **Async Performance** - Lightning-fast file operations powered by Tokio
- **Beautiful Terminal UI** - Color-themed interface with proper terminal size validation

## Installation

```bash
git clone git@github.com:rottedfm/tap.git
cd tap
cargo build --release
```

**Requirements**: Rust 1.85+

The compiled binary will be available at `target/release/tap`.

## Usage

### Inspect a Drive

Scan and categorize all files on a drive:

```bash
cargo run --release -- inspect [DRIVE]
```

If no drive is specified, an interactive picker will be shown.

### Export Files

Export categorized files to a directory:

```bash
cargo run --release -- export [OPTIONS] [DRIVE]

Options:
  -o, --output-dir <PATH>  Output directory (default: ./export)
  -z, --zip                Create a zip archive of exported files
```

## Configuration

TAP can be configured via a TOML configuration file for customizing UI themes and categorization rules.

## Development

```bash
# Run directly
cargo run

# Build release binary
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Project Structure

```
src/
├── categories.rs      - File categorization logic
├── cli.rs             - Command-line argument parsing
├── config.rs          - Configuration management
├── device_picker.rs   - Interactive device selection
├── export.rs          - File export functionality
├── inspect.rs         - Drive inspection logic
├── scanner.rs         - File system scanning
├── tui.rs             - Terminal UI components
└── zip.rs             - Archive creation utilities
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

Contributions are welcome! Please feel free to submit a Pull Request.
