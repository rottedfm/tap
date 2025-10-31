// tui.rs
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::io;

const BANNER: &str = r#"

    .....               ..                  ....      ..     
 .H8888888h.  ~-.    :**888H: `: .xH""    +^""888h. ~"888h   
 888888888888x  `>  X   `8888k XX888     8X.  ?8888X  8888f  
X~     `?888888hx~ '8hx  48888 ?8888    '888x  8888X  8888~  
'      x8.^"*88*"  '8888 '8888 `8888    '88888 8888X   "88x: 
 `-:- X8888x        %888>'8888  8888     `8888 8888X  X88x.  
      488888>         "8 '888"  8888       `*` 8888X '88888X 
    .. `"88*         .-` X*"    8888      ~`...8888X  "88888 
  x88888nX"      .     .xhx.    8888       x8888888X.   `%8" 
 !"*8888888n..  :    .H88888h.~`8888.>    '%"*8888888h.   "  
'    "*88888888*    .~  `%88!` '888*~     ~    888888888!`   
        ^"***"`           `"     ""            X888^"""      
                                               `88f          
                                                88           
                                                ""           "#;

pub enum Mode {
    Inspect,
    Export,
}

impl Mode {
    pub fn as_str(&self) -> &str {
        match self {
            Mode::Inspect => "INSPECT",
            Mode::Export => "EXPORT",
        }
    }
}

// TODO: Get max recent from toml
pub struct UI {
    pub term: Term,
    recent_files: VecDeque<String>,
    pub max_recent: usize,
}

impl UI {
    pub fn new() -> io::Result<Self> {
        let term = Term::stdout();
        Ok(Self {
            term,
            recent_files: VecDeque::with_capacity(10),
            max_recent: 10,
        })
    }

    /// Print banner with mode
    pub fn print_banner_with_mode(&self, mode: &Mode) -> io::Result<()> {
        // Print banner
        println!("{}", BANNER);
        println!();
        println!("{}", "=".repeat(70));

        // Print mode
        println!("MODE: {}", mode.as_str());

        println!("{}", "=".repeat(70));

        Ok(())
    }

    /// Init the UI with banner and mode
    pub fn init(&self, mode: &Mode, message: &str) -> io::Result<()> {
        self.term.clear_screen()?;
        self.term.hide_cursor()?;

        self.print_banner_with_mode(mode)?;

        if !message.is_empty() {
            println!();
            println!("{}", message);
            println!();
        }

        Ok(())
    }

    /// Create a progress bar for counting/scanning
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars(r#"/-\\|"#),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    /// create a progess bar with known total
    pub fn create_progress_bar(&self, total: u64, message: &str) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("#-")
                .tick_chars(r#"/-\\|"#),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    /// Add a file to the recent files list
    pub fn add_recent_file(&mut self, path: String) {
        if self.recent_files.len() >= self.max_recent {
            self.recent_files.pop_back();
        }

        self.recent_files.push_front(path);
    }

    /// Update recent files list with a new file and redraw the display
    pub fn update_recent_files(&mut self, path: String) -> io::Result<()> {
        self.add_recent_file(path);

        // Move cursor up to the start of the recent files content (not including the header)
        // We need to move up past: max_recent file lines
        for _ in 0..self.max_recent {
            self.term.move_cursor_up(1)?;
        }

        // Redraw just the file list (not the header), clearing each line as we go
        for file in &self.recent_files {
            self.term.clear_line()?;
            // Truncate long paths to fit screen
            let display = if file.len() > 65 {
                format!("  {}...{}", &file[..30], &file[file.len() - 32..])
            } else {
                format!("  {}", file)
            };
            println!("{}", display);
        }

        // Fill remaining lines with blanks (clear them)
        for _ in self.recent_files.len()..self.max_recent {
            self.term.clear_line()?;
            println!();
        }

        Ok(())
    }

    /// Draw the recent files section
    pub fn draw_recent_files(&self) -> io::Result<()> {
        println!("{}", "=".repeat(70));
        println!("RECENT FILES:");

        for file in &self.recent_files {
            // Truncate long paths to fit screen
            let display = if file.len() > 65 {
                format!("  {}...{}", &file[..30], &file[file.len() - 32..])
            } else {
                format!("  {}", file)
            };
            println!("{}", display);
        }

        for _ in self.recent_files.len()..self.max_recent {
            println!();
        }

        Ok(())
    }

    /// Print a sumary section with statistics
    pub fn print_summary(
        &self,
        title: &str,
        stats: &[(String, usize, u64)],
        clear_before: bool,
    ) -> io::Result<()> {
        // Clear recent files section first if requested
        if clear_before {
            self.term.clear_last_lines(self.max_recent + 2)?;
        }

        println!();
        println!("{}", title);
        println!();

        let mut total_files = 0;
        let mut total_size = 0u64;

        for (category, count, size) in stats {
            total_files += count;
            total_size += size;

            println!(
                "  {:.<20} {} files ({})",
                format!("{}:", category),
                count,
                format_size(*size)
            );
        }

        println!();
        println!("{}", "=".repeat(70));
        println!(
            "  TOTAL: {} files ({})",
            total_files,
            format_size(total_size)
        );

        Ok(())
    }

    /// Print an info message
    pub fn print_info(&self, message: &str) -> io::Result<()> {
        println!("INFO: {}", message);
        Ok(())
    }

    /// Print an error message
    pub fn print_error(&self, message: &str) -> io::Result<()> {
        println!("ERROR: {}", message);
        Ok(())
    }

    /// Print a success message
    pub fn print_success(&self, message: &str) -> io::Result<()> {
        println!("SUCCESS: {}", message);
        Ok(())
    }

    /// Print a warning message
    pub fn print_warning(&self, message: &str) -> io::Result<()> {
        println!("WARNING: {}", message);
        Ok(())
    }

    /// Cleanup the terminal (show cursor, etc.)
    pub fn cleanup(&self) -> io::Result<()> {
        self.term.show_cursor()?;
        Ok(())
    }
}

impl Default for UI {
    fn default() -> Self {
        Self::new().expect("Failed to create UI")
    }
}

impl Drop for UI {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

// Helper function to format file sizes
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}
