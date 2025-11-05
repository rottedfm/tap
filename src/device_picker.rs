//! Interactive device selection.
//!
//! This module provides an interactive UI for selecting block devices (partitions)
//! from available system storage, filtering out system partitions and encrypted volumes.

use console::Term;
use dialoguer::Select;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::tui::{BANNER, UI};

#[derive(Debug)]
pub struct BlockDevice {
    pub path: String,
    pub display_name: String,
}

/// Get list of partitions that are part of the Linux system
fn get_linux_system_partitions() -> HashSet<String> {
    let mut system_partitions = HashSet::new();

    // Use findmnt to get all mounted partitions
    if let Ok(output) = Command::new("findmnt")
        .args(&["-n", "-o", "SOURCE"])
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines() {
                let source = line.trim();
                // Skip pseudo-filesystems (tmpfs, devtmpfs, etc.)
                if source.starts_with("/dev/") {
                    system_partitions.insert(source.to_string());
                }
            }
        }
    }

    system_partitions
}

/// Enumerate available block devices from /dev/
pub fn enumerate_block_devices() -> color_eyre::Result<Vec<BlockDevice>> {
    let mut devices = Vec::new();

    // Get Linux system partitions to filter out
    let system_partitions = get_linux_system_partitions();

    // Read /dev/ directory
    let dev_dir = fs::read_dir("/dev")?;

    for entry in dev_dir {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Only look for partitions, not whole disks
        let is_sata_partition = name.starts_with("sd") && name.len() > 3 && name.chars().nth(3).unwrap().is_ascii_digit();  // sda1, sdb2, etc.
        let is_nvme_partition = name.starts_with("nvme") && name.contains("p") && name.chars().last().unwrap().is_ascii_digit();  // nvme0n1p1, etc.
        let is_mmc_partition = name.starts_with("mmcblk") && name.contains("p") && name.chars().last().unwrap().is_ascii_digit();  // mmcblk0p1, etc.
        let is_virtual_partition = name.starts_with("vd") && name.len() > 3 && name.chars().nth(3).unwrap().is_ascii_digit();  // vda1, vdb2, etc.

        if is_sata_partition || is_nvme_partition || is_mmc_partition || is_virtual_partition {
            let path_str = path.to_string_lossy().to_string();

            // Skip if this is a Linux system partition
            if system_partitions.contains(&path_str) {
                continue;
            }

            // Skip if this is an encrypted partition
            if is_encrypted(&path) {
                continue;
            }

            // Get size info if available
            let size_info = get_device_size(&path);
            let display_name = if let Some(size) = size_info {
                format!("{} ({})", path.display(), size)
            } else {
                format!("{}", path.display())
            };

            devices.push(BlockDevice {
                path: path_str,
                display_name,
            });
        }
    }

    // Sort by device name
    devices.sort_by(|a, b| a.path.cmp(&b.path));

    if devices.is_empty() {
        return Err(color_eyre::eyre::eyre!("No removable partitions found. All partitions appear to be part of the Linux system."));
    }

    Ok(devices)
}

/// Check if a device is LUKS encrypted
fn is_encrypted(path: &PathBuf) -> bool {
    use std::process::Command;

    let output = Command::new("lsblk")
        .args(&["-n", "-o", "FSTYPE", path.to_str().unwrap_or("")])
        .output();

    if let Ok(output) = output {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            // Check if any line contains crypto_LUKS
            return stdout.lines().any(|line| line.trim() == "crypto_LUKS");
        }
    }

    false
}

/// Get device size information using lsblk
fn get_device_size(path: &PathBuf) -> Option<String> {
    use std::process::Command;

    let output = Command::new("lsblk")
        .args(&["-b", "-d", "-n", "-o", "SIZE", path.to_str()?])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let size_bytes = String::from_utf8(output.stdout)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;

    Some(human_readable_size(size_bytes))
}

/// Convert bytes to human-readable size
fn human_readable_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Show interactive device picker and return selected device path
pub fn pick_device(theme: &str) -> color_eyre::Result<String> {
    // Clear screen and show banner
    let term = Term::stdout();
    term.clear_screen()?;

    // Get style for theme
    let style = match theme {
        "cyan" => console::Style::new().cyan(),
        "magenta" => console::Style::new().magenta(),
        "yellow" => console::Style::new().yellow(),
        "green" => console::Style::new().green(),
        "red" => console::Style::new().red(),
        "blue" => console::Style::new().blue(),
        "white" => console::Style::new().white(),
        _ => console::Style::new().white(),
    };

    let white_bold = console::Style::new().white().bold();

    println!("{}", style.apply_to(BANNER).bold());
    println!();
    println!("{}", white_bold.apply_to("=".repeat(70)));
    println!("{}", style.apply_to("DEVICE SELECTION").bold());
    println!("{}", white_bold.apply_to("=".repeat(70)));
    println!();
    println!("{}", white_bold.apply_to("Available partitions (excluding system drives):"));
    println!();

    let devices = enumerate_block_devices()?;

    let items: Vec<&str> = devices
        .iter()
        .map(|d| d.display_name.as_str())
        .collect();

    let colorful_theme = UI::get_colorful_theme(theme);
    let selection = Select::with_theme(&colorful_theme)
        .with_prompt("Select a partition")
        .items(&items)
        .default(0)
        .interact()?;

    println!();

    Ok(devices[selection].path.clone())
}
