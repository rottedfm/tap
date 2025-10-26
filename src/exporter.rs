// exporter.rs
use futures::stream::{self, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

use crate::scanner::ScanStats;

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
