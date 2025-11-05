# 🔍 tap

**As easy as tap-to-pay... for sorting files fast for AI analysis**

`tap` is a blazing-fast file organization tool that categorizes and exports files from drives in seconds. Just like tapping your card to pay, `tap` makes preparing files for AI analysis effortless and instant.

## Why tap?

When you need to feed files to an AI for analysis, the last thing you want is to manually sort through thousands of files. `tap` does the heavy lifting:

- **Instant categorization** - Documents, images, videos, code, databases, and more
- **One command** - No complex setup or configuration needed
- **Lightning fast** - Async scanning with progress bars
- **AI-ready output** - Organized directory structure perfect for LLM ingestion
- **Zero friction** - Interactive device picker when you need it

## Quick Start

### Inspect a drive
```bash
# Let tap find your drives
tap inspect

# Or specify directly
tap inspect /dev/sda
tap inspect /mnt/evidence
```

See exactly what's on a drive, categorized and counted, before you export.

### Export organized files
```bash
# Export and organize files by type
tap export -o ./output

# Create a zip archive ready for AI analysis
tap export -o ./output --zip
```

Files are automatically sorted into categories:
```
output/
├── documents/    # PDFs, Word docs, text files
├── images/       # JPG, PNG, GIF, etc.
├── videos/       # MP4, AVI, MOV, etc.
├── code/         # Python, JavaScript, Rust, etc.
├── databases/    # SQLite, Access, etc.
├── email/        # EML, MSG, PST, etc.
└── misc/         # Everything else
```

## Installation

```bash
cargo install --path .
```

Or build from source:
```bash
git clone https://github.com/yourusername/tap
cd tap
cargo build --release
```

## Use Cases

### AI-Powered Forensics
Need to analyze a drive with an LLM? Export files categorized and zipped, then upload directly to Claude or your preferred AI.

### Data Triage
Quickly scan drives to see what's there before committing to a full export. The inspect mode shows file counts by category.

### Organized Backups
Export files from old drives with automatic categorization. No more digging through messy folder structures.

### Security Research
Catalog and organize files from suspect drives for analysis. Export only what you need.

## Configuration

Customize file categories in `config.toml`:

```toml
[categories.custom_category]
extensions = [".xyz", ".abc"]
description = "My custom file type"
```

## Features

- **Interactive device picker** - Lists available drives when you don't specify one
- **Progress tracking** - Real-time progress bars for scanning and exporting
- **Configurable categories** - Add your own file types via config
- **Parallel processing** - Async file operations for maximum speed
- **Smart defaults** - Works out of the box, customize when needed
- **Error handling** - Graceful failure with helpful error messages

## How It Works

1. **Scan** - Recursively walks the drive to find all files
2. **Categorize** - Matches file extensions against configured categories
3. **Export** - Copies files to organized directories (optional: creates zip)
4. **Done** - Files ready for AI analysis in seconds

## Requirements

- Linux (tested on 6.17.5-zen1-1-zen)
- Rust 1.70+

## License

MIT OR Apache-2.0

---

**tap** - Because preparing files for AI should be as easy as tapping to pay.
