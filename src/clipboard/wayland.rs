use anyhow::{Context, Result, anyhow};
use std::process::{Command, Stdio};

use super::backend::ClipboardBackend;

/// Wayland clipboard backend using wl-clipboard tools
/// Requires wl-copy to be installed
/// Write-only: used to copy selected clips back to clipboard
pub struct WaylandBackend;

impl WaylandBackend {
    /// Create a new Wayland clipboard backend
    pub fn new() -> Result<Self> {
        // Verify wl-copy is available
        Command::new("wl-copy")
            .arg("--version")
            .output()
            .context("wl-copy not found. Install wl-clipboard package")?;

        log::debug!("WaylandBackend initialized successfully");
        Ok(WaylandBackend)
    }
}

impl ClipboardBackend for WaylandBackend {
    fn write_text(&self, text: &str) -> Result<()> {
        let mut child = Command::new("wl-copy")
            .arg("--type")
            .arg("text/plain")
            .arg(text)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn wl-copy")?;

        let status = child.wait().context("Failed to wait for wl-copy")?;

        if !status.success() {
            return Err(anyhow!("wl-copy failed with status: {}", status));
        }

        log::debug!("Wrote {} bytes text to clipboard", text.len());
        Ok(())
    }

    fn write_image(&self, data: &[u8]) -> Result<()> {
        let mut child = Command::new("wl-copy")
            .arg("--type")
            .arg("image/png")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn wl-copy for image")?;

        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(data)
                .context("Failed to write image to wl-copy stdin")?;
        }

        let status = child.wait().context("Failed to wait for wl-copy")?;

        if !status.success() {
            return Err(anyhow!("wl-copy failed with status: {}", status));
        }

        log::debug!("Wrote {} bytes image to clipboard", data.len());
        Ok(())
    }

    fn paste_from_clipboard(&self, delay_ms: u64) -> Result<()> {
        // Spawn detached background process to simulate Ctrl-V after delay
        let cmd = format!(
            "sleep {} && exec wtype -M ctrl v -m ctrl",
            delay_ms as f64 / 1000.0
        );

        Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .spawn()
            .context("Failed to spawn wtype for Ctrl-V. Make sure wtype is installed.")?;

        log::debug!("Scheduled Ctrl-V paste via wtype after {}ms delay", delay_ms);
        Ok(())
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Wayland"
    }
}
