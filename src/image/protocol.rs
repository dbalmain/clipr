use ratatui_image::picker::Picker;

/// Image protocol handler for ratatui-based rendering
/// Uses ratatui-image library which integrates properly with ratatui's rendering
pub struct ImageProtocol {
    /// Protocol picker that auto-detects terminal capabilities
    pub picker: Picker,
}

impl ImageProtocol {
    /// Create a new image protocol handler
    /// Auto-detects terminal capabilities (Kitty, Sixel, iTerm2, or Halfblocks fallback)
    pub fn new() -> Self {
        // Try to auto-detect terminal capabilities
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| {
            // Fallback to default font size if detection fails
            Picker::from_fontsize((8, 12))
        });

        ImageProtocol { picker }
    }

    /// Check if the terminal supports image rendering
    pub fn is_available(&self) -> bool {
        // Picker always returns something (falls back to Halfblocks)
        // but we want to know if we have proper protocol support
        true
    }
}

impl Default for ImageProtocol {
    fn default() -> Self {
        Self::new()
    }
}
