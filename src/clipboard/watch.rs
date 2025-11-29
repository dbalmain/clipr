use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

/// Start watching clipboard for text changes
/// Spawns detached background process: `wl-paste --type text --watch clipr store --type text`
/// Uses process_group(0) to create a new process group, making it independent of the parent
pub fn start_text_watcher() -> Result<()> {
    log::info!("Starting text clipboard watcher");

    // Get the path to the current executable
    let clipr_path = std::env::current_exe()
        .context("Failed to get current executable path")?;

    // Open /dev/null for stdout/stderr
    let dev_null = OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .context("Failed to open /dev/null")?;

    // Spawn wl-paste --type text --watch <clipr> store-text
    // process_group(0) creates a new process group, detaching it from the parent's session
    Command::new("wl-paste")
        .arg("--type")
        .arg("text")
        .arg("--watch")
        .arg(&clipr_path)
        .arg("store-text")
        .stdin(Stdio::null())
        .stdout(dev_null.try_clone()?)
        .stderr(dev_null)
        .process_group(0) // Create new process group (detached)
        .spawn()
        .context("Failed to spawn text clipboard watcher")?;

    log::info!("Text clipboard watcher started in background");
    Ok(())
}

/// Start watching clipboard for image changes
/// Spawns detached background process: `wl-paste --type image/png --watch clipr store --type image`
/// Uses process_group(0) to create a new process group, making it independent of the parent
pub fn start_image_watcher() -> Result<()> {
    log::info!("Starting image clipboard watcher");

    // Get the path to the current executable
    let clipr_path = std::env::current_exe()
        .context("Failed to get current executable path")?;

    // Open /dev/null for stdout/stderr
    let dev_null = OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .context("Failed to open /dev/null")?;

    // Spawn wl-paste --type image/png --watch <clipr> store-image
    // process_group(0) creates a new process group, detaching it from the parent's session
    Command::new("wl-paste")
        .arg("--type")
        .arg("image/png")
        .arg("--watch")
        .arg(&clipr_path)
        .arg("store-image")
        .stdin(Stdio::null())
        .stdout(dev_null.try_clone()?)
        .stderr(dev_null)
        .process_group(0) // Create new process group (detached)
        .spawn()
        .context("Failed to spawn image clipboard watcher")?;

    log::info!("Image clipboard watcher started in background");
    Ok(())
}
