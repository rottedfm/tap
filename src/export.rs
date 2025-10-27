// src/export.rs
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::log::write_log_file;
use crate::mount::{mount_drive_readonly, unmount_drive, validate_source_path};
use crate::scanner::{count_files, scan_directory, ScanStats};
use crate::tui::{Mode, UI};
use crate::zip::zip_directory;

const MAX_CONCURRENT_COPIES: usize = 10;

pub struct ExportStats {
    pub copied: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

impl ExportStats {
    pub fn new() -> Self {
        Self {
            copied: 0,
            failed: 0,
            errors: Vec::new(),
        }
    }
}

async fn copy_file_with_rename(
    src: &Path,
    dest_dir: &Path,
    filename: &str,
) -> color_eyre::Result<PathBuf> {
    let mut dest_path = dest_dir.join(filename);

    // Handle duplicate filenames
    if dest_path.exists() {
        let stem = Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let mut counter = 1;
        loop {
            let new_filename = if extension.is_empty() {
                format!("{}_{}", stem, counter)
            } else {
                format!("{}_{}.{}", stem, counter, extension)
            };

            dest_path = dest_dir.join(new_filename);

            if !dest_path.exists() {
                break;
            }
            counter += 1;
        }
    }
    fs::copy(src, &dest_path).await?;
    Ok(dest_path)
}

pub async fn export_files<F>(
    scan_stats: &ScanStats,
    dest_base: &Path,
    progress_callback: F,
) -> color_eyre::Result<ExportStats>
where
    F: Fn(String) + Send + Sync + 'static,
{
    let export_stats = Arc::new(Mutex::new(ExportStats::new()));
    let callback = Arc::new(progress_callback);

    // Create base destination directiory
    fs::create_dir_all(dest_base).await?;

    // Create category directory
    for category in scan_stats.files_by_category.keys() {
        let category_dir = dest_base.join(category);
        fs::create_dir_all(&category_dir).await?;
    }

    // Collect all files to copy
    let all_files: Vec<_> = scan_stats
        .files_by_category
        .iter()
        .flat_map(|(category, files)| {
            files
                .iter()
                .map(move |file| (category.clone(), file.clone()))
        })
        .collect();

    // Copy files concurrently with limited parallelism
    stream::iter(all_files)
        .map(|(category, file_info)| {
            let dest_base = dest_base.to_path_buf();
            let export_stats = Arc::clone(&export_stats);
            let callback = Arc::clone(&callback);

            async move {
                let category_dir = dest_base.join(&category);
                let filename = file_info
                    .path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                callback(file_info.path.display().to_string());

                match copy_file_with_rename(&file_info.path, &category_dir, filename).await {
                    Ok(_) => {
                        let mut stats = export_stats.lock().await;
                        stats.copied += 1;
                    }
                    Err(e) => {
                        let mut stats = export_stats.lock().await;
                        stats.failed += 1;
                        stats.errors.push(format!(
                            "Failed to copy {}: {}",
                            file_info.path.display(),
                            e
                        ));
                    }
                }
            }
        })
        .buffer_unordered(MAX_CONCURRENT_COPIES)
        .collect::<Vec<_>>()
        .await;

    let export_stats = Arc::try_unwrap(export_stats)
        .map_err(|_| color_eyre::eyre::eyre!("Failed to unwrap export stats"))?
        .into_inner();

    Ok(export_stats)
}

pub async fn handle_export(
    drive: &str,
    output_dir: &Path,
) -> color_eyre::Result<()> {
    // Check if output directory already exists
    if output_dir.exists() {
        println!(
            "{} Output directory already exists: {}",
            style("⚠️").yellow(),
            style(output_dir.display()).bold()
        );

        let should_continue = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Overwrite/merge?")
            .default(false)
            .interact()?;

        if !should_continue {
            println!("Aborted.");
            std::process::exit(0);
        }
    }

    // Check if it's a device or a path
    let is_device = drive.starts_with("/dev/");
    let source_path = if is_device {
        mount_drive_readonly(drive).await?
    } else {
        validate_source_path(drive)?
    };

    // Create UI
    let ui = UI::new()?;

    let mode_message = format!(
        "Exporting: {} → {}",
        source_path.display(),
        output_dir.display()
    );

    ui.init(&Mode::Export, &mode_message)?;

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

            if (*counter).is_multiple_of(5) {
                let mut ui = futures::executor::block_on(ui_arc.lock());
                let _ = ui.update_recent_files(path);
            }
        }
    })
    .await?;

    pb.finish_and_clear();

    // Get UI back
    let mut ui = Arc::try_unwrap(ui_arc)
        .map_err(|_| color_eyre::eyre::eyre!("Failed to unwrap UI"))?
        .into_inner();

    // Display scan results
    let summary = scan_stats.get_summary();
    ui.print_summary("SCAN RESULTS", &summary)?;

    if !scan_stats.errors.is_empty() {
        ui.print_warning(&format!(
            "{} errors occurred during scan",
            scan_stats.errors.len()
        ))?;
    }

    // Phase 3: Export
    ui.print_info("Phase 3: Copying files...")?;

    let pb = ui.create_progress_bar(scan_stats.total_files as u64, "Copying");
    ui.draw_recent_files()?;

    let update_counter = Arc::new(Mutex::new(0u64));
    let ui_arc = Arc::new(Mutex::new(ui));

    let export_stats = export_files(&scan_stats, output_dir, {
        let pb = pb.clone();
        let update_counter = Arc::clone(&update_counter);
        let ui_arc = Arc::clone(&ui_arc);

        move |path| {
            pb.inc(1);

            let mut counter = futures::executor::block_on(update_counter.lock());
            *counter += 1;

            if (*counter).is_multiple_of(5) {
                let mut ui = futures::executor::block_on(ui_arc.lock());
                let _ = ui.update_recent_files(path);
            }
        }
    })
    .await?;

    pb.finish_and_clear();

    // Get UI back
    ui = Arc::try_unwrap(ui_arc)
        .map_err(|_| color_eyre::eyre::eyre!("Failed to unwrap UI"))?
        .into_inner();

    // Clear the recent files section
    ui.term.clear_last_lines(ui.max_recent + 2)?;

    // Display export results
    println!();
    ui.print_success(&format!(
        "Successfully copied {} files",
        export_stats.copied
    ))?;

    if export_stats.failed > 0 {
        ui.print_error(&format!("Failed to copy {} files", export_stats.failed))?;
    }

    if !export_stats.errors.is_empty() {
        ui.print_warning("Check log file for error details")?;
    }

    // Write log file
    write_log_file(output_dir, &scan_stats, &export_stats).await?;
    let log_path = output_dir.join("tap.log");
    ui.print_info(&format!("Log written to: {}", log_path.display()))?;
    println!();

    // Zip the exported directory
    ui.print_info("Phase 4: Creating archive...")?;
    let zip_path = zip_directory(output_dir).await?;
    ui.print_success(&format!("Archive created: {}", zip_path.display()))?;

    // Remove the original directory
    println!("{} Removing temporary directory...", style("ℹ️").cyan());
    tokio::fs::remove_dir_all(output_dir).await?;
    println!("{} Temporary directory removed", style("✓").green());

    ui.cleanup()?;

    // Unmount drive if we mounted it
    if is_device {
        println!();
        unmount_drive(&source_path, drive)?;
    }

    Ok(())
}
