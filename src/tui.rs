// tui.rs
use console::{style, Term};
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

    /// Init the UI with banner and mode
    pub fn init(&self, mode: &Mode, message: &str) -> io::Result<()> {
        self.term.clear_screen()?;
        self.term.hide_cursor()?;

        // Print banner
        println!("{}", style(BANNER).white().bold());
        println!("{}", style("=".repeat(70)).dim());
        println!();

        // Print mode
        let mode_styled = match mode {
            Mode::Inspect => style(format!("Mode: {} MODE", mode.as_str())).blue().bold(),
            Mode::Export => style(format!("Mode: {} MODE", mode.as_str()))
                .yellow()
                .bold(),
        };
        println!("{}", mode_styled);

        if !message.is_empty() {
            println!("{}", style(message).dim());
        }

        println!("{}", style("=".repeat(70)).dim());
        println!();

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

    /// Update recent files list with a new file
    pub fn update_recent_files(&mut self, path: String) -> io::Result<()> {
        self.add_recent_file(path);
        Ok(())
    }

    /// Draw the recent files section
    pub fn draw_recent_files(&self) -> io::Result<()> {
        println!();
        println!("{}", style("Currently processing:").bold());

        for file in &self.recent_files {
            // Truncate long paths to fit screen
            let display = if file.len() > 65 {
                format!("  {}...{}", &file[..30], &file[file.len() - 32..])
            } else {
                format!("  {}", file)
            };
            println!("{}", style(display).dim());
        }

        for _ in self.recent_files.len()..self.max_recent {
            println!();
        }

        Ok(())
    }

    /// Print a sumary section with statistics
    pub fn print_summary(&self, title: &str, stats: &[(String, usize, u64)]) -> io::Result<()> {
        // Clear recent files section first
        self.term.clear_last_lines(self.max_recent + 2)?;

        println!("{}", style("=".repeat(70)).dim());
        println!("{}", style(title).green().bold());
        println!("{}", style("=".repeat(70)).dim());
        println!();

        let mut total_files = 0;
        let mut total_size = 0u64;

        for (category, count, size) in stats {
            total_files += count;
            total_size += size;

            println!(
                "  {} {} files ({})",
                style(format!("{:.<20}", format!("{}:", category))).cyan(),
                style(count).bold(),
                format_size(*size)
            );
        }

        println!();
        println!("{}", style("-".repeat(70)).dim());
        println!(
            "  {} {} files ({})",
            style("Total:".to_string()).bold(),
            style(total_files).green().bold(),
            format_size(total_size)
        );

        Ok(())
    }

    /// Print an info message
    pub fn print_info(&self, message: &str) -> io::Result<()> {
        println!("{} {}", style("INFO").cyan().bold(), style(message).cyan());
        Ok(())
    }

    /// Print an error message
    pub fn print_error(&self, message: &str) -> io::Result<()> {
        println!("{} {}", style("ERROR").red().bold(), style(message).red());
        Ok(())
    }

    /// Print a success message
    pub fn print_success(&self, message: &str) -> io::Result<()> {
        println!("{} {}", style("SUCCESS").red().bold(), style(message).red());
        Ok(())
    }

    /// Print a warning message
    pub fn print_warning(&self, message: &str) -> io::Result<()> {
        println!(
            "{} {}",
            style("WARNING").yellow().bold(),
            style(message).yellow()
        );
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
