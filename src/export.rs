// src/export.rs
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::exporter::export_files;
use crate::mount::{mount_drive_readonly, unmount_drive, validate_source_path};
use crate::scanner::{count_files, scan_directory};
use crate::tui::{format_size, Mode, UI};

pub async fn handle_export(
    drive: &str,
    output_dir: &Path,
    dry_run: bool,
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

    let mode_message = if dry_run {
        format!(
            "DRY RUN: {} → {}",
            source_path.display(),
            output_dir.display()
        )
    } else {
        format!(
            "Exporting: {} → {}",
            source_path.display(),
            output_dir.display()
        )
    };

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
    if dry_run {
        ui.print_info("DRY RUN: No files will be copied")?;
        ui.print_info(&format!(
            "Would copy {} files to {}",
            scan_stats.total_files,
            output_dir.display()
        ))?;
    } else {
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
    }

    ui.cleanup()?;

    // Unmount drive if we mounted it
    if is_device {
        println!();
        unmount_drive(&source_path, drive)?;
    }

    Ok(())
}

// TODO: move to zip.rs
async fn zip_directory(source_dir: &Path) -> color_eyre::Result<PathBuf> {
    println!("{} Creating zip archive...", style("ℹ️").cyan());

    // Create zip file path
    let zip_path = source_dir.with_extension("zip");
    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Walk through the directory
    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.strip_prefix(source_dir)?;

        if path.is_file() {
            zip.start_file(name.to_string_lossy().to_string(), options)?;
            let mut f = File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        } else if !name.as_os_str().is_empty() {
            // Add directory entry
            zip.add_directory(name.to_string_lossy().to_string(), options)?;
        }
    }

    zip.finish()?;

    println!(
        "{} Archive created: {}",
        style("✓").green(),
        style(zip_path.display()).bold()
    );

    Ok(zip_path)
}

// TODO: move to log.rs
async fn write_log_file(
    dest: &Path,
    scan_stats: &crate::scanner::ScanStats,
    export_stats: &crate::exporter::ExportStats,
) -> color_eyre::Result<()> {
    let log_path = dest.join("tap.log");
    let mut file = tokio::fs::File::create(&log_path).await?;

    let mut content = String::new();
    content.push_str("TAP LOG\n");
    content.push_str(&"═".repeat(70));
    content.push_str("\n\n");

    content.push_str(&format!(
        "Total files scanned: {}\n",
        scan_stats.total_files
    ));
    content.push_str(&format!(
        "Total size: {}\n\n",
        format_size(scan_stats.total_size)
    ));

    content.push_str("FILES BY CATEGORY\n");
    content.push_str(&"─".repeat(70));
    content.push('\n');

    for (category, count, size) in scan_stats.get_summary() {
        content.push_str(&format!(
            "{}: {} files ({})\n",
            category,
            count,
            format_size(size)
        ));
    }

    content.push('\n');
    content.push_str(&format!("Files copied: {}\n", export_stats.copied));
    content.push_str(&format!("Files failed: {}\n", export_stats.failed));

    if !scan_stats.errors.is_empty() {
        content.push_str("\nSCAN ERRORS\n");
        content.push_str(&"─".repeat(70));
        content.push('\n');
        for error in &scan_stats.errors {
            content.push_str(&format!("{}\n", error));
        }
    }

    if !export_stats.errors.is_empty() {
        content.push_str("\nEXPORT ERRORS\n");
        content.push_str(&"─".repeat(70));
        content.push('\n');
        for error in &export_stats.errors {
            content.push_str(&format!("{}\n", error));
        }
    }

    file.write_all(content.as_bytes()).await?;
    Ok(())
}
