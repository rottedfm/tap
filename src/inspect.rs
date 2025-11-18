//! Drive inspection workflow.
//!
//! This module implements the inspect command, which mounts a drive, scans
//! its contents, and displays categorized file statistics.

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::log::write_inspect_log;
use crate::mount::{mount_drive_readonly, unmount_drive, validate_source_path};
use crate::scanner::{count_files, scan_directory};
use crate::tui::{Mode, UI};

pub async fn handle_inspect(
    drive: &str,
    write_log: bool,
    config: &Config,
) -> color_eyre::Result<()> {
    // Check if it's a device or a path
    let is_device = drive.starts_with("/dev/");
    let source_path = if is_device {
        mount_drive_readonly(drive, &config.ui.color.theme).await?
    } else {
        validate_source_path(drive, &config.ui.color.theme)?
    };

    // Create UI with color theme from config
    let ui = UI::new()?.with_color_theme(config.ui.color.theme.clone());
    let inspect_msg = format!("Source: {}", source_path.display());
    ui.init(&Mode::Inspect, &inspect_msg)?;

    // Phase 1: Count files
    ui.print_info("Phase 1/2: Counting filesystem entries")?;
    let spinner = ui.create_spinner("Walking directory tree...");

    let total_files = count_files(&source_path).await;

    spinner.finish_and_clear();
    ui.print_success(&format!("Discovered {} files", total_files))?;

    // Phase 2: Scan and categorize
    ui.print_info("Phase 2/2: Analyzing and categorizing files")?;

    // Draw the recent files section first, then create progress bar below it
    ui.draw_recent_files()?;
    let pb = ui.create_progress_bar(total_files, "Analyzing");

    let ui_arc = Arc::new(Mutex::new(ui));
    let counter = Arc::new(Mutex::new(0u64));

    let scan_stats = scan_directory(&source_path, {
        let pb = pb.clone();
        let ui_arc = Arc::clone(&ui_arc);
        let counter = Arc::clone(&counter);

        move |path| {
            pb.inc(1);

            // Rate limit UI updates to prevent screen overflow
            // Only update every 100 files
            // Use try_lock to avoid blocking in the scanning thread
            if let Ok(mut count) = counter.try_lock() {
                *count += 1;

                if *count % 100 == 0 {
                    if let Ok(mut ui) = ui_arc.try_lock() {
                        let _ = ui.update_recent_files(path);
                    }
                }
            }
        }
    })
    .await?;

    pb.finish_and_clear();

    // Get UI back
    let ui = Arc::try_unwrap(ui_arc)
        .map_err(|_| color_eyre::eyre::eyre!("Failed to unwrap UI"))?
        .into_inner();

    // Wait for user to see final scan files
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Clear the recent files section after scan completes
    ui.term.clear_last_lines(ui.max_recent + 2)?;

    // Clear screen and show clean output
    ui.term.clear_screen()?;

    // Show banner with mode again for context
    ui.print_banner_with_mode(&Mode::Inspect)?;

    // Display scan results
    let summary = scan_stats.get_summary();
    let all_files = scan_stats.get_all_files();
    ui.print_summary(
        &Mode::Inspect,
        "INSPECTION COMPLETE",
        &summary,
        &all_files,
        None,
        false,
    )?;

    // Clear screen for final messages
    ui.term.clear_screen()?;
    ui.print_banner_with_mode(&Mode::Inspect)?;
    println!();

    if !scan_stats.errors.is_empty() {
        ui.print_warning(&format!(
            "{} file(s) skipped due to permission errors or I/O failures",
            scan_stats.errors.len()
        ))?;
        println!();
    }

    ui.print_success("Inspection complete")?;
    println!();

    // Write log file if requested
    if write_log {
        ui.print_info("Writing log file...")?;
        match write_inspect_log(&source_path, &scan_stats).await {
            Ok(log_path) => {
                ui.print_success(&format!("Log written to: {}", log_path.display()))?;
                println!();
            }
            Err(e) => {
                ui.print_warning(&format!("Failed to write log file: {}", e))?;
                println!();
            }
        }
    }

    ui.cleanup()?;

    // Unmount drive if we mounted it
    if is_device {
        unmount_drive(&source_path, drive, &config.ui.color.theme)?;
    }

    Ok(())
}
