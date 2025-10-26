// src/inspect.rs
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::mount::{mount_drive_readonly, unmount_drive, validate_source_path};
use crate::scanner::{count_files, scan_directory};
use crate::tui::{format_size, Mode, UI};

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
    ui.print_info("Phase 1: Counting files...")?;
    let spinner = ui.create_spinner("Scanning drive...");

    let total_files = count_files(&source_path).await;

    spinner.finish_and_clear();
    ui.print_success(&format!("Found {} files", total_files))?;

    // Phase 2: Scan and categorize
    ui.print_info("Phase 2: Analyzing files...")?;

    let pb = ui.create_progress_bar(total_files, "Scanning");
    ui.draw_recent_files()?;

    let update_counter = Arc::new(Mutex::new(0u64));
    let ui_arc = Arc::new(Mutex::new(ui));

    let scan_stats = scan_directory(&source_path, {
        let pb = pb.clone();
        let update_counter = Arc::clone(&update_counter);
        let ui_arc = Arc::clone(&ui_arc);

        move |path| {
            pb.inc(1);

            let mut counter = futures::executor::block_on(update_counter.lock());
            *counter += 1;

            // Update UI every 5 files to reduce flicker
            if (*counter).is_multiple_of(5) {
                let mut ui = futures::executor::block_on(ui_arc.lock());
                let _ = ui.update_recent_files(path);
            }
        }
    })
    .await?;

    pb.finish_and_clear();

    // Get UI back
    let ui = Arc::try_unwrap(ui_arc)
        .map_err(|_| color_eyre::eyre::eyre!("Failed to unwrap UI"))?
        .into_inner();

    // Display scan results
    let summary = scan_stats.get_summary();
    ui.print_summary("INSPECTION COMPLETE", &summary)?;

    if !scan_stats.errors.is_empty() {
        ui.print_warning(&format!(
            "{} errors occurred during scan",
            scan_stats.errors.len()
        ))?;
    }

    ui.print_info(&format!("Total files: {}", scan_stats.total_files))?;
    ui.print_info(&format!(
        "Total size: {}",
        format_size(scan_stats.total_size)
    ))?;

    ui.cleanup()?;

    // Unmount drive if we mounted it
    if is_device {
        println!();
        unmount_drive(&source_path, drive)?;
    }

    Ok(())
}
