// config.rs
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub categories: HashMap<String, Vec<String>>,
    pub export: ExportConfig,
    pub zip: ZipConfig,
    pub ui: UIConfig,
    pub scan: ScanConfig,
    pub mount: MountConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub max_concurrent_copies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipConfig {
    pub enabled: bool,
    pub compression_level: u8,
    pub buffer_size_kb: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    pub max_recent_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    pub mount_base_dir: String,
    pub mount_prefix: String,
    pub device_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        let mut categories = HashMap::new();

        // Image Files
        categories.insert(
            "images".to_string(),
            vec![
                // Common formats
                ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".tiff", ".tif", ".svg", ".webp", ".ico",
                // Apple formats
                ".heic", ".heif",
                // RAW camera formats
                ".raw", ".cr2", ".nef", ".arw", ".dng", ".orf", ".rw2",
                // Other formats
                ".psd", ".ai", ".eps", ".indd", ".xcf", ".sketch", ".fig",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft Office Documents
        categories.insert(
            "documents".to_string(),
            vec![
                // Microsoft Word
                ".doc", ".docx", ".docm", ".dot", ".dotx", ".dotm",
                // PDF
                ".pdf",
                // Rich Text & Plain Text
                ".rtf", ".txt", ".text",
                // Markdown
                ".md", ".markdown",
                // OpenDocument Text
                ".odt", ".ott",
                // Apple Pages
                ".pages",
                // WordPerfect
                ".wpd", ".wp",
                // Other document formats
                ".tex", ".wps", ".wri", ".abw",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft PowerPoint Presentations
        categories.insert(
            "presentations".to_string(),
            vec![
                // Microsoft PowerPoint
                ".ppt", ".pptx", ".pptm", ".pot", ".potx", ".potm", ".pps", ".ppsx", ".ppsm", ".ppa", ".ppam",
                // OpenDocument Presentation
                ".odp", ".otp",
                // Apple Keynote
                ".key",
                // Google Slides (exported)
                ".gslides",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft Excel Spreadsheets
        categories.insert(
            "spreadsheets".to_string(),
            vec![
                // Microsoft Excel
                ".xls", ".xlsx", ".xlsm", ".xlsb", ".xlt", ".xltx", ".xltm", ".xla", ".xlam",
                // CSV and data files
                ".csv", ".tsv",
                // OpenDocument Spreadsheet
                ".ods", ".ots",
                // Apple Numbers
                ".numbers",
                // Google Sheets (exported)
                ".gsheet",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft Access & Other Office Files
        categories.insert(
            "databases".to_string(),
            vec![
                // Microsoft Access
                ".mdb", ".accdb", ".accde", ".accdt", ".accdr",
                // Database files
                ".db", ".sqlite", ".sqlite3", ".sql", ".dbf",
                // FileMaker
                ".fmp12", ".fp7",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft Outlook & Email
        categories.insert(
            "email".to_string(),
            vec![
                // Outlook
                ".msg", ".oft", ".ost", ".pst",
                // Email formats
                ".eml", ".emlx", ".mbox", ".mbx",
                // Apple Mail
                ".mailbox",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft OneNote & Note-taking
        categories.insert(
            "notes".to_string(),
            vec![
                // Microsoft OneNote
                ".one", ".onetoc2", ".onepkg",
                // Apple Notes (exported)
                ".note",
                // Evernote
                ".enex", ".enl",
                // Notion
                ".notion",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft Publisher & Design
        categories.insert(
            "publishing".to_string(),
            vec![
                // Microsoft Publisher
                ".pub",
                // Adobe InDesign
                ".indd", ".indt",
                // QuarkXPress
                ".qxd", ".qxp",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft Visio & Diagrams
        categories.insert(
            "diagrams".to_string(),
            vec![
                // Microsoft Visio
                ".vsd", ".vsdx", ".vsdm", ".vst", ".vstx", ".vstm", ".vss", ".vssx", ".vssm",
                // Other diagram formats
                ".drawio", ".vsdx",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Microsoft Project & Task Management
        categories.insert(
            "project_files".to_string(),
            vec![
                // Microsoft Project
                ".mpp", ".mpt",
                // Other project formats
                ".gan", ".planner",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Video Files
        categories.insert(
            "videos".to_string(),
            vec![
                // Common formats
                ".mp4", ".avi", ".mov", ".mkv", ".wmv", ".flv", ".webm", ".m4v", ".mpg", ".mpeg",
                ".m2v", ".3gp", ".3g2",
                // Professional formats
                ".mts", ".m2ts", ".ts", ".vob", ".ogv", ".mxf", ".roq", ".nsv", ".f4v", ".f4p", ".f4a", ".f4b",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Audio Files
        categories.insert(
            "audio".to_string(),
            vec![
                // Common formats
                ".mp3", ".wav", ".flac", ".aac", ".ogg", ".m4a", ".wma", ".aiff", ".aif", ".aifc",
                // Apple formats
                ".caf",
                // Other formats
                ".opus", ".ape", ".alac", ".amr", ".au", ".mka", ".mid", ".midi", ".ra", ".rm",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Archives & Compressed Files
        categories.insert(
            "archives".to_string(),
            vec![
                // Common formats
                ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2", ".xz", ".tgz", ".tbz2", ".tar.gz", ".tar.bz2", ".tar.xz",
                // Windows formats
                ".cab", ".msi", ".msix",
                // macOS formats
                ".dmg", ".pkg", ".app.zip",
                // Other formats
                ".z", ".lz", ".lzma", ".tlz", ".war", ".jar", ".iso", ".img", ".sit", ".sitx", ".sea", ".zipx",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Executable & Installation Files
        categories.insert(
            "executables".to_string(),
            vec![
                // Windows
                ".exe", ".msi", ".msix", ".appx", ".bat", ".cmd", ".com", ".scr", ".dll",
                // macOS
                ".app", ".dmg", ".pkg", ".command", ".workflow",
                // Linux
                ".deb", ".rpm", ".run", ".sh", ".AppImage",
                // Cross-platform
                ".jar",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Programming & Development
        categories.insert(
            "code".to_string(),
            vec![
                // Web
                ".html", ".htm", ".css", ".scss", ".sass", ".less", ".js", ".jsx", ".ts", ".tsx", ".vue", ".php", ".asp", ".aspx", ".jsp",
                // Python
                ".py", ".pyw", ".pyc", ".pyo", ".pyd",
                // Java
                ".java", ".class", ".jar",
                // C/C++
                ".c", ".cpp", ".cc", ".cxx", ".h", ".hpp", ".hxx",
                // C#
                ".cs", ".csx",
                // Objective-C
                ".m", ".mm",
                // Swift
                ".swift",
                // Rust
                ".rs",
                // Go
                ".go",
                // Ruby
                ".rb", ".erb",
                // Perl
                ".pl", ".pm",
                // R
                ".r", ".R",
                // Matlab
                ".m", ".mat",
                // Shell
                ".sh", ".bash", ".zsh", ".fish",
                // PowerShell
                ".ps1", ".psm1", ".psd1",
                // Batch
                ".bat", ".cmd",
                // Other
                ".lua", ".scala", ".kt", ".kts", ".dart", ".vim", ".el",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Data & Configuration Files
        categories.insert(
            "config".to_string(),
            vec![
                ".ini", ".conf", ".cfg", ".config", ".properties", ".toml", ".yaml", ".yml", ".json", ".json5", ".jsonc",
                ".xml", ".plist", ".reg", ".env", ".editorconfig", ".gitignore", ".gitattributes", ".dockerignore",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Fonts
        categories.insert(
            "fonts".to_string(),
            vec![
                ".ttf", ".otf", ".woff", ".woff2", ".eot", ".fon", ".fnt", ".dfont", ".suit",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // 3D & CAD Files
        categories.insert(
            "three_d".to_string(),
            vec![
                // 3D models
                ".obj", ".fbx", ".dae", ".3ds", ".blend", ".stl", ".ply", ".gltf", ".glb", ".usd", ".usdz",
                // CAD
                ".dwg", ".dxf", ".dwf", ".step", ".stp", ".iges", ".igs", ".ipt", ".iam", ".sldprt", ".sldasm",
                ".catpart", ".catproduct",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // eBooks
        categories.insert(
            "ebooks".to_string(),
            vec![
                ".epub", ".mobi", ".azw", ".azw3", ".kf8", ".ibooks", ".fb2", ".djvu", ".cbr", ".cbz", ".cb7", ".cbt",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Backup Files
        categories.insert(
            "backups".to_string(),
            vec![
                ".bak", ".backup", ".old", ".orig", ".tmp", ".temp", ".swp", ".swo", "~", ".gho", ".bkf", ".bck",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // System Files
        categories.insert(
            "system".to_string(),
            vec![
                // Windows
                ".sys", ".dll", ".ocx", ".drv", ".cpl", ".scr", ".ini", ".dat",
                // macOS
                ".ds_store", ".localized", ".plist",
                // Linux
                ".so", ".ko",
                // Other
                ".lnk", ".url", ".webloc",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Virtual Machine & Disk Images
        categories.insert(
            "virtual".to_string(),
            vec![
                ".vmdk", ".vdi", ".vhd", ".vhdx", ".hdd", ".ova", ".ovf", ".qcow", ".qcow2", ".iso", ".img",
                ".toast", ".cdr",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Logs & Reports
        categories.insert(
            "logs".to_string(),
            vec![
                ".log", ".out", ".trace", ".dmp", ".crash", ".diag",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Certificates & Security
        categories.insert(
            "certificates".to_string(),
            vec![
                ".cer", ".crt", ".der", ".p7b", ".p7c", ".p12", ".pfx", ".pem", ".key", ".pub", ".sig", ".gpg",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Web & Internet
        categories.insert(
            "web".to_string(),
            vec![
                ".html", ".htm", ".mhtml", ".mht", ".url", ".webloc", ".website", ".download", ".crdownload", ".part",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Subtitles & Captions
        categories.insert(
            "subtitles".to_string(),
            vec![
                ".srt", ".sub", ".sbv", ".ass", ".ssa", ".vtt", ".idx",
            ].iter().map(|s| s.to_string()).collect(),
        );

        // Torrent Files
        categories.insert(
            "torrents".to_string(),
            vec![
                ".torrent", ".magnet",
            ].iter().map(|s| s.to_string()).collect(),
        );

        Self {
            categories,
            export: ExportConfig {
                max_concurrent_copies: 10,
            },
            zip: ZipConfig {
                enabled: true,
                compression_level: 6,
                buffer_size_kb: 256,
            },
            ui: UIConfig {
                max_recent_files: 10,
            },
            scan: ScanConfig {
                exclude_patterns: vec![
                    ".*".to_string(),  // Hidden files/directories
                    "System Volume Information".to_string(),
                    "$RECYCLE.BIN".to_string(),
                    "node_modules".to_string(),
                ],
            },
            mount: MountConfig {
                mount_base_dir: "/mnt".to_string(),
                mount_prefix: "tap_".to_string(),
                device_patterns: vec![
                    "/dev/sd".to_string(),    // SATA
                    "/dev/nvme".to_string(),  // NVMe
                    "/dev/mmcblk".to_string(), // MMC
                    "/dev/vd".to_string(),    // Virtual
                ],
            },
        }
    }
}

impl Config {
    /// Get the config directory path
    fn get_config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| color_eyre::eyre::eyre!("Could not determine home directory"))?;

        Ok(PathBuf::from(home).join(".config").join("tap"))
    }

    /// Get the config file path
    fn get_config_path() -> Result<PathBuf> {
        Ok(Self::get_config_dir()?.join("config.toml"))
    }

    /// Load config from file, creating default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            println!("INFO: Config file not found, creating default config...");
            let config = Self::default();
            config.save()?;
            println!("INFO: Default config created at: {}", config_path.display());
            return Ok(config);
        }

        let contents = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir)?;

        let config_path = Self::get_config_path()?;
        let contents = toml::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;

        Ok(())
    }

    /// Get category for a file extension
    pub fn get_category(&self, extension: &str) -> String {
        let ext = extension.to_lowercase();

        for (category, extensions) in self.categories.iter() {
            if extensions.iter().any(|e| e == &ext) {
                return category.clone();
            }
        }

        "misc".to_string()
    }

    /// Get extension from a file path
    pub fn get_extension(path: &std::path::Path) -> String {
        path.extension()
            .and_then(|s| s.to_str())
            .map(|s| format!(".{}", s.to_lowercase()))
            .unwrap_or_default()
    }
}
