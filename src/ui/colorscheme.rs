use ratatui::prelude::*;

/// Catppuccin Mocha color scheme
/// https://github.com/catppuccin/catppuccin
pub struct ColorScheme {
    // Text colors
    pub text: Color,           // Main text
    pub subtext: Color,        // Dimmed text (descriptions, metadata)
    pub subtext_dim: Color,    // More dimmed text

    // Semantic accent colors
    pub temp_reg: Color,       // Temporary registers
    pub perm_reg: Color,       // Permanent registers
    pub selection: Color,      // Selected items
    pub search_input: Color,   // Search input text
    pub success: Color,        // Success indicators
    pub danger: Color,         // Delete/danger indicators

    // UI elements
    pub surface0: Color,       // Background for UI elements
    pub surface1: Color,       // Slightly lighter background
    pub overlay: Color,        // Overlays/popups
    pub mantle: Color,         // Darker background
    pub base: Color,           // Main background

    // Status bar
    pub status_bg: Color,      // Status bar background
    pub status_fg: Color,      // Status bar foreground
}

impl ColorScheme {
    /// Catppuccin Mocha theme
    pub fn catppuccin_mocha() -> Self {
        ColorScheme {
            // Text
            text: Color::Rgb(205, 214, 244),       // #cdd6f4
            subtext: Color::Rgb(166, 173, 200),    // #a6adc8
            subtext_dim: Color::Rgb(186, 194, 222), // #bac2de

            // Semantic accents
            temp_reg: Color::Rgb(137, 220, 235),   // #89dceb (sky)
            perm_reg: Color::Rgb(245, 194, 231),   // #f5c2e7 (pink)
            selection: Color::Rgb(137, 180, 250),  // #89b4fa (blue)
            search_input: Color::Rgb(249, 226, 175), // #f9e2af (yellow)
            success: Color::Rgb(166, 227, 161),    // #a6e3a1 (green)
            danger: Color::Rgb(243, 139, 168),     // #f38ba8 (red)

            // Backgrounds
            surface0: Color::Rgb(54, 58, 79),      // #363a4f
            surface1: Color::Rgb(69, 71, 90),      // #45475a
            overlay: Color::Rgb(108, 112, 134),    // #6c7086
            mantle: Color::Rgb(24, 24, 37),        // #181825
            base: Color::Rgb(30, 30, 46),          // #1e1e2e

            // Status bar
            status_bg: Color::Rgb(69, 71, 90),     // #45475a
            status_fg: Color::Rgb(205, 214, 244),  // #cdd6f4
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

/// Global color scheme instance
/// TODO: Make this configurable via config file in the future
pub fn colors() -> ColorScheme {
    ColorScheme::default()
}
