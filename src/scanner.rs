//! File system scanning and analysis.
//!
//! This module provides functionality for scanning directories and categorizing files
//! based on their extensions. It supports parallel processing and progress tracking
//! for efficient analysis of large file systems.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::task;
use walkdir::WalkDir;

use crate::categories::{get_category, get_extension};

/// Information about a scanned file.
///
/// Contains metadata about a file discovered during directory scanning,
/// including its path, size, and categorization.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    /// Size of the file in bytes
    pub size: u64,
    /// The category this file belongs to (e.g., "images", "documents")
    pub category: String,
}

/// Statistics collected during a directory scan.
///
/// Aggregates information about all files discovered during a scan,
/// organized by category, along with error information.
#[derive(Debug)]
pub struct ScanStats {
    pub files_by_category: HashMap<String, Vec<FileInfo>>,
    pub total_files: usize,
    pub total_size: u64,
    pub errors: Vec<String>,
}

impl Default for ScanStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ScanStats {
    /// Creates a new empty `ScanStats` instance.
    pub fn new() -> Self {
        Self {
            files_by_category: HashMap::new(),
            total_files: 0,
            total_size: 0,
            errors: Vec::new(),
        }
    }

    /// Adds a file to the statistics.
    ///
    /// Updates the total file count, total size, and adds the file to its
    /// corresponding category.
    ///
    /// # Arguments
    ///
    /// * `file_info` - Information about the file to add
    pub fn add_file(&mut self, file_info: FileInfo) {
        self.total_files += 1;
        self.total_size += file_info.size;

        self.files_by_category
            .entry(file_info.category.clone())
            .or_default()
            .push(file_info);
    }

    /// Records an error encountered during scanning.
    ///
    /// # Arguments
    ///
    /// * `error` - Description of the error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Generates a summary of files by category.
    ///
    /// Returns a vector of tuples containing category name, file count, and total size.
    /// The results are sorted by file count in descending order.
    ///
    /// # Returns
    ///
    /// A vector of `(category_name, file_count, total_size)` tuples
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

    /// Returns a flat list of all scanned files.
    ///
    /// # Returns
    ///
    /// A vector of `(filename, size, category)` tuples for all files
    pub fn get_all_files(&self) -> Vec<(String, u64, String)> {
        self.files_by_category
            .iter()
            .flat_map(|(category, files)| {
                files.iter().map(move |f| {
                    let name = f.path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();
                    (name, f.size, category.clone())
                })
            })
            .collect()
    }
}

/// Counts the number of files in a directory tree.
///
/// Performs a fast count of all files in the given path, excluding system
/// directories and hidden files. This is useful for displaying progress bars
/// with accurate total counts.
///
/// # Arguments
///
/// * `path` - The root directory to count files in
///
/// # Returns
///
/// The total number of files found, or 0 if an error occurs
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use tap::scanner::count_files;
///
/// # async fn example() {
/// let count = count_files(Path::new("/mnt/evidence")).await;
/// println!("Found {} files", count);
/// # }
/// ```
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

/// Scans a directory and categorizes all files.
///
/// Walks through the directory tree, categorizes each file based on its extension,
/// and collects statistics. System directories and hidden files are automatically excluded.
///
/// # Arguments
///
/// * `path` - The root directory to scan
/// * `progress_callback` - A function called for each file processed, receives the file path as a string
///
/// # Returns
///
/// A `Result` containing `ScanStats` with all collected information
///
/// # Errors
///
/// Returns an error if the directory cannot be accessed or if a critical I/O error occurs.
/// Individual file errors are recorded in the `ScanStats.errors` field.
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use tap::scanner::scan_directory;
///
/// # async fn example() -> color_eyre::Result<()> {
/// let stats = scan_directory(Path::new("/mnt/evidence"), |path| {
///     println!("Processing: {}", path);
/// }).await?;
///
/// println!("Total files: {}", stats.total_files);
/// println!("Total size: {} bytes", stats.total_size);
/// # Ok(())
/// # }
/// ```
pub async fn scan_directory<F>(path: &Path, progress_callback: F) -> color_eyre::Result<ScanStats>
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
                            let mut stats = stats_clone.lock().unwrap();
                            stats.add_file(file_info);
                        }
                        Err(e) => {
                            let mut stats = stats_clone.lock().unwrap();
                            stats.add_error(format!("Error reading {}: {}", path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    let mut stats = stats_clone.lock().unwrap();
                    stats.add_error(format!("Error walking directory: {}", e));
                }
                _ => {}
            }
        }
    })
    .await?;

    let stats = Arc::try_unwrap(stats)
        .map_err(|_| color_eyre::eyre::eyre!("Failed to unwrap stats"))?
        .into_inner()?;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_info_creation() {
        let file_info = FileInfo {
            path: PathBuf::from("/test/file.txt"),
            size: 1024,
            category: "documents".to_string(),
        };

        assert_eq!(file_info.path, PathBuf::from("/test/file.txt"));
        assert_eq!(file_info.size, 1024);
        assert_eq!(file_info.category, "documents");
    }

    #[test]
    fn test_scan_stats_new() {
        let stats = ScanStats::new();

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_size, 0);
        assert!(stats.files_by_category.is_empty());
        assert!(stats.errors.is_empty());
    }

    #[test]
    fn test_scan_stats_add_file() {
        let mut stats = ScanStats::new();

        let file_info = FileInfo {
            path: PathBuf::from("/test/file.txt"),
            size: 1024,
            category: "documents".to_string(),
        };

        stats.add_file(file_info);

        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_size, 1024);
        assert!(stats.files_by_category.contains_key("documents"));
        assert_eq!(stats.files_by_category["documents"].len(), 1);
    }

    #[test]
    fn test_scan_stats_add_multiple_files() {
        let mut stats = ScanStats::new();

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/file1.txt"),
            size: 1024,
            category: "documents".to_string(),
        });

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/file2.jpg"),
            size: 2048,
            category: "images".to_string(),
        });

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/file3.txt"),
            size: 512,
            category: "documents".to_string(),
        });

        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.total_size, 1024 + 2048 + 512);
        assert_eq!(stats.files_by_category["documents"].len(), 2);
        assert_eq!(stats.files_by_category["images"].len(), 1);
    }

    #[test]
    fn test_scan_stats_add_error() {
        let mut stats = ScanStats::new();

        stats.add_error("Test error".to_string());
        stats.add_error("Another error".to_string());

        assert_eq!(stats.errors.len(), 2);
        assert_eq!(stats.errors[0], "Test error");
        assert_eq!(stats.errors[1], "Another error");
    }

    #[test]
    fn test_scan_stats_get_summary() {
        let mut stats = ScanStats::new();

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/file1.txt"),
            size: 1024,
            category: "documents".to_string(),
        });

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/file2.txt"),
            size: 512,
            category: "documents".to_string(),
        });

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/file3.jpg"),
            size: 2048,
            category: "images".to_string(),
        });

        let summary = stats.get_summary();

        // Should be sorted by count descending
        assert_eq!(summary.len(), 2);

        // Documents has 2 files
        let docs = summary.iter().find(|(cat, _, _)| cat == "documents").unwrap();
        assert_eq!(docs.1, 2);
        assert_eq!(docs.2, 1024 + 512);

        // Images has 1 file
        let images = summary.iter().find(|(cat, _, _)| cat == "images").unwrap();
        assert_eq!(images.1, 1);
        assert_eq!(images.2, 2048);
    }

    #[test]
    fn test_scan_stats_get_all_files() {
        let mut stats = ScanStats::new();

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/document.txt"),
            size: 1024,
            category: "documents".to_string(),
        });

        stats.add_file(FileInfo {
            path: PathBuf::from("/test/image.jpg"),
            size: 2048,
            category: "images".to_string(),
        });

        let all_files = stats.get_all_files();

        assert_eq!(all_files.len(), 2);

        // Check that filenames are extracted correctly
        let has_document = all_files.iter().any(|(name, _, _)| name == "document.txt");
        let has_image = all_files.iter().any(|(name, _, _)| name == "image.jpg");

        assert!(has_document);
        assert!(has_image);
    }
}
