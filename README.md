# TAP

**Transfer and Analyze Project** - A high-performance file investigation and export tool designed for digital forensics, data recovery, and LLM-assisted analysis workflows.

Built with Rust for maximum performance and reliability on Linux systems.

## Overview

TAP scans mountable drives, categorizes files by type, and exports them in an organized structure. It's particularly useful for:

- **Digital Forensics** - Quickly catalog and extract files from evidence drives
- **Data Recovery** - Organize recovered files by type for easier analysis
- **LLM Analysis** - Structure files in formats optimized for AI-powered investigation
- **Drive Migration** - Systematically transfer and categorize files between systems

## Features

- **Intelligent File Categorization** - Automatically sorts files into 11+ categories based on extension
- **Interactive Device Selection** - TUI-based drive picker when no path is specified
- **Flexible Export Formats** - Save as directory structure or compressed ZIP archives
- **Comprehensive Logging** - Generate detailed inspection reports
- **Async I/O** - Lightning-fast concurrent file operations with Tokio
- **Configurable** - Customize categories and UI themes via TOML configuration

## Installation

### Prerequisites

- Linux operating system
- Rust 1.85 or later
- Cargo (included with Rust)

### Build from Source

```bash
git clone git@github.com:rottedfm/tap.git
cd tap
cargo build --release
```

The compiled binary will be at `target/release/tap`.

### Optional: Install Globally

```bash
cargo install --path .
```

## Commands

TAP provides two primary commands: `inspect` and `export`.

### inspect - Catalog Drive Contents

Scans a drive and categorizes all files by type, displaying statistics and optional logging.

**Syntax:**
```bash
tap inspect [DRIVE] [OPTIONS]
```

**Arguments:**
- `DRIVE` - Optional. Path to drive or directory (e.g., `/dev/sda`, `/mnt/evidence`, or `/path/to/folder`)
  - If omitted, an interactive device picker is displayed

**Options:**
- `--log` - Write a text summary of inspection results to disk
  - Output file: `tap_inspect_<timestamp>.txt`

**Examples:**
```bash
# Interactive mode - pick from available devices
tap inspect

# Inspect specific drive
tap inspect /dev/sdb1

# Inspect directory with logging
tap inspect /mnt/evidence --log

# Inspect mounted USB drive
tap inspect /media/usb
```

**Output:**
Displays categorized file counts and total sizes:
```
Documents:    142 files (45.2 MB)
Images:       1,893 files (2.3 GB)
Videos:       87 files (12.1 GB)
Code:         3,421 files (89.4 MB)
...
```

---

### export - Extract Organized Files

Exports files from a drive, organized by category into separate folders.

**Syntax:**
```bash
tap export [DRIVE] [OPTIONS]
```

**Arguments:**
- `DRIVE` - Optional. Path to drive or directory to export from
  - If omitted, interactive device picker is displayed

**Options:**
- `-o, --output-dir <PATH>` - **Required.** Destination directory for exported files
- `--zip` - Create a ZIP archive instead of directory structure

**Examples:**
```bash
# Export to directory
tap export /dev/sdb1 --output-dir ./recovered_files

# Export with interactive drive selection
tap export -o ./evidence_export

# Create compressed archive
tap export /mnt/usb --output-dir ./backup --zip

# Export current directory
tap export . -o ./organized
```

**Output Structure:**
```
output_dir/
├── documents/
│   ├── file1.pdf
│   └── file2.docx
├── images/
│   ├── photo1.jpg
│   └── photo2.png
├── videos/
│   └── video1.mp4
├── code/
│   ├── script.py
│   └── main.rs
└── ... (other categories)
```

With `--zip`, creates: `output_dir.zip`

## File Categories

TAP automatically categorizes files into the following types:

| Category      | Extensions |
|---------------|------------|
| **Documents** | `.doc`, `.docx`, `.pdf`, `.odt`, `.rtf`, `.txt`, `.md` |
| **Spreadsheets** | `.xls`, `.xlsx`, `.ods`, `.csv` |
| **Images** | `.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`, `.tiff`, `.tif`, `.svg`, `.heic`, `.webp`, `.ico` |
| **Videos** | `.mp4`, `.avi`, `.mov`, `.mkv`, `.wmv`, `.flv`, `.webm`, `.m4v`, `.mpg`, `.mpeg` |
| **Audio** | `.mp3`, `.wav`, `.flac`, `.aac`, `.ogg`, `.m4a`, `.wma` |
| **Archives** | `.zip`, `.rar`, `.7z`, `.tar`, `.gz`, `.bz2`, `.xz` |
| **Email** | `.eml`, `.msg`, `.pst`, `.ost`, `.mbox` |
| **Databases** | `.db`, `.sqlite`, `.sqlite3`, `.mdb`, `.accdb` |
| **Code** | `.py`, `.js`, `.html`, `.css`, `.xml`, `.json`, `.yaml`, `.yml`, `.php`, `.cpp`, `.c`, `.h`, `.java`, `.rs`, `.go` |
| **Config** | `.ini`, `.conf`, `.cfg`, `.config` |
| **Logs** | `.log` |
| **Misc** | All other file types |

## Configuration

TAP uses a TOML configuration file located at `~/.config/tap/config.toml`. On first run, a default configuration is automatically created.

### Configuration File Structure

```toml
[export]
max_concurrent_copies = 10  # Maximum parallel file copy operations

[zip]
enabled = true              # Enable ZIP compression support
compression_level = 6       # Compression level (0-9, higher = better compression but slower)
buffer_size_kb = 256        # Buffer size in kilobytes for ZIP operations

[ui]
max_recent_files = 10       # Number of recent files to display in UI

[ui.color]
theme = "default"           # Color theme: default, cyan, magenta, yellow, green, red, blue, white

[scan]
exclude_patterns = [        # Patterns to exclude from scanning
    ".*",                   # Hidden files/directories
    "System Volume Information",
    "$RECYCLE.BIN",
    "node_modules"
]

[mount]
mount_base_dir = "/mnt"     # Base directory for mounting drives
mount_prefix = "tap_"       # Prefix for mount point names
device_patterns = [         # Device patterns to detect
    "/dev/sd",              # SATA drives
    "/dev/nvme",            # NVMe drives
    "/dev/mmcblk",          # MMC/SD cards
    "/dev/vd"               # Virtual disks
]

[categories]
# Custom file categories - see "Supported Categories" section below for defaults
# Format: category_name = [".ext1", ".ext2", ...]
documents = [".doc", ".docx", ".pdf", ".odt", ".rtf", ".txt", ".md"]
images = [".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg", ".webp"]
# ... (27 total categories in default config)
```

### Customizing Configuration

**Add new file extensions to existing categories:**
```toml
[categories]
documents = [".doc", ".docx", ".pdf", ".custom_ext"]
```

**Create custom categories:**
```toml
[categories]
my_custom_category = [".xyz", ".abc"]
```

**Change UI theme:**
```toml
[ui.color]
theme = "cyan"  # Options: default, cyan, magenta, yellow, green, red, blue, white
```

**Adjust performance settings:**
```toml
[export]
max_concurrent_copies = 20  # Increase for faster exports on SSDs

[zip]
compression_level = 9       # Maximum compression
buffer_size_kb = 512        # Larger buffer for better performance
```

**Configure scanning exclusions:**
```toml
[scan]
exclude_patterns = [
    ".*",           # Hidden files
    "node_modules", # Node.js dependencies
    "target",       # Rust build output
    ".git"          # Git repositories
]
```

**Configuration location:** `~/.config/tap/config.toml`

To reset to defaults, delete the configuration file and TAP will recreate it on next run.

## Development

### Running from Source

```bash
# Run with arguments
cargo run -- inspect /dev/sdb1

# Run export command
cargo run -- export -o ./output /mnt/drive

# Run with interactive picker
cargo run -- inspect
```

### Testing and Quality

```bash
# Run test suite
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues
cargo check
```

### Project Structure

```
src/
├── categories.rs      - File type categorization and extension mappings
├── cli.rs             - Command-line argument parsing with clap
├── config.rs          - TOML configuration management
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
