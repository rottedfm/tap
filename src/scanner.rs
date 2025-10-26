// scanner.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use walkdir::WalkDir;

use crate::categories::{get_category, get_extension};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub category: String,
}

#[derive(Debug)]
pub struct ScanStats {
    pub files_by_category: HashMap<String, Vec<FileInfo>>,
    pub total_files: usize,
    pub total_size: u64,
    pub errors: Vec<String>,
}

impl ScanStats {
    pub fn new() -> Self {
        Self {
            files_by_category: HashMap::new(),
            total_files: 0,
            total_size: 0,
            errors: Vec::new(),
        }
    }

    pub fn add_file(&mut self, file_info: FileInfo) {
        self.total_files += 1;
        self.total_size += file_info.size;

        self.files_by_category
            .entry(file_info.category.clone())
            .or_default()
            .push(file_info);
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn get_summary(&self) -> Vec<(String, usize, u64)> {
        let mut summary: Vec<_> = self
            .files_by_category
            .iter()
            .map(|(category, files)| {
                let count = files.len();
                let size: u64 = files.iter().map(|f| f.size).sum();
                (category.clone(), count, size)
            })
            .collect();

        summary.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        summary
    }
}

pub async fn count_files(path: &Path) -> u64 {
    let result: Result<u64, tokio::task::JoinError> = task::spawn_blocking({
        let path = path.to_path_buf();
        move || -> u64 {
            WalkDir::new(&path)
                .into_iter()
                .filter_entry(|e| {
                    let file_name = e.file_name().to_string_lossy();
                    !file_name.starts_with('.')
                        && file_name != "System Volume Information"
                        && file_name != "$RECYCLE.BIN"
                        && file_name != "node_modules"
                })
                .filter_map(|e: Result<walkdir::DirEntry, walkdir::Error>| e.ok())
                .filter(|e| e.file_type().is_file())
                .count() as u64
        }
    })
    .await;

    result.unwrap_or(0)
}

pub async fn scan_directory<F>(
    path: &Path,
    progress_callback: F,
) -> color_eyre::Result<ScanStats>
where
    F: Fn(String) + Send + Sync + 'static,
{
    let stats = Arc::new(Mutex::new(ScanStats::new()));
    let callback = Arc::new(progress_callback);

    let path = path.to_path_buf();
    let stats_clone = Arc::clone(&stats);
    let callback_clone = Arc::clone(&callback);

    task::spawn_blocking(move || {
        for entry in WalkDir::new(&path).into_iter().filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            !file_name.starts_with('.')
                && file_name != "System Volume Information"
                && file_name != "$RECYCLE.BIN"
                && file_name != "node_modules"
        }) {
            match entry {
                Ok(entry) if entry.file_type().is_file() => {
                    let path = entry.path();
                    let extension = get_extension(path);
                    let category = get_category(&extension);

                    match std::fs::metadata(path) {
                        Ok(metadata) => {
                            let file_info = FileInfo {
                                path: path.to_path_buf(),
                                size: metadata.len(),
                                category: category.to_string(),
                            };

                            // Callback with current file
                            callback_clone(path.display().to_string());

                            // add to stats
                            let mut stats = futures::executor::block_on(stats_clone.lock());
                            stats.add_file(file_info);
                        }
                        Err(e) => {
                            let mut stats = futures::executor::block_on(stats_clone.lock());
                            stats.add_error(format!("Error reading {}: {}", path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    let mut stats = futures::executor::block_on(stats_clone.lock());
                    stats.add_error(format!("Error walking directory: {}", e));
                }
                _ => {}
            }
        }
    })
    .await?;

    let stats = Arc::try_unwrap(stats)
        .map_err(|_| color_eyre::eyre::eyre!("Failed to unwrap stats"))?
        .into_inner();

    Ok(stats)
}
