pub mod backend;
pub mod watch;
pub mod wayland;

use anyhow::{anyhow, Result};
use std::env;

pub use backend::ClipboardBackend;
pub use wayland::WaylandBackend;

/// Create a clipboard backend based on the current display server
/// Detects Wayland via WAYLAND_DISPLAY environment variable
/// Returns error if no supported display server is detected
pub fn create_backend() -> Result<Box<dyn ClipboardBackend>> {
    // Check for Wayland
    if env::var("WAYLAND_DISPLAY").is_ok() {
        log::info!("Detected Wayland display server");
        let backend = WaylandBackend::new()?;
        return Ok(Box::new(backend));
    }

    // X11 support will be added in Phase 8
    if env::var("DISPLAY").is_ok() {
        return Err(anyhow!(
            "X11 detected but not yet supported. Wayland support only (set WAYLAND_DISPLAY)"
        ));
    }

    Err(anyhow!(
        "No supported display server detected. Set WAYLAND_DISPLAY for Wayland"
    ))
}
