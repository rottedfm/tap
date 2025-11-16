//! Log file generation.
//!
//! This module creates detailed log files documenting scan and export operations,
//! including statistics, errors, and file categorization summaries.

use std::path::Path;
use tokio::io::AsyncWriteExt;

use crate::export::ExportStats;
use crate::scanner::ScanStats;
use crate::tui::format_size;

pub async fn write_log_file(
    dest: &Path,
    scan_stats: &ScanStats,
    export_stats: &ExportStats,
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
