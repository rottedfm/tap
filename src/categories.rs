// categories.rs
use std::collections::HashMap;
use std::path::Path;

// TODO: Get these file types from a toml config
pub fn get_categories() -> HashMap<&'static str, Vec<&'static str>> {
    let mut categories = HashMap::new();

    categories.insert(
        "documents",
        vec![".doc", ".docx", ".pdf", ".obt", ".rtf", ".txt", ".md"],
    );

    categories.insert("spreadsheets", vec![".xls", ".xlsx", ".ods", ".csv"]);

    categories.insert(
        "images",
        vec![
            ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".tiff", ".tif", ".svg", ".heic", ".webp",
            ".ico",
        ],
    );

    categories.insert(
        "videos",
        vec![
            ".mp4", ".avi", ".mov", ".mkv", ".wmv", ".flv", ".webm", ".m4v", ".mpg", ".mpeg",
        ],
    );

    categories.insert(
        "audio",
        vec![".mp3", ".wav", ".flac", ".aac", ".ogg", ".m4a", ".wma"],
    );

    categories.insert(
        "archives",
        vec![".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz"],
    );

    categories.insert("email", vec![".eml", ".msg", ".pst", ".ost", ".mbox"]);

    categories.insert(
        "databases",
        vec![".db", ".sqlite", ".sqlite3", ".mdb", ".accdb"],
    );

    categories.insert(
        "code",
        vec![
            ".py", ".js", ".html", ".css", ".xml", ".json", ".yaml", ".yml", ".php", ".cpp", ".c",
            ".h", ".java", ".rs", ".go",
        ],
    );

    categories.insert("config", vec![".ini", ".conf", ".cfg", ".config"]);

    categories.insert("logs", vec![".log"]);

    categories
}

pub fn get_category(extension: &str) -> &'static str {
    let ext = extension.to_lowercase();
    let categories = get_categories();

    for (category, extensions) in categories.iter() {
        if extensions.contains(&ext.as_str()) {
            return category;
        }
    }

    "misc"
}

pub fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{}", s.to_lowercase()))
        .unwrap_or_default()
}
