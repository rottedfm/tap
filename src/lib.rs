//! # TAP - File Investigation and Export Tool
//!
//! TAP is a comprehensive file investigation and export tool designed for organizing and
//! categorizing files from drives or directories for LLM analysis. It automatically sorts
//! files into meaningful categories, making it easy to prepare datasets for AI processing,
//! content analysis, or data organization tasks.
//!
//! ## Features
//!
//! - **Intelligent File Categorization**: Automatically categorizes files into 20+ categories
//!   including documents, images, videos, databases, code, and more - perfect for LLM analysis
//! - **Read-Only Mounting**: Safely mount and inspect drives in read-only mode to preserve data
//! - **Parallel Processing**: Concurrent file operations for maximum performance
//! - **Rich Terminal UI**: Beautiful, themed terminal interface with progress tracking
//! - **Export & Archive**: Export categorized files and optionally compress to ZIP archives
//! - **Comprehensive Logging**: Detailed logs of all operations and errors
//!
//! ## Command Line Usage
//!
//! ### Inspect a Drive
//!
//! ```bash
//! # Interactive device selection
//! tap inspect
//!
//! # Inspect specific device
//! tap inspect /dev/sda1
//!
//! # Inspect mounted path
//! tap inspect /mnt/evidence
//! ```
//!
//! ### Export Files
//!
//! ```bash
//! # Export with interactive device selection
//! tap export --output-dir ./extracted
//!
//! # Export from specific device
//! tap export /dev/sda1 --output-dir ./extracted
//!
//! # Export and create ZIP archive
//! tap export /dev/sda1 --output-dir ./extracted --zip
//! ```
//!
//! ## Library Usage
//!
//! TAP can also be used as a library for building custom file investigation tools:
//!
//! ```rust,no_run
//! use tap::scanner::{scan_directory, ScanStats};
//! use tap::config::Config;
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> color_eyre::Result<()> {
//!     let config = Config::load()?;
//!     let path = Path::new("/mnt/evidence");
//!
//!     let stats = scan_directory(path, |file_path| {
//!         println!("Scanning: {}", file_path);
//!     }).await?;
//!
//!     println!("Found {} files", stats.total_files);
//!     println!("Total size: {} bytes", stats.total_size);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Configuration
//!
//! TAP uses a TOML configuration file located at `~/.config/tap/config.toml`.
//! On first run, a default configuration is created automatically.
//!
//! ### Configuration Options
//!
//! - **Categories**: File extension mappings for categorization
//! - **Export Settings**: Concurrent copy limits
//! - **ZIP Settings**: Compression level and buffer sizes
//! - **UI Settings**: Color themes and display options
//! - **Scan Settings**: Exclusion patterns for directories
//! - **Mount Settings**: Device patterns and mount locations
//!
//! ## Module Organization
//!
//! - [`categories`]: File categorization and extension mapping
//! - [`cli`]: Command-line argument parsing
//! - [`config`]: Configuration management
//! - [`device_picker`]: Interactive device selection
//! - [`export`]: File export and copy operations
//! - [`inspect`]: Drive inspection workflows
//! - [`log`]: Log file generation
//! - [`mount`]: Drive mounting and validation
//! - [`scanner`]: File system scanning and analysis
//! - [`tui`]: Terminal user interface components
//! - [`zip`]: Archive creation utilities

pub mod categories;
pub mod cli;
pub mod config;
pub mod device_picker;
pub mod export;
pub mod inspect;
pub mod log;
pub mod mount;
pub mod scanner;
pub mod tui;
pub mod zip;

// Re-export commonly used types
pub use config::Config;
pub use export::ExportStats;
pub use scanner::{FileInfo, ScanStats};
