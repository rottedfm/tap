//! Terminal user interface components.
//!
//! This module provides a rich terminal UI with progress tracking, themed colors,
//! navigation, and various visualization components for file statistics.

use console::Term;
use dialoguer::theme::{ColorfulTheme, Theme};
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
    pub color_theme: String,
}

impl UI {
    pub fn new() -> io::Result<Self> {
        let term = Term::stdout();
        Ok(Self {
            term,
            recent_files: VecDeque::with_capacity(3),
            max_recent: 3,
            color_theme: "default".to_string(),
        })
    }

    pub fn with_color_theme(mut self, theme: String) -> Self {
        self.color_theme = theme;
        self
    }

    /// Get the console::Style for the configured theme
    fn get_style(&self) -> console::Style {
        use console::Style;

        match self.color_theme.as_str() {
            "cyan" => Style::new().cyan(),
            "magenta" => Style::new().magenta(),
            "yellow" => Style::new().yellow(),
            "green" => Style::new().green(),
            "red" => Style::new().red(),
            "blue" => Style::new().blue(),
            "white" => Style::new().white(),
            _ => Style::new().white(),
        }
    }

    /// Get different shades for status codes based on theme
    /// Returns (info_style, warning_style, error_style, success_style)
    fn get_status_styles(
        &self,
    ) -> (
        console::Style,
        console::Style,
        console::Style,
        console::Style,
    ) {
        use console::Style;

        match self.color_theme.as_str() {
            "cyan" => (
                Style::new().cyan(),        // info - base
                Style::new().color256(51),  // warning - bright cyan
                Style::new().color256(87),  // error - darker cyan
                Style::new().color256(123), // success - lighter cyan
            ),
            "magenta" => (
                Style::new().magenta(),     // info - base
                Style::new().color256(201), // warning - bright magenta
                Style::new().color256(126), // error - darker magenta
                Style::new().color256(213), // success - lighter magenta
            ),
            "yellow" => (
                Style::new().yellow(),      // info - base
                Style::new().color256(226), // warning - bright yellow
                Style::new().color256(178), // error - darker yellow/orange
                Style::new().color256(227), // success - lighter yellow
            ),
            "green" => (
                Style::new().green(),       // info - base
                Style::new().color256(46),  // warning - bright green
                Style::new().color256(28),  // error - darker green
                Style::new().color256(120), // success - lighter green
            ),
            "red" => (
                Style::new().red(),         // info - base
                Style::new().color256(196), // warning - bright red
                Style::new().color256(124), // error - darker red
                Style::new().color256(210), // success - lighter red/pink
            ),
            "blue" => (
                Style::new().blue(),        // info - base
                Style::new().color256(39),  // warning - bright blue
                Style::new().color256(25),  // error - darker blue
                Style::new().color256(117), // success - lighter blue
            ),
            "white" => (
                Style::new().white(),       // info - base
                Style::new().color256(255), // warning - bright white
                Style::new().color256(250), // error - darker white/gray
                Style::new().color256(255), // success - bright white
            ),
            _ => (
                Style::new().white(),
                Style::new().color256(255),
                Style::new().color256(250),
                Style::new().color256(255),
            ),
        }
    }

    /// Get spinner color string for progress bar templates
    fn get_spinner_color(&self) -> &str {
        match self.color_theme.as_str() {
            "cyan" => ".cyan",
            "magenta" => ".magenta",
            "yellow" => ".yellow",
            "green" => ".green",
            "red" => ".red",
            "blue" => ".blue",
            "white" => ".white",
            _ => ".white",
        }
    }

    /// Get bar colors (spinner_color, bar_color) for progress bar templates
    fn get_bar_colors(&self) -> (&str, &str) {
        match self.color_theme.as_str() {
            "cyan" => (".cyan", "bright_cyan/bright_cyan"),
            "magenta" => (".magenta", "bright_magenta/bright_magenta"),
            "yellow" => (".yellow", "bright_yellow/bright_yellow"),
            "green" => (".green", "bright_green/bright_green"),
            "red" => (".red", "bright_red/bright_red"),
            "blue" => (".blue", "bright_blue/bright_blue"),
            "white" => (".white", "bright_white/bright_white"),
            _ => (".white", "bright_white/bright_white"),
        }
    }

    /// Create a themed ColorfulTheme based on the configured color
    fn get_theme(&self) -> Box<dyn Theme> {
        use console::{Style, style};

        match self.color_theme.as_str() {
            "cyan" => Box::new(ColorfulTheme {
                values_style: Style::new().cyan(),
                active_item_style: Style::new().cyan().bold(),
                active_item_prefix: style("❯".to_string()).cyan().bold(),
                ..ColorfulTheme::default()
            }),
            "magenta" => Box::new(ColorfulTheme {
                values_style: Style::new().magenta(),
                active_item_style: Style::new().magenta().bold(),
                active_item_prefix: style("❯".to_string()).magenta().bold(),
                ..ColorfulTheme::default()
            }),
            "yellow" => Box::new(ColorfulTheme {
                values_style: Style::new().yellow(),
                active_item_style: Style::new().yellow().bold(),
                active_item_prefix: style("❯".to_string()).yellow().bold(),
                ..ColorfulTheme::default()
            }),
            "green" => Box::new(ColorfulTheme {
                values_style: Style::new().green(),
                active_item_style: Style::new().green().bold(),
                active_item_prefix: style("❯".to_string()).green().bold(),
                ..ColorfulTheme::default()
            }),
            "red" => Box::new(ColorfulTheme {
                values_style: Style::new().red(),
                active_item_style: Style::new().red().bold(),
                active_item_prefix: style("❯".to_string()).red().bold(),
                ..ColorfulTheme::default()
            }),
            "blue" => Box::new(ColorfulTheme {
                values_style: Style::new().blue(),
                active_item_style: Style::new().blue().bold(),
                active_item_prefix: style("❯".to_string()).blue().bold(),
                ..ColorfulTheme::default()
            }),
            "white" => Box::new(ColorfulTheme {
                values_style: Style::new().white(),
                active_item_style: Style::new().white().bold(),
                active_item_prefix: style("❯".to_string()).white().bold(),
                ..ColorfulTheme::default()
            }),
            _ => Box::new(ColorfulTheme::default()),
        }
    }

    /// Get a static ColorfulTheme based on theme string for use in static contexts
    pub fn get_colorful_theme(theme: &str) -> ColorfulTheme {
        use console::{Style, style};

        match theme {
            "cyan" => ColorfulTheme {
                values_style: Style::new().cyan(),
                active_item_style: Style::new().cyan().bold(),
                active_item_prefix: style("❯".to_string()).cyan().bold(),
                ..ColorfulTheme::default()
            },
            "magenta" => ColorfulTheme {
                values_style: Style::new().magenta(),
                active_item_style: Style::new().magenta().bold(),
                active_item_prefix: style("❯".to_string()).magenta().bold(),
                ..ColorfulTheme::default()
            },
            "yellow" => ColorfulTheme {
                values_style: Style::new().yellow(),
                active_item_style: Style::new().yellow().bold(),
                active_item_prefix: style("❯".to_string()).yellow().bold(),
                ..ColorfulTheme::default()
            },
            "green" => ColorfulTheme {
                values_style: Style::new().green(),
                active_item_style: Style::new().green().bold(),
                active_item_prefix: style("❯".to_string()).green().bold(),
                ..ColorfulTheme::default()
            },
            "red" => ColorfulTheme {
                values_style: Style::new().red(),
                active_item_style: Style::new().red().bold(),
                active_item_prefix: style("❯".to_string()).red().bold(),
                ..ColorfulTheme::default()
            },
            "blue" => ColorfulTheme {
                values_style: Style::new().blue(),
                active_item_style: Style::new().blue().bold(),
                active_item_prefix: style("❯".to_string()).blue().bold(),
                ..ColorfulTheme::default()
            },
            "white" => ColorfulTheme {
                values_style: Style::new().white(),
                active_item_style: Style::new().white().bold(),
                active_item_prefix: style("❯".to_string()).white().bold(),
                ..ColorfulTheme::default()
            },
            _ => ColorfulTheme::default(),
        }
    }

    /// Get static status styles based on theme string
    /// Returns (info_style, warning_style, error_style, success_style)
    pub fn get_static_status_styles(
        theme: &str,
    ) -> (
        console::Style,
        console::Style,
        console::Style,
        console::Style,
    ) {
        use console::Style;

        match theme {
            "cyan" => (
                Style::new().cyan(),        // info - base
                Style::new().color256(51),  // warning - bright cyan
                Style::new().color256(87),  // error - darker cyan
                Style::new().color256(123), // success - lighter cyan
            ),
            "magenta" => (
                Style::new().magenta(),     // info - base
                Style::new().color256(201), // warning - bright magenta
                Style::new().color256(126), // error - darker magenta
                Style::new().color256(213), // success - lighter magenta
            ),
            "yellow" => (
                Style::new().yellow(),      // info - base
                Style::new().color256(226), // warning - bright yellow
                Style::new().color256(178), // error - darker yellow/orange
                Style::new().color256(227), // success - lighter yellow
            ),
            "green" => (
                Style::new().green(),       // info - base
                Style::new().color256(46),  // warning - bright green
                Style::new().color256(28),  // error - darker green
                Style::new().color256(120), // success - lighter green
            ),
            "red" => (
                Style::new().red(),         // info - base
                Style::new().color256(196), // warning - bright red
                Style::new().color256(124), // error - darker red
                Style::new().color256(210), // success - lighter red/pink
            ),
            "blue" => (
                Style::new().blue(),        // info - base
                Style::new().color256(39),  // warning - bright blue
                Style::new().color256(25),  // error - darker blue
                Style::new().color256(117), // success - lighter blue
            ),
            "white" => (
                Style::new().white(),       // info - base
                Style::new().color256(255), // warning - bright white
                Style::new().color256(250), // error - darker white/gray
                Style::new().color256(255), // success - bright white
            ),
            _ => (
                Style::new().white(),
                Style::new().color256(255),
                Style::new().color256(250),
                Style::new().color256(255),
            ),
        }
    }

    /// Check terminal size and wait for resize if insufficient
    pub fn check_terminal_size(mode: &Mode, theme: &str) -> io::Result<()> {
        use console::Style;

        let term = Term::stdout();

        // Get style for theme
        let style = match theme {
            "cyan" => Style::new().cyan(),
            "magenta" => Style::new().magenta(),
            "yellow" => Style::new().yellow(),
            "green" => Style::new().green(),
            "red" => Style::new().red(),
            "blue" => Style::new().blue(),
            "white" => Style::new().white(),
            _ => Style::new().white(),
        };

        // Calculate space requirements
        const REQUIRED_WIDTH: usize = 115;

        // Calculate minimum height requirements:
        // - Banner: 23 lines
        // - Headers and separators: ~10 lines
        // - Content varies by section: max ~12 lines
        // - Navigation prompt: ~5 lines
        // Add buffer for safety
        let required_height = 30;

        loop {
            let (rows, cols) = term.size();
            let width_ok = (cols as usize) >= REQUIRED_WIDTH;
            let height_ok = (rows as usize) >= required_height;

            if width_ok && height_ok {
                break;
            }

            use console::Style;
            let white_bold = Style::new().white().bold();

            term.clear_screen()?;
            Self::print_banner_with_mode_static(mode, &style)?;
            println!();
            Self::print_warning_static(
                "Terminal size insufficient for displaying content!",
                &style,
            )?;
            println!();

            if !width_ok {
                println!(
                    "{}",
                    white_bold.apply_to(format!(
                        "  Width:  {} columns (minimum: {} required)",
                        cols, REQUIRED_WIDTH
                    ))
                );
            }
            if !height_ok {
                println!(
                    "{}",
                    white_bold.apply_to(format!(
                        "  Height: {} rows (minimum: {} required)",
                        rows, required_height
                    ))
                );
            }

            println!();
            println!(
                "{}",
                white_bold.apply_to("Please resize your terminal window to continue...")
            );
            println!();
            println!(
                "{}",
                white_bold
                    .apply_to("TIP: The pie chart visualization requires extra width to display")
            );
            println!(
                "{}",
                white_bold
                    .apply_to("     category names, bars, percentages, sizes, and statistics.")
            );

            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        Ok(())
    }

    /// Static version of print_banner_with_mode for early checks
    fn print_banner_with_mode_static(mode: &Mode, style: &console::Style) -> io::Result<()> {
        use console::Style;
        let white_bold = Style::new().white().bold();

        // Print banner
        println!("{}", style.apply_to(BANNER).bold());
        println!();
        println!("{}", white_bold.apply_to("=".repeat(70)));

        // Print mode - "MODE:" is themed and bold, mode name is white, bold, and italic
        println!(
            "{} {}",
            style.apply_to("MODE:").bold(),
            white_bold.apply_to(mode.as_str()).italic()
        );

        println!("{}", white_bold.apply_to("=".repeat(70)));

        Ok(())
    }

    /// Static version of print_warning for early checks
    fn print_warning_static(message: &str, style: &console::Style) -> io::Result<()> {
        use console::Style;
        let white_bold = Style::new().white().bold();
        println!(
            "{} {}",
            style.apply_to("[!] WARNING:").bold(),
            white_bold.apply_to(message)
        );
        Ok(())
    }

    /// Print banner with mode
    pub fn print_banner_with_mode(&self, mode: &Mode) -> io::Result<()> {
        use console::Style;
        let style = self.get_style();
        let white_bold = Style::new().white().bold();

        // Print banner
        println!("{}", style.apply_to(BANNER).bold());
        println!();
        println!("{}", white_bold.apply_to("=".repeat(70)));

        // Print mode - "MODE:" is themed and bold, mode name is white, bold, and italic
        println!(
            "{} {}",
            style.apply_to("MODE:").bold(),
            white_bold.apply_to(mode.as_str()).italic()
        );

        println!("{}", white_bold.apply_to("=".repeat(70)));

        Ok(())
    }

    /// Init the UI with banner and mode
    pub fn init(&self, mode: &Mode, message: &str) -> io::Result<()> {
        use console::Style;
        let white_bold = Style::new().white().bold();

        self.term.clear_screen()?;
        self.term.hide_cursor()?;

        self.print_banner_with_mode(mode)?;

        if !message.is_empty() {
            println!();
            println!("{}", white_bold.apply_to(message));
            println!();
        }

        Ok(())
    }

    /// Create a progress bar for counting/scanning
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        let spinner_color = self.get_spinner_color();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template(&format!("{{spinner:{}}} {{msg}}", spinner_color))
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
        let (spinner_color, bar_color) = self.get_bar_colors();
        pb.set_style(
            ProgressStyle::default_bar()
                .template(&format!(
                    "{{spinner:{}}} {{bar:40.{}/{}}} {{pos}}/{{len}} ({{percent}}%) {{msg}}",
                    spinner_color, bar_color, bar_color
                ))
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
        use console::Style;
        let white_bold = Style::new().white().bold();

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
            let display = format!("  {}", safe_truncate_path(file, 65));
            println!("{}", white_bold.apply_to(display));
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
        use console::Style;
        let style = self.get_style();
        let white_bold = Style::new().white().bold();

        println!("{}", white_bold.apply_to("=".repeat(70)));
        println!("{}", style.apply_to("RECENT FILES:").bold());

        for file in &self.recent_files {
            // Truncate long paths to fit screen
            let display = format!("  {}", safe_truncate_path(file, 65));
            println!("{}", white_bold.apply_to(display));
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
        let sections = ["Categories", "Statistics", "Largest Files"];
        let mut current_section = 0;

        loop {
            // Clear and redraw
            self.term.clear_screen()?;
            self.print_banner_with_mode(mode)?;

            use console::Style;
            let style = self.get_style();
            let white_bold = Style::new().white().bold();

            println!();
            println!("{}", style.apply_to(title).bold());
            println!();
            println!("{}", white_bold.apply_to("=".repeat(70)));
            println!(
                "  {} {} {} {}",
                style.apply_to("TOTAL:").bold(),
                white_bold.apply_to(format!("{}", total_files)).italic(),
                white_bold.apply_to("files"),
                white_bold
                    .apply_to(format!("({})", format_size(total_size)))
                    .italic()
            );
            println!("{}", white_bold.apply_to("=".repeat(70)));
            println!();

            // Display current section
            match sections[current_section] {
                "Categories" => {
                    println!("{}", style.apply_to("CATEGORY DISTRIBUTION").bold());
                    println!();
                    let pie_chart =
                        create_fixed_pie_chart(stats, total_drive_size, &self.color_theme);
                    for line in pie_chart {
                        println!("  {}", line);
                    }
                    println!();
                }
                "Statistics" => {
                    println!("{}", style.apply_to("STATISTICS").bold());
                    println!();
                    let statistics = create_statistics_summary(stats, total_files, total_size);
                    for line in statistics {
                        println!("  {}", line);
                    }
                    println!();
                }
                "Largest Files" => {
                    println!("{}", style.apply_to("TOP 10 LARGEST FILES").bold());
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
                sections[current_section],
            )?;

            match nav_choice.as_str() {
                "next" => {
                    if current_section < sections.len() - 1 {
                        current_section += 1;
                    }
                }
                "back" => {
                    current_section = current_section.saturating_sub(1);
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
        use console::Style;
        use dialoguer::Select;

        let style = self.get_style();
        let white_bold = Style::new().white().bold();

        println!("{}", white_bold.apply_to("=".repeat(70)));
        println!(
            "{} {}",
            style
                .apply_to(format!(
                    "Section {}/{}:",
                    current_section + 1,
                    total_sections
                ))
                .bold(),
            white_bold.apply_to(section_name)
        );
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

        let theme = self.get_theme();
        let selection = Select::with_theme(theme.as_ref())
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
        use console::Style;
        let (info_style, _, _, _) = self.get_status_styles();
        let white_bold = Style::new().white().bold();
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold.apply_to(message)
        );
        Ok(())
    }

    /// Print an error message
    pub fn print_error(&self, message: &str) -> io::Result<()> {
        use console::Style;
        let (_, _, error_style, _) = self.get_status_styles();
        let white_bold = Style::new().white().bold();
        println!(
            "{} {}",
            error_style.apply_to("[!] ERROR:").bold(),
            white_bold.apply_to(message)
        );
        Ok(())
    }

    /// Print a success message
    pub fn print_success(&self, message: &str) -> io::Result<()> {
        use console::Style;
        let (_, _, _, success_style) = self.get_status_styles();
        let white_bold = Style::new().white().bold();
        println!(
            "{} {}",
            success_style.apply_to("[✓]").bold(),
            white_bold.apply_to(message)
        );
        Ok(())
    }

    /// Print a warning message
    pub fn print_warning(&self, message: &str) -> io::Result<()> {
        use console::Style;
        let (_, warning_style, _, _) = self.get_status_styles();
        let white_bold = Style::new().white().bold();
        println!(
            "{} {}",
            warning_style.apply_to("[!] WARNING:").bold(),
            white_bold.apply_to(message)
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

/// Safely truncate a string to display width, respecting UTF-8 character boundaries
fn safe_truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    // Use char indices to respect UTF-8 boundaries
    let chars: Vec<char> = path.chars().collect();

    if chars.len() <= max_len {
        return path.to_string();
    }

    // Take first 30 chars and last 32 chars
    let prefix_len = 30;
    let suffix_len = 32;

    if chars.len() <= prefix_len + suffix_len {
        return path.to_string();
    }

    let prefix: String = chars.iter().take(prefix_len).collect();
    let suffix: String = chars.iter().skip(chars.len() - suffix_len).collect();

    format!("{}...{}", prefix, suffix)
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
    _theme: &str,
) -> Vec<String> {
    let mut lines = Vec::new();

    // Calculate total scanned size
    let total_scanned: u64 = stats.iter().map(|(_, _, size)| size).sum();
    if total_scanned == 0 {
        use console::Style;
        let white_bold = Style::new().white().bold();
        lines.push(format!("{}", white_bold.apply_to("No data to display")));
        return lines;
    }

    // Use drive size if provided, otherwise use scanned size
    let reference_size = total_drive_size.unwrap_or(total_scanned);

    use console::Style;

    let white_bold = Style::new().white().bold();
    let char = "█";

    // Sort categories by size descending
    let mut sorted_stats: Vec<_> = stats.iter().collect();
    sorted_stats.sort_by(|a, b| b.2.cmp(&a.2));

    // Fixed bar width
    const BAR_WIDTH: usize = 40;

    for (category, count, size) in sorted_stats.iter() {
        let percentage_of_drive = (*size as f64 / reference_size as f64) * 100.0;
        let bar_length = ((*size as f64 / reference_size as f64) * BAR_WIDTH as f64) as usize;

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

        // Apply white bold to text, italicize important numbers
        let line = format!(
            "{} {:<15} {}{} {} {} ({} files, avg: {})",
            char,
            category_label,
            bar,
            " ".repeat(BAR_WIDTH.saturating_sub(bar_length)),
            white_bold
                .apply_to(format!("{:>6.2}%", percentage_of_drive))
                .italic(),
            white_bold
                .apply_to(format!("{:>12}", format_size(*size)))
                .italic(),
            count,
            white_bold.apply_to(format_size(avg_size)).italic()
        );

        lines.push(format!("{}", white_bold.apply_to(line)));
    }

    lines
}

// Helper function to create statistics summary
fn create_statistics_summary(
    stats: &[(String, usize, u64)],
    total_files: usize,
    total_size: u64,
) -> Vec<String> {
    use console::Style;
    let white_bold = Style::new().white().bold();
    let mut lines = Vec::new();

    if total_files == 0 {
        lines.push(format!("{}", white_bold.apply_to("No data to display")));
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

    // Display statistics - italicize important values
    lines.push(format!(
        "{} {}",
        white_bold.apply_to("Average file size:       "),
        white_bold.apply_to(format_size(overall_avg)).italic()
    ));
    lines.push(format!(
        "{} {}",
        white_bold.apply_to("Median file size:        "),
        white_bold.apply_to(format_size(median)).italic()
    ));
    lines.push(format!(
        "{} {}",
        white_bold.apply_to("Total categories:        "),
        white_bold.apply_to(format!("{}", stats.len())).italic()
    ));

    if let Some((cat, count, size)) = largest_category {
        lines.push(format!(
            "{} {} ({}, {} files)",
            white_bold.apply_to("Largest category:        "),
            white_bold.apply_to(cat).italic(),
            white_bold.apply_to(format_size(*size)).italic(),
            white_bold.apply_to(format!("{}", count)).italic()
        ));
    }

    if let Some((cat, count, size)) = smallest_category {
        lines.push(format!(
            "{} {} ({}, {} files)",
            white_bold.apply_to("Smallest category:       "),
            white_bold.apply_to(cat).italic(),
            white_bold.apply_to(format_size(*size)).italic(),
            white_bold.apply_to(format!("{}", count)).italic()
        ));
    }

    if let Some((cat, count, _)) = most_files_category {
        lines.push(format!(
            "{} {} ({} files)",
            white_bold.apply_to("Most files in category:  "),
            white_bold.apply_to(cat).italic(),
            white_bold.apply_to(format!("{}", count)).italic()
        ));
    }

    lines
}

// Helper function to create top 10 largest files leaderboard
fn create_leaderboard(all_files: &[(String, u64, String)]) -> Vec<String> {
    use console::Style;
    let white_bold = Style::new().white().bold();
    let mut lines = Vec::new();

    if all_files.is_empty() {
        lines.push(format!("{}", white_bold.apply_to("No files to display")));
        return lines;
    }

    // Sort by size descending and take top 10
    let mut sorted_files: Vec<_> = all_files.iter().collect();
    sorted_files.sort_by(|a, b| b.1.cmp(&a.1));
    let top_files: Vec<_> = sorted_files.iter().take(10).collect();

    // Header
    lines.push(format!(
        "{}",
        white_bold.apply_to(format!(
            "{:<3} {:<35} {:<12} {:<15}",
            "Rank", "Name", "Size", "Category"
        ))
    ));
    lines.push(format!("{}", white_bold.apply_to("-".repeat(68))));

    // Top 10 files - italicize important data (rank, size)
    for (rank, (name, size, category)) in top_files.iter().enumerate() {
        // Truncate long file names
        let display_name = if name.len() > 35 {
            format!("{}...", &name[..32])
        } else {
            name.to_string()
        };

        let line = format!(
            "{:<3} {:<35} {:<12} {:<15}",
            white_bold.apply_to(format!("{}", rank + 1)).italic(),
            display_name,
            white_bold.apply_to(format_size(*size)).italic(),
            category
        );

        lines.push(format!("{}", white_bold.apply_to(line)));
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
