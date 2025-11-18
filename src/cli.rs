//! Command-line interface definitions.
//!
//! This module defines the CLI structure using clap, including all commands
//! and their arguments.

use crate::tui::BANNER;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tap")]
#[command(about = "File investigation and export tool for mountable drives")]
#[command(before_help = BANNER)]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inspect a drive and catalog its contents
    Inspect {
        /// Drive or path to inspect (e.g, /dev/sda or /mnt/evidence)
        drive: Option<String>,

        /// Write a text log file summarizing the inspection results
        #[arg(long)]
        log: bool,
    },
    /// Export files from a drive organized by type
    Export {
        /// Drive or path to export from (e.g, /dev/sda or /mnt/evidence)
        drive: Option<String>,

        /// Output directory for organized files
        #[arg(short, long)]
        output_dir: PathBuf,

        /// Create a zip archive of the exported files
        #[arg(long)]
        zip: bool,
    },
    // TODO: Discover -- find eleigables and output what is most likely data not boot partitions
}
