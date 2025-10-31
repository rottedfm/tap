// src/mount.rs
use dialoguer::{Confirm, theme::ColorfulTheme};
use std::path::{Path, PathBuf};
use std::process::Command;

pub async fn mount_drive_readonly(device: &str) -> color_eyre::Result<PathBuf> {
    // Check if already mounted
    if let Some(existing_mount) = get_mount_point(device)? {
        println!(
            "INFO: Drive already mounted at: {}",
            existing_mount.display()
        );

        if is_mounted_readonly(&existing_mount)? {
            println!("SUCCESS: Drive is mounted read-only");
            return Ok(existing_mount);
        } else {
            println!("WARNING: Drive is mounted READ-WRITE!");
            println!("   For safety, the drive should be remounted read-only.");

            let remount = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Remount as read-only?")
                .default(true)
                .interact()?;

            if !remount {
                println!("WARNING: Continuing with read-write mount (NOT RECOMMENDED)");
                return Ok(existing_mount);
            }

            // Remount read-only
            println!("Remounting {} as read-only...", device);
            let output = Command::new("sudo")
                .args(["mount", "-o", "remount,ro", device])
                .output()?;

            if !output.status.success() {
                eprintln!("ERROR: Failed to remount read-only");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }

            println!("SUCCESS: Remounted as read-only");
            return Ok(existing_mount);
        }
    }

    // Drive not mounted - mount it
    println!("Drive {} is not mounted", device);

    let should_mount = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Mount as read-only?")
        .default(true)
        .interact()?;

    if !should_mount {
        eprintln!("ERROR: Drive must be mounted to proceed");
        std::process::exit(1);
    }

    // Create mount point
    let new_mount_point = PathBuf::from(format!(
        "/mnt/tap_{}",
        device.trim_start_matches("/dev/")
    ));

    println!("Creating mount point: {}", new_mount_point.display());

    let output = Command::new("sudo")
        .args(["mkdir", "-p", new_mount_point.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        eprintln!("ERROR: Failed to create mount point");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    // Mount read-only
    println!(
        "Mounting {} to {} (read-only)...",
        device,
        new_mount_point.display()
    );

    let output = Command::new("sudo")
        .args([
            "mount",
            "-o",
            "ro",
            device,
            new_mount_point.to_str().unwrap(),
        ])
        .output()?;

    if !output.status.success() {
        eprintln!("ERROR: Failed to mount drive");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));

        // Try to detect filesystem and suggest mounting
        println!("\nTROUBLESHOOTING:");
        println!("  1. Check if device exists: lsblk");
        println!("  2. Check filesystem: sudo blkid {}", device);
        println!("  3. Try manual mount: sudo mount -o ro {} /mnt/evidence", device);

        std::process::exit(1);
    }

    println!(
        "SUCCESS: Drive mounted successfully at {}",
        new_mount_point.display()
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

pub fn validate_source_path(drive: &str) -> color_eyre::Result<PathBuf> {
    let path = PathBuf::from(drive);
    if !path.exists() {
        eprintln!("ERROR: Path does not exist: {}", drive);
        std::process::exit(1);
    }

    // Warn if not mounted read-only
    if !is_mounted_readonly(&path)? {
        println!("WARNING: Path is not mounted read-only!");
        println!("   This could potentially modify the evidence.");

        let should_continue = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue anyway?")
            .default(false)
            .interact()?;

        if !should_continue {
            println!("Aborted.");
            std::process::exit(0);
        }
    }

    Ok(path)
}

pub fn unmount_drive(mount_point: &Path, _device: &str) -> color_eyre::Result<()> {
    // Only unmount if it's a mount point we created
    let mount_point_str = mount_point.to_string_lossy();
    if !mount_point_str.starts_with("/mnt/tap_") {
        println!("INFO: Skipping unmount - not a tap-managed mount point");
        return Ok(());
    }

    println!("INFO: Unmounting {}...", mount_point.display());

    let output = Command::new("sudo")
        .args(["umount", mount_point.to_str().unwrap()])
        .output()?;

    if !output.status.success() {
        eprintln!("WARNING: Failed to unmount drive");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err(color_eyre::eyre::eyre!("Failed to unmount drive"));
    }

    println!("SUCCESS: Drive unmounted successfully");

    // Try to remove the mount point directory
    let output = Command::new("sudo")
        .args(["rmdir", mount_point.to_str().unwrap()])
        .output()?;

    if output.status.success() {
        println!("SUCCESS: Mount point removed");
    }

    Ok(())
}
