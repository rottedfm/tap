// src/main.rs
mod categories;
mod cli;
mod export;
mod inspect;
mod log;
mod mount;
mod scanner;
mod tui;
mod zip;

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
        } => {
            handle_export(&drive, &output_dir).await?;
        }
    }

    Ok(())
}
