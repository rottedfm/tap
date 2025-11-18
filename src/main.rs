// src/main.rs
mod categories;
mod cli;
mod config;
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
use config::Config;
use device_picker::pick_device;
use export::handle_export;
use inspect::handle_inspect;
use tui::{Mode, UI};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Load configuration
    let config = Config::load()?;

    let args = Args::parse();

    match args.command {
        Commands::Inspect { drive, log } => {
            // Check terminal size before device picker
            UI::check_terminal_size(&Mode::Inspect, &config.ui.color.theme)?;

            let drive_path = match drive {
                Some(d) => d,
                None => pick_device(&config.ui.color.theme)?,
            };
            handle_inspect(&drive_path, log, &config).await?;
        }
        Commands::Export {
            drive,
            output_dir,
            zip,
        } => {
            // Check terminal size before device picker
            UI::check_terminal_size(&Mode::Export, &config.ui.color.theme)?;

            let drive_path = match drive {
                Some(d) => d,
                None => pick_device(&config.ui.color.theme)?,
            };
            handle_export(&drive_path, &output_dir, zip, &config).await?;
        }
    }

    Ok(())
}
