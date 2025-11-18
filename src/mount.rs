//! Drive mounting and validation.
//!
//! This module handles mounting block devices in read-only mode, validating
//! existing mounts, and safely unmounting drives when operations complete.

use crate::tui::UI;
use dialoguer::Confirm;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Check if a device is a RAID member
fn is_raid_member(device: &str) -> color_eyre::Result<bool> {
    let output = Command::new("blkid")
        .args(["-s", "TYPE", "-o", "value", device])
        .output()?;

    if output.status.success() {
        let fs_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
        // Common RAID member types - includes Intel Software RAID (ISW)
        if fs_type.contains("raid_member")
            || fs_type.contains("linux_raid_member")
            || fs_type.contains("isw_raid_member")
        {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Check if a device is an Intel Software RAID (ISW) member
fn is_isw_raid_member(device: &str) -> color_eyre::Result<bool> {
    let output = Command::new("blkid")
        .args(["-s", "TYPE", "-o", "value", device])
        .output()?;

    if output.status.success() {
        let fs_type = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if fs_type.contains("isw_raid_member") {
            return Ok(true);
        }
    }

    Ok(false)
}

/// RAID array metadata extracted from mdadm --examine
#[derive(Debug)]
struct RaidMetadata {
    uuid: Option<String>,
    raid_level: Option<String>,
    raid_devices: Option<u32>,
    total_devices: Option<u32>,
    name: Option<String>,
}

/// Intel RAID (dmraid) metadata
#[derive(Debug)]
struct DmraidMetadata {
    raid_set_name: Option<String>,
    raid_type: Option<String>,
    status: Option<String>,
    #[allow(dead_code)]
    total_devices: Option<u32>,
}

/// Get Intel RAID (dmraid) information for a device
fn get_dmraid_info(device: &str) -> color_eyre::Result<Option<DmraidMetadata>> {
    // Use dmraid to discover RAID sets
    let output = Command::new("sudo").args(["dmraid", "-s", "-c"]).output()?;

    if output.status.success() {
        let _info = String::from_utf8_lossy(&output.stdout);
        let mut metadata = DmraidMetadata {
            raid_set_name: None,
            raid_type: None,
            status: None,
            total_devices: None,
        };

        // Get detailed info with dmraid -r to see if this device is part of a RAID set
        let detail_output = Command::new("sudo").args(["dmraid", "-r"]).output()?;

        if detail_output.status.success() {
            let detail_info = String::from_utf8_lossy(&detail_output.stdout);
            let device_short = device.trim_start_matches("/dev/");

            // Check if this device is listed in dmraid output
            if detail_info.contains(device_short) {
                // Get RAID set info
                let sets_output = Command::new("sudo").args(["dmraid", "-s"]).output()?;

                if sets_output.status.success() {
                    let sets_info = String::from_utf8_lossy(&sets_output.stdout);

                    for line in sets_info.lines() {
                        if line.starts_with("name") && line.contains(':') {
                            if let Some(name) = line.split(':').nth(1) {
                                metadata.raid_set_name = Some(name.trim().to_string());
                            }
                        } else if line.starts_with("type") && line.contains(':') {
                            if let Some(raid_type) = line.split(':').nth(1) {
                                metadata.raid_type = Some(raid_type.trim().to_string());
                            }
                        } else if line.starts_with("status") && line.contains(':') {
                            if let Some(status) = line.split(':').nth(1) {
                                metadata.status = Some(status.trim().to_string());
                            }
                        }
                    }
                }

                return Ok(Some(metadata));
            }
        }
    }

    Ok(None)
}

/// Get RAID array information for a device
fn get_raid_array_info(device: &str) -> color_eyre::Result<Option<RaidMetadata>> {
    // Check if mdadm can examine this device
    let output = Command::new("sudo")
        .args(["mdadm", "--examine", device])
        .output()?;

    if output.status.success() {
        let info = String::from_utf8_lossy(&output.stdout);
        let mut metadata = RaidMetadata {
            uuid: None,
            raid_level: None,
            raid_devices: None,
            total_devices: None,
            name: None,
        };

        // Parse the mdadm output
        for line in info.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("UUID") && trimmed.contains(':') {
                if let Some(uuid) = trimmed.split(':').nth(1) {
                    metadata.uuid = Some(uuid.trim().to_string());
                }
            } else if trimmed.starts_with("Raid Level") && trimmed.contains(':') {
                if let Some(level) = trimmed.split(':').nth(1) {
                    metadata.raid_level = Some(level.trim().to_string());
                }
            } else if trimmed.starts_with("Raid Devices") && trimmed.contains(':') {
                if let Some(devices) = trimmed.split(':').nth(1) {
                    metadata.raid_devices = devices.trim().parse().ok();
                }
            } else if trimmed.starts_with("Total Devices") && trimmed.contains(':') {
                if let Some(devices) = trimmed.split(':').nth(1) {
                    metadata.total_devices = devices.trim().parse().ok();
                }
            } else if (trimmed.starts_with("Name") || trimmed.starts_with("MD_DEVNAME"))
                && trimmed.contains(':')
            {
                if let Some(name) = trimmed.split(':').nth(1) {
                    metadata.name = Some(name.trim().to_string());
                }
            }
        }

        return Ok(Some(metadata));
    }

    Ok(None)
}

/// Activate Intel RAID array using dmraid
fn activate_dmraid_array(
    device: &str,
    metadata: &DmraidMetadata,
    theme: &str,
) -> color_eyre::Result<Option<String>> {
    let _colorful_theme = UI::get_colorful_theme(theme);
    let (info_style, _warning_style, error_style, success_style) =
        UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    println!(
        "{} {}",
        info_style.apply_to("[*]").bold(),
        white_bold.apply_to("Detected Intel RAID (ISW) member - attempting to activate array...")
    );

    // Display RAID metadata to user
    println!();
    println!("{}", white_bold.apply_to("Intel RAID Array Information:"));
    if let Some(ref name) = metadata.raid_set_name {
        println!(
            "{}",
            white_bold.apply_to(format!("  RAID Set Name: {}", name))
        );
    }
    if let Some(ref raid_type) = metadata.raid_type {
        println!(
            "{}",
            white_bold.apply_to(format!("  RAID Type: {}", raid_type))
        );
    }
    if let Some(ref status) = metadata.status {
        println!("{}", white_bold.apply_to(format!("  Status: {}", status)));
    }
    println!();

    // Activate the RAID array using dmraid
    println!(
        "{} {}",
        info_style.apply_to("[*]").bold(),
        white_bold.apply_to("Activating Intel RAID array with dmraid...")
    );

    let output = Command::new("sudo").args(["dmraid", "-ay"]).output()?;

    if output.status.success() {
        println!(
            "{} {}",
            success_style.apply_to("[✓]").bold(),
            white_bold.apply_to("Intel RAID array activated successfully")
        );

        // Find the activated device mapper device
        return find_dmraid_device(device, metadata, theme);
    } else {
        println!(
            "{} {}",
            error_style.apply_to("[!]").bold(),
            white_bold.apply_to("Failed to activate Intel RAID array")
        );
        println!(
            "{}",
            white_bold.apply_to(String::from_utf8_lossy(&output.stderr))
        );
    }

    Ok(None)
}

/// Find the device mapper device for the activated dmraid array
fn find_dmraid_device(
    #[allow(unused_variables)] device: &str,
    metadata: &DmraidMetadata,
    theme: &str,
) -> color_eyre::Result<Option<String>> {
    let (info_style, warning_style, _, _) = UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    // List device mapper devices
    let output = Command::new("ls").args(["-1", "/dev/mapper"]).output()?;

    if output.status.success() {
        let devices = String::from_utf8_lossy(&output.stdout);

        // Look for the RAID set name in the device mapper devices
        if let Some(ref raid_name) = metadata.raid_set_name {
            for line in devices.lines() {
                let line = line.trim();
                // dmraid creates devices like /dev/mapper/isw_xxxxx_Volume0
                if line.contains(raid_name) || line.starts_with("isw_") {
                    let dm_device = format!("/dev/mapper/{}", line);
                    println!(
                        "{} {}",
                        info_style.apply_to("[*]").bold(),
                        white_bold.apply_to(format!("Intel RAID device: {}", dm_device))
                    );
                    return Ok(Some(dm_device));
                }
            }
        }

        // If we can't match by name, show all mapper devices
        println!(
            "{} {}",
            warning_style.apply_to("[!]").bold(),
            white_bold.apply_to("Available device mapper devices:")
        );
        for line in devices.lines() {
            let line = line.trim();
            if !line.is_empty() && line != "control" {
                println!("{}", white_bold.apply_to(format!("  /dev/mapper/{}", line)));
            }
        }
    }

    Err(color_eyre::eyre::eyre!(
        "Could not find activated Intel RAID device in /dev/mapper"
    ))
}

/// Assemble a RAID array from a member device
fn assemble_raid_array(
    device: &str,
    metadata: &RaidMetadata,
    theme: &str,
) -> color_eyre::Result<Option<String>> {
    let colorful_theme = UI::get_colorful_theme(theme);
    let (info_style, warning_style, error_style, success_style) =
        UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    println!(
        "{} {}",
        info_style.apply_to("[*]").bold(),
        white_bold.apply_to("Detected RAID array member - attempting to assemble array...")
    );

    // First try to assemble normally with scan
    let output = Command::new("sudo")
        .args(["mdadm", "--assemble", "--scan", "--readonly"])
        .output()?;

    if output.status.success() {
        println!(
            "{} {}",
            success_style.apply_to("[✓]").bold(),
            white_bold.apply_to("RAID array assembled successfully")
        );

        return find_assembled_array(device, theme);
    }

    // Normal assembly failed - check if array is degraded
    println!(
        "{} {}",
        warning_style.apply_to("[!]").bold(),
        white_bold.apply_to("Normal RAID assembly failed")
    );

    // Display RAID metadata to user
    println!();
    println!("{}", white_bold.apply_to("RAID Array Information:"));
    if let Some(ref level) = metadata.raid_level {
        println!(
            "{}",
            white_bold.apply_to(format!("  RAID Level: {}", level))
        );
    }
    if let Some(ref uuid) = metadata.uuid {
        println!("{}", white_bold.apply_to(format!("  UUID: {}", uuid)));
    }
    if let Some(ref name) = metadata.name {
        println!("{}", white_bold.apply_to(format!("  Name: {}", name)));
    }
    if let Some(raid_devices) = metadata.raid_devices {
        println!(
            "{}",
            white_bold.apply_to(format!("  Expected devices: {}", raid_devices))
        );
    }
    if let Some(total_devices) = metadata.total_devices {
        println!(
            "{}",
            white_bold.apply_to(format!("  Total devices: {}", total_devices))
        );
    }
    println!();

    // Check if this might be a degraded array
    let is_likely_degraded = match (metadata.raid_devices, metadata.total_devices) {
        (Some(expected), Some(total)) => total < expected,
        _ => true, // Unknown, assume degraded
    };

    if is_likely_degraded {
        println!(
            "{} {}",
            warning_style.apply_to("[!] WARNING:").bold(),
            white_bold.apply_to("This appears to be a DEGRADED RAID array!")
        );
        println!(
            "{}",
            white_bold.apply_to("  - Not all array members are present")
        );
        println!(
            "{}",
            white_bold.apply_to("  - Depending on RAID level, data may be incomplete or corrupted")
        );
        println!(
            "{}",
            white_bold.apply_to("  - Force-assembling may allow read-only access to partial data")
        );
        println!();

        let should_force = Confirm::with_theme(&colorful_theme)
            .with_prompt("Attempt to force-assemble degraded RAID array? (read-only)")
            .default(false)
            .interact()?;

        if !should_force {
            println!("{}", white_bold.apply_to("RAID assembly aborted by user."));
            return Ok(None);
        }

        // Try force assembly
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold.apply_to("Attempting force assembly of degraded array...")
        );

        // Use UUID if available, otherwise try with device
        let force_output = if let Some(ref uuid) = metadata.uuid {
            Command::new("sudo")
                .args([
                    "mdadm",
                    "--assemble",
                    "--force",
                    "--readonly",
                    "--uuid",
                    uuid,
                    "/dev/md127",
                ])
                .output()?
        } else {
            Command::new("sudo")
                .args([
                    "mdadm",
                    "--assemble",
                    "--force",
                    "--readonly",
                    "/dev/md127",
                    device,
                ])
                .output()?
        };

        if force_output.status.success() {
            println!(
                "{} {}",
                success_style.apply_to("[✓]").bold(),
                white_bold.apply_to("Degraded RAID array assembled successfully (read-only)")
            );
            println!(
                "{} {}",
                warning_style.apply_to("[!]").bold(),
                white_bold.apply_to("Note: Array is degraded - some data may be inaccessible")
            );

            return find_assembled_array(device, theme);
        } else {
            println!(
                "{} {}",
                error_style.apply_to("[!]").bold(),
                white_bold.apply_to("Failed to force-assemble RAID array")
            );
            println!(
                "{}",
                white_bold.apply_to(String::from_utf8_lossy(&force_output.stderr))
            );
        }
    }

    Ok(None)
}

/// Find the MD device that was assembled for the given physical device
fn find_assembled_array(device: &str, theme: &str) -> color_eyre::Result<Option<String>> {
    let (info_style, warning_style, _, _) = UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    // Find the assembled array device
    let list_output = Command::new("cat").arg("/proc/mdstat").output()?;

    if list_output.status.success() {
        let mdstat = String::from_utf8_lossy(&list_output.stdout);
        // Parse mdstat to find array that contains this device
        for line in mdstat.lines() {
            if line.starts_with("md") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(md_name) = parts.first() {
                    let md_device = format!("/dev/{}", md_name);
                    // Check if this array contains our device
                    let detail_output = Command::new("sudo")
                        .args(["mdadm", "--detail", &md_device])
                        .output()?;

                    if detail_output.status.success() {
                        let detail = String::from_utf8_lossy(&detail_output.stdout);
                        let device_short = device.trim_start_matches("/dev/");
                        if detail.contains(device_short) {
                            println!(
                                "{} {}",
                                info_style.apply_to("[*]").bold(),
                                white_bold.apply_to(format!("RAID array device: {}", md_device))
                            );
                            return Ok(Some(md_device));
                        }
                    }
                }
            }
        }
    }

    // If we can't find the specific array, list all arrays
    println!(
        "{} {}",
        warning_style.apply_to("[!]").bold(),
        white_bold.apply_to("Array assembled but couldn't determine device name")
    );
    println!("{}", white_bold.apply_to("Available RAID arrays:"));

    let _ = Command::new("sh")
        .arg("-c")
        .arg("cat /proc/mdstat | grep '^md'")
        .status();

    Err(color_eyre::eyre::eyre!(
        "Please manually specify the RAID array device (e.g., /dev/md0)"
    ))
}

pub async fn mount_drive_readonly(device: &str, theme: &str) -> color_eyre::Result<PathBuf> {
    let colorful_theme = UI::get_colorful_theme(theme);
    let (info_style, warning_style, _, success_style) = UI::get_static_status_styles(theme);
    let white_bold = console::Style::new().white().bold();

    // Check if this is a RAID member and assemble/activate if needed
    let actual_device = if is_raid_member(device)? {
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold.apply_to("Detected RAID array member")
        );

        // Check if this is an Intel Software RAID (ISW) member
        if is_isw_raid_member(device)? {
            println!(
                "{} {}",
                info_style.apply_to("[*]").bold(),
                white_bold.apply_to("Detected Intel Software RAID (ISW) member")
            );

            if let Some(metadata) = get_dmraid_info(device)? {
                match activate_dmraid_array(device, &metadata, theme)? {
                    Some(dm_device) => dm_device,
                    None => {
                        let (_, _, error_style, _) = UI::get_static_status_styles(theme);
                        println!(
                            "{} {}",
                            error_style.apply_to("[!] ERROR:").bold(),
                            white_bold.apply_to("Failed to activate Intel RAID array")
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                let (_, _, error_style, _) = UI::get_static_status_styles(theme);
                println!(
                    "{} {}",
                    error_style.apply_to("[!] ERROR:").bold(),
                    white_bold.apply_to("Could not read Intel RAID metadata")
                );
                std::process::exit(1);
            }
        } else {
            // Handle standard Linux RAID with mdadm
            if let Some(metadata) = get_raid_array_info(device)? {
                if let Some(ref name) = metadata.name {
                    println!(
                        "{} {}",
                        info_style.apply_to("[*]").bold(),
                        white_bold.apply_to(format!("RAID array name: {}", name))
                    );
                }
                if let Some(ref level) = metadata.raid_level {
                    println!(
                        "{} {}",
                        info_style.apply_to("[*]").bold(),
                        white_bold.apply_to(format!("RAID level: {}", level))
                    );
                }

                match assemble_raid_array(device, &metadata, theme)? {
                    Some(md_device) => md_device,
                    None => {
                        let (_, _, error_style, _) = UI::get_static_status_styles(theme);
                        println!(
                            "{} {}",
                            error_style.apply_to("[!] ERROR:").bold(),
                            white_bold.apply_to("Failed to assemble RAID array")
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                let (_, _, error_style, _) = UI::get_static_status_styles(theme);
                println!(
                    "{} {}",
                    error_style.apply_to("[!] ERROR:").bold(),
                    white_bold.apply_to("Could not read RAID metadata")
                );
                std::process::exit(1);
            }
        }
    } else {
        device.to_string()
    };

    let device = actual_device.as_str();

    // Check if already mounted
    if let Some(existing_mount) = get_mount_point(device)? {
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold.apply_to(format!(
                "Drive already mounted at: {}",
                existing_mount.display()
            ))
        );

        if is_mounted_readonly(&existing_mount)? {
            println!(
                "{} {}",
                success_style.apply_to("[✓]").bold(),
                white_bold.apply_to("Drive is mounted read-only")
            );
            return Ok(existing_mount);
        } else {
            println!(
                "{} {}",
                warning_style.apply_to("[!] WARNING:").bold(),
                white_bold.apply_to("Drive is mounted READ-WRITE!")
            );
            println!(
                "{}",
                white_bold.apply_to("   For safety, the drive should be remounted read-only.")
            );

            let remount = Confirm::with_theme(&colorful_theme)
                .with_prompt("Remount as read-only?")
                .default(true)
                .interact()?;

            if !remount {
                println!(
                    "{} {}",
                    warning_style.apply_to("[!] WARNING:").bold(),
                    white_bold.apply_to("Continuing with read-write mount (NOT RECOMMENDED)")
                );
                return Ok(existing_mount);
            }

            // Remount read-only
            println!(
                "{} {}",
                info_style.apply_to("[*]").bold(),
                white_bold.apply_to(format!("Remounting {} as read-only...", device))
            );
            let output = Command::new("sudo")
                .args(["mount", "-o", "remount,ro", device])
                .output()?;

            if !output.status.success() {
                let (_, _, error_style, _) = UI::get_static_status_styles(theme);
                println!(
                    "{} {}",
                    error_style.apply_to("[!] ERROR:").bold(),
                    white_bold.apply_to("Failed to remount read-only")
                );
                println!(
                    "{}",
                    white_bold.apply_to(String::from_utf8_lossy(&output.stderr))
                );
                std::process::exit(1);
            }

            println!(
                "{} {}",
                success_style.apply_to("[✓]").bold(),
                white_bold.apply_to("Remounted as read-only")
            );
            return Ok(existing_mount);
        }
    }

    // Drive not mounted - mount it
    println!(
        "{} {}",
        info_style.apply_to("[*]").bold(),
        white_bold.apply_to(format!("Drive {} is not mounted", device))
    );

    let should_mount = Confirm::with_theme(&colorful_theme)
        .with_prompt("Mount as read-only?")
        .default(true)
        .interact()?;

    if !should_mount {
        let (_, _, error_style, _) = UI::get_static_status_styles(theme);
        println!(
            "{} {}",
            error_style.apply_to("[!] ERROR:").bold(),
            white_bold.apply_to("Drive must be mounted to proceed")
        );
        std::process::exit(1);
    }

    // Create mount point
    let new_mount_point = PathBuf::from(format!("/mnt/tap_{}", device.trim_start_matches("/dev/")));

    println!(
        "{} {}",
        info_style.apply_to("[*]").bold(),
        white_bold.apply_to(format!(
            "Creating mount point: {}",
            new_mount_point.display()
        ))
    );

    let output = Command::new("sudo")
        .args(["mkdir", "-p", new_mount_point.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        let (_, _, error_style, _) = UI::get_static_status_styles(theme);
        println!(
            "{} {}",
            error_style.apply_to("[!] ERROR:").bold(),
            white_bold.apply_to("Failed to create mount point")
        );
        println!(
            "{}",
            white_bold.apply_to(String::from_utf8_lossy(&output.stderr))
        );
        std::process::exit(1);
    }

    // Detect filesystem type
    let fs_type = get_filesystem_type(device)?;
    let use_ntfs3g = fs_type.as_ref().map(|t| t == "ntfs").unwrap_or(false);

    if use_ntfs3g {
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold
                .apply_to("Detected NTFS filesystem - using ntfs-3g for better compatibility")
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
        println!(
            "{} {}",
            error_style.apply_to("[!] ERROR:").bold(),
            white_bold.apply_to("Failed to mount drive")
        );
        println!(
            "{}",
            white_bold.apply_to(String::from_utf8_lossy(&output.stderr))
        );

        // Try to detect filesystem and suggest mounting
        println!();
        println!("{}", white_bold.apply_to("TROUBLESHOOTING:"));
        println!(
            "{}",
            white_bold.apply_to("  1. Check if device exists: lsblk")
        );
        println!(
            "{}",
            white_bold.apply_to(format!("  2. Check filesystem: sudo blkid {}", device))
        );
        println!(
            "{}",
            white_bold.apply_to(format!(
                "  3. Try manual mount: sudo mount -o ro {} /mnt/evidence",
                device
            ))
        );
        if use_ntfs3g {
            println!(
                "{}",
                white_bold.apply_to("  4. Ensure ntfs-3g is installed: which ntfs-3g")
            );
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
        println!(
            "{} {}",
            error_style.apply_to("[!] ERROR:").bold(),
            white_bold.apply_to(format!("Path does not exist: {}", drive))
        );
        std::process::exit(1);
    }

    // Warn if not mounted read-only
    if !is_mounted_readonly(&path)? {
        println!(
            "{} {}",
            warning_style.apply_to("[!] WARNING:").bold(),
            white_bold.apply_to("Path is not mounted read-only!")
        );
        println!(
            "{}",
            white_bold.apply_to("   This could potentially modify the evidence.")
        );

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
        println!(
            "{} {}",
            info_style.apply_to("[*]").bold(),
            white_bold.apply_to("Skipping unmount - not a tap-managed mount point")
        );
        return Ok(());
    }

    println!(
        "{} {}",
        info_style.apply_to("[*]").bold(),
        white_bold.apply_to(format!("Unmounting {}...", mount_point.display()))
    );

    let output = Command::new("sudo")
        .args(["umount", mount_point.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        println!(
            "{} {}",
            warning_style.apply_to("[!] WARNING:").bold(),
            white_bold.apply_to("Failed to unmount drive")
        );
        println!(
            "{}",
            white_bold.apply_to(String::from_utf8_lossy(&output.stderr))
        );
        return Err(color_eyre::eyre::eyre!("Failed to unmount drive"));
    }

    println!(
        "{} {}",
        success_style.apply_to("[✓]").bold(),
        white_bold.apply_to("Drive unmounted successfully")
    );

    // Try to remove the mount point directory
    let output = Command::new("sudo")
        .args(["rmdir", mount_point.to_str().unwrap()])
        .output()?;

    if output.status.success() {
        println!(
            "{} {}",
            success_style.apply_to("[✓]").bold(),
            white_bold.apply_to("Mount point removed")
        );
    }

    Ok(())
}
