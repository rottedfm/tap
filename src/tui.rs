// tui.rs
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::VecDeque;
use std::io;

pub const BANNER: &str = r#"
      ░██                               
      ██▒    ██                         
     ░██     ██                         
     ▓█▒   ███████    ▒████▓   ██░███▒  
     ██    ███████    ██████▓  ███████▒ 
    ▓█▒      ██       █▒  ▒██  ███  ███ 
    ██       ██        ▒█████  ██░  ░██ 
   ▒█▓       ██      ░███████  ██    ██ 
   ██        ██      ██▓░  ██  ██░  ░██ 
  ▒█▓        ██░     ██▒  ███  ███  ███ 
  ██░        █████   ████████  ███████▒ 
 ▒██         ░████    ▓███░██  ██░███▒  
 ██░                           ██       
                               ██       
                               ██       "#;

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

    /// Check terminal size and wait for resize if insufficient
    pub fn check_terminal_size(mode: &Mode) -> io::Result<()> {
        let term = Term::stdout();

        // Calculate space requirements
        const REQUIRED_WIDTH: usize = 115;

        // Calculate minimum height requirements:
        // - Banner: 23 lines
        // - Headers and separators: ~10 lines
        // - Content varies by section: max ~12 lines
        // - Navigation prompt: ~5 lines
        // Add buffer for safety
        let required_height = 23 + 10 + 12 + 5 + 2; // +2 buffer = 52 lines

        loop {
            let (rows, cols) = term.size();
            let width_ok = (cols as usize) >= REQUIRED_WIDTH;
            let height_ok = (rows as usize) >= required_height;

            if width_ok && height_ok {
                break;
            }

            term.clear_screen()?;
            Self::print_banner_with_mode_static(mode)?;
            println!();
            Self::print_warning_static("Terminal size insufficient for displaying content!")?;
            println!();

            if !width_ok {
                println!("  Width:  {} columns (minimum: {} required)", cols, REQUIRED_WIDTH);
            }
            if !height_ok {
                println!("  Height: {} rows (minimum: {} required)", rows, required_height);
            }

            println!();
            println!("Please resize your terminal window to continue...");
            println!();
            println!("TIP: The pie chart visualization requires extra width to display");
            println!("     category names, bars, percentages, sizes, and statistics.");

            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        Ok(())
    }

    /// Static version of print_banner_with_mode for early checks
    fn print_banner_with_mode_static(mode: &Mode) -> io::Result<()> {
        // Print banner
        println!("{}", BANNER);
        println!();
        println!("{}", "=".repeat(70));

        // Print mode
        println!("MODE: {}", mode.as_str());

        println!("{}", "=".repeat(70));

        Ok(())
    }

    /// Static version of print_warning for early checks
    fn print_warning_static(message: &str) -> io::Result<()> {
        println!("[!] WARNING: {}", message);
        Ok(())
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
                .template("{spinner:.white} {msg}")
                .unwrap()
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
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
                .template("{spinner:.white} {bar:40.bright_white/bright_white} {pos}/{len} ({percent}%) {msg}")
                .unwrap()
                .progress_chars("█ ")
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
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

    /// Print a summary section with navigation
    pub fn print_summary(
        &self,
        mode: &Mode,
        title: &str,
        stats: &[(String, usize, u64)],
        all_files: &[(String, u64, String)], // (name, size, category)
        total_drive_size: Option<u64>,
        _clear_before: bool,
    ) -> io::Result<()> {
        let mut total_files = 0;
        let mut total_size = 0u64;

        for (_category, count, size) in stats {
            total_files += count;
            total_size += size;
        }

        // Start navigation system
        let sections = vec!["Categories", "Statistics", "Largest Files"];
        let mut current_section = 0;

        loop {
            // Clear and redraw
            self.term.clear_screen()?;
            self.print_banner_with_mode(mode)?;
            println!();
            println!("{}", title);
            println!();
            println!("{}", "=".repeat(70));
            println!(
                "  TOTAL: {} files ({})",
                total_files,
                format_size(total_size)
            );
            println!("{}", "=".repeat(70));
            println!();

            // Display current section
            match sections[current_section] {
                "Categories" => {
                    println!("CATEGORY DISTRIBUTION");
                    println!();
                    let pie_chart = create_fixed_pie_chart(stats, total_drive_size);
                    for line in pie_chart {
                        println!("  {}", line);
                    }
                    println!();
                }
                "Statistics" => {
                    println!("STATISTICS");
                    println!();
                    let statistics = create_statistics_summary(stats, total_files, total_size);
                    for line in statistics {
                        println!("  {}", line);
                    }
                    println!();
                }
                "Largest Files" => {
                    println!("TOP 10 LARGEST FILES");
                    println!();
                    let leaderboard = create_leaderboard(all_files);
                    for line in leaderboard {
                        println!("  {}", line);
                    }
                    println!();
                }
                _ => {}
            }

            // Show navigation prompt
            let nav_choice = self.show_navigation_prompt(
                current_section,
                sections.len(),
                &sections[current_section],
            )?;

            match nav_choice.as_str() {
                "next" => {
                    if current_section < sections.len() - 1 {
                        current_section += 1;
                    }
                }
                "back" => {
                    if current_section > 0 {
                        current_section -= 1;
                    }
                }
                "exit" => {
                    break;
                }
                _ => break,
            }
        }

        Ok(())
    }

    /// Show navigation prompt with options
    fn show_navigation_prompt(
        &self,
        current_section: usize,
        total_sections: usize,
        section_name: &str,
    ) -> io::Result<String> {
        use dialoguer::{theme::ColorfulTheme, Select};

        println!("{}", "=".repeat(70));
        println!("Section {}/{}: {}", current_section + 1, total_sections, section_name);
        println!();

        let mut options = Vec::new();

        // Add "Next →" first when available (since it's the default)
        if current_section < total_sections - 1 {
            options.push("Next →");
        }

        if current_section > 0 {
            options.push("← Previous");
        }

        options.push("Continue");

        // Default to "Next →" if it exists in options, otherwise default to first option
        let default_index = options.iter().position(|&opt| opt == "Next →").unwrap_or(0);

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Navigate")
            .items(&options)
            .default(default_index)
            .interact_on(&self.term)?;

        let choice = options[selection];
        match choice {
            "← Previous" => Ok("back".to_string()),
            "Next →" => Ok("next".to_string()),
            "Continue" => Ok("exit".to_string()),
            _ => Ok("exit".to_string()),
        }
    }

    /// Print an info message
    pub fn print_info(&self, message: &str) -> io::Result<()> {
        println!("[*] {}", message);
        Ok(())
    }

    /// Print an error message
    pub fn print_error(&self, message: &str) -> io::Result<()> {
        println!("[!] ERROR: {}", message);
        Ok(())
    }

    /// Print a success message
    pub fn print_success(&self, message: &str) -> io::Result<()> {
        println!("[✓] {}", message);
        Ok(())
    }

    /// Print a warning message
    pub fn print_warning(&self, message: &str) -> io::Result<()> {
        println!("[!] WARNING: {}", message);
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

// Helper function to create fixed-size pie chart showing folder sizes and percentages
fn create_fixed_pie_chart(
    stats: &[(String, usize, u64)],
    total_drive_size: Option<u64>,
) -> Vec<String> {
    let mut lines = Vec::new();

    // Calculate total scanned size
    let total_scanned: u64 = stats.iter().map(|(_, _, size)| size).sum();
    if total_scanned == 0 {
        lines.push("No data to display".to_string());
        return lines;
    }

    // Use drive size if provided, otherwise use scanned size
    let reference_size = total_drive_size.unwrap_or(total_scanned);

    // Map categories to consistent characters based on category name
    fn get_category_char(_category: &str) -> &'static str {
        "█" // Use same block character for all categories
    }

    // Sort categories by size descending
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.2.cmp(&a.2));

    // Fixed bar width
    const BAR_WIDTH: usize = 40;

    for (category, count, size) in sorted_stats.iter() {
        let percentage_of_drive = (*size as f64 / reference_size as f64) * 100.0;
        let bar_length = ((*size as f64 / reference_size as f64) * BAR_WIDTH as f64) as usize;

        // Get consistent character for this category
        let char = get_category_char(category);

        // Build the bar
        let bar = if bar_length > 0 {
            char.repeat(bar_length)
        } else {
            " ".to_string()
        };

        // Format category name with fixed width
        let category_label = format!("{}:", category);

        // Calculate average file size for this category
        let avg_size = if *count > 0 {
            *size / (*count as u64)
        } else {
            0
        };

        lines.push(format!(
            "{} {:<15} {}{} {:>6.2}% {:>12} ({} files, avg: {})",
            char,
            category_label,
            bar,
            " ".repeat(BAR_WIDTH.saturating_sub(bar_length)),
            percentage_of_drive,
            format_size(*size),
            count,
            format_size(avg_size)
        ));
    }

    lines
}

// Helper function to create statistics summary
fn create_statistics_summary(
    stats: &[(String, usize, u64)],
    total_files: usize,
    total_size: u64,
) -> Vec<String> {
    let mut lines = Vec::new();

    if total_files == 0 {
        lines.push("No data to display".to_string());
        return lines;
    }

    // Calculate overall average file size
    let overall_avg = total_size / (total_files as u64);

    // Find largest and smallest category by size
    let largest_category = stats.iter().max_by_key(|(_, _, size)| size);
    let smallest_category = stats.iter().min_by_key(|(_, _, size)| size);

    // Find category with most files
    let most_files_category = stats.iter().max_by_key(|(_, count, _)| count);

    // Calculate median file size (approximation using sorted categories)
    let mut all_sizes: Vec<u64> = Vec::new();
    for (_, count, size) in stats {
        if *count > 0 {
            let avg_size = *size / (*count as u64);
            for _ in 0..*count {
                all_sizes.push(avg_size);
            }
        }
    }
    all_sizes.sort_unstable();
    let median = if !all_sizes.is_empty() {
        all_sizes[all_sizes.len() / 2]
    } else {
        0
    };

    // Display statistics
    lines.push(format!("Average file size:        {}", format_size(overall_avg)));
    lines.push(format!("Median file size:         {}", format_size(median)));
    lines.push(format!("Total categories:         {}", stats.len()));

    if let Some((cat, count, size)) = largest_category {
        lines.push(format!(
            "Largest category:         {} ({}, {} files)",
            cat,
            format_size(*size),
            count
        ));
    }

    if let Some((cat, count, size)) = smallest_category {
        lines.push(format!(
            "Smallest category:        {} ({}, {} files)",
            cat,
            format_size(*size),
            count
        ));
    }

    if let Some((cat, count, _)) = most_files_category {
        lines.push(format!(
            "Most files in category:   {} ({} files)",
            cat, count
        ));
    }

    lines
}

// Helper function to create top 10 largest files leaderboard
fn create_leaderboard(all_files: &[(String, u64, String)]) -> Vec<String> {
    let mut lines = Vec::new();

    if all_files.is_empty() {
        lines.push("No files to display".to_string());
        return lines;
    }

    // Sort by size descending and take top 10
    let mut sorted_files: Vec<_> = all_files.iter().collect();
    sorted_files.sort_by(|a, b| b.1.cmp(&a.1));
    let top_files: Vec<_> = sorted_files.iter().take(10).collect();

    // Header
    lines.push(format!(
        "{:<3} {:<35} {:<12} {:<15}",
        "Rank", "Name", "Size", "Category"
    ));
    lines.push("-".repeat(68));

    // Top 10 files
    for (rank, (name, size, category)) in top_files.iter().enumerate() {
        // Truncate long file names
        let display_name = if name.len() > 35 {
            format!("{}...", &name[..32])
        } else {
            name.to_string()
        };

        lines.push(format!(
            "{:<3} {:<35} {:<12} {:<15}",
            rank + 1,
            display_name,
            format_size(*size),
            category
        ));
    }

    lines
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
