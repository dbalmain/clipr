use anyhow::Result;

/// Trait for clipboard backend abstraction
/// Supports different clipboard systems (Wayland, X11)
/// Backend is write-only: used to copy selected clips back to the clipboard
/// Clipboard monitoring is handled separately via daemon processes
pub trait ClipboardBackend: Send + Sync {
    /// Write text to clipboard
    fn write_text(&self, text: &str) -> Result<()>;

    /// Write image to clipboard (PNG format)
    fn write_image(&self, data: &[u8]) -> Result<()>;

    /// Check if this backend supports image operations
    fn supports_images(&self) -> bool;

    /// Get the backend name (for logging/debugging)
    fn name(&self) -> &'static str;
}
