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
    /// Catppuccin Mocha theme (dark)
    pub fn catppuccin_mocha() -> Self {
        ColorScheme {
            text: Color::Rgb(205, 214, 244),
            subtext: Color::Rgb(166, 173, 200),
            subtext_dim: Color::Rgb(186, 194, 222),
            temp_reg: Color::Rgb(137, 220, 235),
            perm_reg: Color::Rgb(245, 194, 231),
            selection: Color::Rgb(137, 180, 250),
            search_input: Color::Rgb(249, 226, 175),
            success: Color::Rgb(166, 227, 161),
            danger: Color::Rgb(243, 139, 168),
            surface0: Color::Rgb(54, 58, 79),
            surface1: Color::Rgb(69, 71, 90),
            overlay: Color::Rgb(108, 112, 134),
            mantle: Color::Rgb(24, 24, 37),
            base: Color::Rgb(30, 30, 46),
            status_bg: Color::Rgb(69, 71, 90),
            status_fg: Color::Rgb(205, 214, 244),
        }
    }

    /// Catppuccin Latte theme (light)
    pub fn catppuccin_latte() -> Self {
        ColorScheme {
            text: Color::Rgb(76, 79, 105),
            subtext: Color::Rgb(108, 111, 133),
            subtext_dim: Color::Rgb(92, 95, 119),
            temp_reg: Color::Rgb(4, 165, 229),
            perm_reg: Color::Rgb(234, 118, 203),
            selection: Color::Rgb(30, 102, 245),
            search_input: Color::Rgb(223, 142, 29),
            success: Color::Rgb(64, 160, 43),
            danger: Color::Rgb(210, 15, 57),
            surface0: Color::Rgb(204, 208, 218),
            surface1: Color::Rgb(188, 192, 204),
            overlay: Color::Rgb(156, 160, 176),
            mantle: Color::Rgb(230, 233, 239),
            base: Color::Rgb(239, 241, 245),
            status_bg: Color::Rgb(188, 192, 204),
            status_fg: Color::Rgb(76, 79, 105),
        }
    }

    /// Tokyo Night (dark)
    pub fn tokyonight_night() -> Self {
        ColorScheme {
            text: Color::Rgb(192, 202, 245),
            subtext: Color::Rgb(169, 177, 214),
            subtext_dim: Color::Rgb(148, 156, 186),
            temp_reg: Color::Rgb(125, 207, 255),
            perm_reg: Color::Rgb(187, 154, 247),
            selection: Color::Rgb(61, 89, 161),
            search_input: Color::Rgb(224, 175, 104),
            success: Color::Rgb(158, 206, 106),
            danger: Color::Rgb(247, 118, 142),
            surface0: Color::Rgb(30, 32, 48),
            surface1: Color::Rgb(36, 40, 59),
            overlay: Color::Rgb(68, 75, 106),
            mantle: Color::Rgb(22, 24, 35),
            base: Color::Rgb(26, 27, 38),
            status_bg: Color::Rgb(36, 40, 59),
            status_fg: Color::Rgb(192, 202, 245),
        }
    }

    /// Tokyo Night Storm (darker blue)
    pub fn tokyonight_storm() -> Self {
        ColorScheme {
            text: Color::Rgb(192, 202, 245),
            subtext: Color::Rgb(169, 177, 214),
            subtext_dim: Color::Rgb(148, 156, 186),
            temp_reg: Color::Rgb(125, 207, 255),
            perm_reg: Color::Rgb(187, 154, 247),
            selection: Color::Rgb(61, 89, 161),
            search_input: Color::Rgb(224, 175, 104),
            success: Color::Rgb(158, 206, 106),
            danger: Color::Rgb(247, 118, 142),
            surface0: Color::Rgb(28, 31, 44),
            surface1: Color::Rgb(36, 40, 59),
            overlay: Color::Rgb(68, 75, 106),
            mantle: Color::Rgb(20, 22, 32),
            base: Color::Rgb(24, 27, 38),
            status_bg: Color::Rgb(36, 40, 59),
            status_fg: Color::Rgb(192, 202, 245),
        }
    }

    /// Tokyo Night Day (light)
    pub fn tokyonight_day() -> Self {
        ColorScheme {
            text: Color::Rgb(52, 59, 88),
            subtext: Color::Rgb(78, 89, 131),
            subtext_dim: Color::Rgb(104, 112, 147),
            temp_reg: Color::Rgb(34, 94, 168),
            perm_reg: Color::Rgb(136, 57, 239),
            selection: Color::Rgb(179, 198, 255),
            search_input: Color::Rgb(150, 80, 0),
            success: Color::Rgb(51, 125, 7),
            danger: Color::Rgb(186, 33, 66),
            surface0: Color::Rgb(232, 235, 246),
            surface1: Color::Rgb(214, 219, 237),
            overlay: Color::Rgb(165, 173, 203),
            mantle: Color::Rgb(243, 244, 249),
            base: Color::Rgb(230, 233, 244),
            status_bg: Color::Rgb(214, 219, 237),
            status_fg: Color::Rgb(52, 59, 88),
        }
    }

    /// Get theme by name
    pub fn from_name(name: &str) -> anyhow::Result<Self> {
        match name {
            "catppuccin-mocha" => Ok(Self::catppuccin_mocha()),
            "catppuccin-latte" => Ok(Self::catppuccin_latte()),
            "tokyonight-night" => Ok(Self::tokyonight_night()),
            "tokyonight-storm" => Ok(Self::tokyonight_storm()),
            "tokyonight-day" => Ok(Self::tokyonight_day()),
            _ => Err(anyhow::anyhow!(
                "Unknown theme '{}'. Available themes: catppuccin-mocha, catppuccin-latte, tokyonight-night, tokyonight-storm, tokyonight-day",
                name
            )),
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
