//! Archive creation utilities.
//!
//! This module provides functionality for creating ZIP archives from directories,
//! with progress tracking and optimized compression settings.

use indicatif::ProgressBar;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::task;
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::FileOptions;

pub async fn zip_directory<F>(
    source_dir: &Path,
    pb: ProgressBar,
    progress_callback: F,
) -> color_eyre::Result<PathBuf>
where
    F: Fn(String) + Send + Sync + 'static,
{
    let source_dir = source_dir.to_path_buf();
    let pb = Arc::new(pb);
    let progress_callback = Arc::new(progress_callback);

    // Run the blocking zip operation in a separate thread pool
    let zip_path = task::spawn_blocking(move || -> color_eyre::Result<PathBuf> {
        // Create zip file path
        let zip_path = source_dir.with_extension("zip");
        let file = File::create(&zip_path)?;
        let file = BufWriter::with_capacity(256 * 1024, file); // 256KB buffer
        let mut zip = ZipWriter::new(file);

        // Use faster compression with level 6 (good balance of speed/compression)
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6))
            .unix_permissions(0o755);

        // Walk through the directory
        for entry in WalkDir::new(&source_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let name = path.strip_prefix(&source_dir)?;

            if path.is_file() {
                // Call callback with file path
                progress_callback(path.display().to_string());

                zip.start_file(name.to_string_lossy().to_string(), options)?;

                // Use buffered reader for better I/O performance
                let f = File::open(path)?;
                let mut f = BufReader::with_capacity(128 * 1024, f); // 128KB buffer
                std::io::copy(&mut f, &mut zip)?;

                // Update progress
                pb.inc(1);
            } else if !name.as_os_str().is_empty() {
                // Add directory entry
                zip.add_directory(name.to_string_lossy().to_string(), options)?;
            }
        }

        zip.finish()?;
        pb.finish_and_clear();

        Ok(zip_path)
    })
    .await??;

    Ok(zip_path)
}
