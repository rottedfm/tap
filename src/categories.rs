//! File categorization and extension mapping.
//!
//! This module provides functionality for categorizing files based on their extensions.
//! Note: This module is deprecated in favor of the config-based categorization system.
//! It's maintained for backwards compatibility but the [`config`](crate::config) module
//! provides a more comprehensive and configurable solution.

use std::collections::HashMap;
use std::path::Path;

/// Returns a static mapping of file categories to their associated extensions.
///
/// This function provides a basic categorization scheme for common file types.
/// For a more comprehensive and configurable solution, use the categories defined
/// in the [`Config`](crate::config::Config) struct.
///
/// # Examples
///
/// ```
/// use tap::categories::get_categories;
///
/// let categories = get_categories();
/// assert!(categories.contains_key("documents"));
/// assert!(categories.contains_key("images"));
/// ```
///
/// # Note
///
/// This function is maintained for backwards compatibility. The configuration-based
/// categorization system in [`Config`](crate::config::Config) is more comprehensive.
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

/// Determines the category for a file based on its extension.
///
/// This function looks up the provided extension in the category mappings
/// and returns the corresponding category name. If no match is found,
/// returns "misc" as the default category.
///
/// # Arguments
///
/// * `extension` - The file extension including the leading dot (e.g., ".txt", ".jpg")
///
/// # Returns
///
/// A string slice representing the category name. Returns "misc" if the
/// extension doesn't match any known category.
///
/// # Examples
///
/// ```
/// use tap::categories::get_category;
///
/// assert_eq!(get_category(".pdf"), "documents");
/// assert_eq!(get_category(".jpg"), "images");
/// assert_eq!(get_category(".unknown"), "misc");
/// ```
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

/// Extracts the file extension from a path.
///
/// This function extracts the extension from a file path and returns it
/// in lowercase with a leading dot. If the path has no extension, returns
/// an empty string.
///
/// # Arguments
///
/// * `path` - The file path to extract the extension from
///
/// # Returns
///
/// A `String` containing the extension with a leading dot in lowercase,
/// or an empty string if no extension exists.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use tap::categories::get_extension;
///
/// let path = Path::new("/home/user/document.PDF");
/// assert_eq!(get_extension(path), ".pdf");
///
/// let no_ext = Path::new("/home/user/README");
/// assert_eq!(get_extension(&no_ext), "");
/// ```
pub fn get_extension(path: &Path) -> String {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| format!(".{}", s.to_lowercase()))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_category_documents() {
        assert_eq!(get_category(".pdf"), "documents");
        assert_eq!(get_category(".doc"), "documents");
        assert_eq!(get_category(".docx"), "documents");
        assert_eq!(get_category(".txt"), "documents");
    }

    #[test]
    fn test_get_category_images() {
        assert_eq!(get_category(".jpg"), "images");
        assert_eq!(get_category(".jpeg"), "images");
        assert_eq!(get_category(".png"), "images");
        assert_eq!(get_category(".gif"), "images");
    }

    #[test]
    fn test_get_category_videos() {
        assert_eq!(get_category(".mp4"), "videos");
        assert_eq!(get_category(".avi"), "videos");
        assert_eq!(get_category(".mov"), "videos");
    }

    #[test]
    fn test_get_category_code() {
        assert_eq!(get_category(".py"), "code");
        assert_eq!(get_category(".js"), "code");
        assert_eq!(get_category(".rs"), "code");
        assert_eq!(get_category(".java"), "code");
    }

    #[test]
    fn test_get_category_unknown() {
        assert_eq!(get_category(".unknown"), "misc");
        assert_eq!(get_category(".xyz"), "misc");
        assert_eq!(get_category(""), "misc");
    }

    #[test]
    fn test_get_category_case_insensitive() {
        assert_eq!(get_category(".PDF"), "documents");
        assert_eq!(get_category(".JPG"), "images");
        assert_eq!(get_category(".Mp4"), "videos");
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension(Path::new("file.txt")), ".txt");
        assert_eq!(get_extension(Path::new("image.JPG")), ".jpg");
        assert_eq!(get_extension(Path::new("document.PDF")), ".pdf");
    }

    #[test]
    fn test_get_extension_no_extension() {
        assert_eq!(get_extension(Path::new("README")), "");
        assert_eq!(get_extension(Path::new("Makefile")), "");
    }

    #[test]
    fn test_get_extension_multiple_dots() {
        assert_eq!(get_extension(Path::new("archive.tar.gz")), ".gz");
        assert_eq!(get_extension(Path::new("file.backup.txt")), ".txt");
    }

    #[test]
    fn test_get_categories_completeness() {
        let categories = get_categories();

        assert!(categories.contains_key("documents"));
        assert!(categories.contains_key("images"));
        assert!(categories.contains_key("videos"));
        assert!(categories.contains_key("audio"));
        assert!(categories.contains_key("archives"));
        assert!(categories.contains_key("code"));
        assert!(categories.contains_key("spreadsheets"));
    }

    #[test]
    fn test_get_categories_extensions_not_empty() {
        let categories = get_categories();

        for (category, extensions) in categories.iter() {
            assert!(!extensions.is_empty(),
                    "Category '{}' has no extensions", category);
        }
    }
}
