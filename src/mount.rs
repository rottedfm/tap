//! Drive mounting and validation.
//!
//! This module handles mounting block devices in read-only mode, validating
//! existing mounts, and safely unmounting drives when operations complete.

use dialoguer::Confirm;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::tui::UI;

/// Detect the filesystem type of a device
fn get_filesystem_type(device: &str) -> color_eyre::Result<Option<String>> {
    let output = Command::new("blkid")
        .args(["-s", "TYPE", "-o", "value", device])
        .output()?;

    if output.status.success() {
        let fs_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !fs_type.is_empty() {
            return Ok(Some(fs_type));
        }
    }

    Ok(None)
}

pub async fn mount_drive_readonly(device: &str, theme: &str) -> color_eyre::Result<PathBuf> {
    let colorful_theme = UI::get_colorful_theme(theme);
    let (info_style, warning_style, _, success_style) = UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    // Check if already mounted
    if let Some(existing_mount) = get_mount_point(device)? {
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold.apply_to(format!("Drive already mounted at: {}", existing_mount.display()))
        );

        if is_mounted_readonly(&existing_mount)? {
            println!("{} {}", success_style.apply_to("[✓]").bold(), white_bold.apply_to("Drive is mounted read-only"));
            return Ok(existing_mount);
        } else {
            println!("{} {}", warning_style.apply_to("[!] WARNING:").bold(), white_bold.apply_to("Drive is mounted READ-WRITE!"));
            println!("{}", white_bold.apply_to("   For safety, the drive should be remounted read-only."));

            let remount = Confirm::with_theme(&colorful_theme)
                .with_prompt("Remount as read-only?")
                .default(true)
                .interact()?;

            if !remount {
                println!("{} {}", warning_style.apply_to("[!] WARNING:").bold(), white_bold.apply_to("Continuing with read-write mount (NOT RECOMMENDED)"));
                return Ok(existing_mount);
            }

            // Remount read-only
            println!("{} {}", info_style.apply_to("[*]").bold(), white_bold.apply_to(format!("Remounting {} as read-only...", device)));
            let output = Command::new("sudo")
                .args(["mount", "-o", "remount,ro", device])
                .output()?;

            if !output.status.success() {
                let (_, _, error_style, _) = UI::get_static_status_styles(theme);
                println!("{} {}", error_style.apply_to("[!] ERROR:").bold(), white_bold.apply_to("Failed to remount read-only"));
                println!("{}", white_bold.apply_to(String::from_utf8_lossy(&output.stderr)));
                std::process::exit(1);
            }

            println!("{} {}", success_style.apply_to("[✓]").bold(), white_bold.apply_to("Remounted as read-only"));
            return Ok(existing_mount);
        }
    }

    // Drive not mounted - mount it
    println!("{} {}", info_style.apply_to("[*]").bold(), white_bold.apply_to(format!("Drive {} is not mounted", device)));

    let should_mount = Confirm::with_theme(&colorful_theme)
        .with_prompt("Mount as read-only?")
        .default(true)
        .interact()?;

    if !should_mount {
        let (_, _, error_style, _) = UI::get_static_status_styles(theme);
        println!("{} {}", error_style.apply_to("[!] ERROR:").bold(), white_bold.apply_to("Drive must be mounted to proceed"));
        std::process::exit(1);
    }

    // Create mount point
    let new_mount_point = PathBuf::from(format!(
        "/mnt/tap_{}",
        device.trim_start_matches("/dev/")
    ));

    println!("{} {}", info_style.apply_to("[*]").bold(), white_bold.apply_to(format!("Creating mount point: {}", new_mount_point.display())));

    let output = Command::new("sudo")
        .args(["mkdir", "-p", new_mount_point.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        let (_, _, error_style, _) = UI::get_static_status_styles(theme);
        println!("{} {}", error_style.apply_to("[!] ERROR:").bold(), white_bold.apply_to("Failed to create mount point"));
        println!("{}", white_bold.apply_to(String::from_utf8_lossy(&output.stderr)));
        std::process::exit(1);
    }

    // Detect filesystem type
    let fs_type = get_filesystem_type(device)?;
    let use_ntfs3g = fs_type.as_ref().map(|t| t == "ntfs").unwrap_or(false);

    if use_ntfs3g {
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold.apply_to("Detected NTFS filesystem - using ntfs-3g for better compatibility")
        );
    }

    // Mount read-only
    println!(
        "{} {}",
        info_style.apply_to("[*]").bold(),
        white_bold.apply_to(format!(
            "Mounting {} to {} (read-only)...",
            device,
            new_mount_point.display()
        ))
    );

    let output = if use_ntfs3g {
        // Use ntfs-3g for NTFS filesystems
        Command::new("sudo")
            .args([
                "ntfs-3g",
                "-o",
                "ro",
                device,
                new_mount_point.to_str().unwrap(),
            ])
            .output()?
    } else {
        // Use regular mount for other filesystems
        Command::new("sudo")
            .args([
                "mount",
                "-o",
                "ro",
                device,
                new_mount_point.to_str().unwrap(),
            ])
            .output()?
    };

    if !output.status.success() {
        let (_, _, error_style, _) = UI::get_static_status_styles(theme);
        println!("{} {}", error_style.apply_to("[!] ERROR:").bold(), white_bold.apply_to("Failed to mount drive"));
        println!("{}", white_bold.apply_to(String::from_utf8_lossy(&output.stderr)));

        // Try to detect filesystem and suggest mounting
        println!();
        println!("{}", white_bold.apply_to("TROUBLESHOOTING:"));
        println!("{}", white_bold.apply_to("  1. Check if device exists: lsblk"));
        println!("{}", white_bold.apply_to(format!("  2. Check filesystem: sudo blkid {}", device)));
        println!("{}", white_bold.apply_to(format!("  3. Try manual mount: sudo mount -o ro {} /mnt/evidence", device)));
        if use_ntfs3g {
            println!("{}", white_bold.apply_to("  4. Ensure ntfs-3g is installed: which ntfs-3g"));
        }

        std::process::exit(1);
    }

    println!(
        "{} {}",
        success_style.apply_to("[✓]").bold(),
        white_bold.apply_to(format!(
            "Drive mounted successfully at {}",
            new_mount_point.display()
        ))
    );

    Ok(new_mount_point)
}

pub fn get_mount_point(device: &str) -> color_eyre::Result<Option<PathBuf>> {
    let output = Command::new("findmnt")
        .args(["-n", "-o", "TARGET", device])
        .output()?;

    if output.status.success() {
        let mount_point_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !mount_point_str.is_empty() {
            return Ok(Some(PathBuf::from(mount_point_str)));
        }
    }

    Ok(None)
}

pub fn is_mounted_readonly(path: &Path) -> color_eyre::Result<bool> {
    let output = Command::new("findmnt")
        .args(["-n", "-o", "OPTIONS", path.to_str().unwrap()])
        .output()?;

    if output.status.success() {
        let options = String::from_utf8_lossy(&output.stdout);
        // Check if 'ro' is in the mount options
        return Ok(options.split(',').any(|opt| opt.trim() == "ro"));
    }

    Ok(false)
}

pub fn validate_source_path(drive: &str, theme: &str) -> color_eyre::Result<PathBuf> {
    let colorful_theme = UI::get_colorful_theme(theme);
    let (_, warning_style, error_style, _) = UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    let path = PathBuf::from(drive);
    if !path.exists() {
        println!("{} {}", error_style.apply_to("[!] ERROR:").bold(), white_bold.apply_to(format!("Path does not exist: {}", drive)));
        std::process::exit(1);
    }

    // Warn if not mounted read-only
    if !is_mounted_readonly(&path)? {
        println!("{} {}", warning_style.apply_to("[!] WARNING:").bold(), white_bold.apply_to("Path is not mounted read-only!"));
        println!("{}", white_bold.apply_to("   This could potentially modify the evidence."));

        let should_continue = Confirm::with_theme(&colorful_theme)
            .with_prompt("Continue anyway?")
            .default(false)
            .interact()?;

        if !should_continue {
            println!("{}", white_bold.apply_to("Aborted."));
            std::process::exit(0);
        }
    }

    Ok(path)
}

pub fn unmount_drive(mount_point: &Path, _device: &str, theme: &str) -> color_eyre::Result<()> {
    let (info_style, warning_style, _, success_style) = UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    // Only unmount if it's a mount point we created
    let mount_point_str = mount_point.to_string_lossy();
    if !mount_point_str.starts_with("/mnt/tap_") {
        println!("{} {}", info_style.apply_to("[*]").bold(), white_bold.apply_to("Skipping unmount - not a tap-managed mount point"));
        return Ok(());
    }

    println!("{} {}", info_style.apply_to("[*]").bold(), white_bold.apply_to(format!("Unmounting {}...", mount_point.display())));

    let output = Command::new("sudo")
        .args(["umount", mount_point.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        println!("{} {}", warning_style.apply_to("[!] WARNING:").bold(), white_bold.apply_to("Failed to unmount drive"));
        println!("{}", white_bold.apply_to(String::from_utf8_lossy(&output.stderr)));
        return Err(color_eyre::eyre::eyre!("Failed to unmount drive"));
    }

    println!("{} {}", success_style.apply_to("[✓]").bold(), white_bold.apply_to("Drive unmounted successfully"));

    // Try to remove the mount point directory
    let output = Command::new("sudo")
        .args(["rmdir", mount_point.to_str().unwrap()])
        .output()?;

    if output.status.success() {
        println!("{} {}", success_style.apply_to("[✓]").bold(), white_bold.apply_to("Mount point removed"));
    }

    Ok(())
}

