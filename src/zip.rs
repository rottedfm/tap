// src/zip.rs
use console::style;
use std::fs::File;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;
use zip::ZipWriter;

pub async fn zip_directory(source_dir: &Path) -> color_eyre::Result<PathBuf> {
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
