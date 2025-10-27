// src/main.rs
mod categories;
mod cli;
mod device_picker;
mod export;
mod inspect;
mod log;
mod mount;
mod scanner;
mod tui;
mod zip;

use clap::Parser;

use cli::{Args, Commands};
use device_picker::pick_device;
use export::handle_export;
use inspect::handle_inspect;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    match args.command {
        Commands::Inspect { drive } => {
            let drive_path = match drive {
                Some(d) => d,
                None => pick_device()?,
            };
            handle_inspect(&drive_path).await?;
        }
        Commands::Export {
            drive,
            output_dir,
        } => {
            let drive_path = match drive {
                Some(d) => d,
                None => pick_device()?,
            };
            handle_export(&drive_path, &output_dir).await?;
        }
    }

    Ok(())
}
