// src/inspect.rs
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::mount::{mount_drive_readonly, unmount_drive, validate_source_path};
use crate::scanner::{count_files, scan_directory};
use crate::tui::{Mode, UI};

pub async fn handle_inspect(drive: &str) -> color_eyre::Result<()> {
    // Check if it's a device or a path
    let is_device = drive.starts_with("/dev/");
    let source_path = if is_device {
        mount_drive_readonly(drive).await?
    } else {
        validate_source_path(drive)?
    };

    // Create UI
    let ui = UI::new()?;
    let inspect_msg = format!("Inspecting: {}", source_path.display());
    ui.init(&Mode::Inspect, &inspect_msg)?;

    // Phase 1: Count files
    ui.print_info("PHASE 1: Counting files...")?;
    let spinner = ui.create_spinner("Scanning drive...");

    let total_files = count_files(&source_path).await;

    spinner.finish_and_clear();
    ui.print_success(&format!("Found {} files", total_files))?;

    // Phase 2: Scan and categorize
    ui.print_info("PHASE 2: Analyzing files...")?;

    // Draw the recent files section first, then create progress bar below it
    ui.draw_recent_files()?;
    let pb = ui.create_progress_bar(total_files, "Scanning");

    let ui_arc = Arc::new(Mutex::new(ui));

    let scan_stats = scan_directory(&source_path, {
        let pb = pb.clone();
        let ui_arc = Arc::clone(&ui_arc);

        move |path| {
            pb.inc(1);

            // Update on every file for realtime display
            let mut ui = futures::executor::block_on(ui_arc.lock());
            let _ = ui.update_recent_files(path);
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
    ui.print_summary("INSPECTION COMPLETE", &summary, false)?;

    if !scan_stats.errors.is_empty() {
        println!();
        ui.print_warning(&format!(
            "{} errors occurred during scan",
            scan_stats.errors.len()
        ))?;
    }

    ui.cleanup()?;

    // Unmount drive if we mounted it
    if is_device {
        println!();
        unmount_drive(&source_path, drive)?;
    }

    Ok(())
}
