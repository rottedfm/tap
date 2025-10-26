// src/main.rs
mod categories;
mod cli;
mod export;
mod exporter;
mod inspect;
mod mount;
mod scanner;
mod tui;

use clap::Parser;

use cli::{Args, Commands};
use export::handle_export;
use inspect::handle_inspect;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    match args.command {
        Commands::Inspect { drive } => {
            handle_inspect(&drive).await?;
        }
        Commands::Export {
            drive,
            output_dir,
            dry_run,
        } => {
            handle_export(&drive, &output_dir, dry_run).await?;
        }
    }

    Ok(())
}
