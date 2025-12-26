use anyhow::{Context, Result, anyhow};
use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Default pin indicator character
const DEFAULT_PIN_INDICATOR: &str = " ";

/// Theme errors
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    #[error("Failed to parse theme file: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid RGB value: {field} = {value} (must be 0-255)")]
    InvalidRgb { field: String, value: i32 },

    #[error("Unknown color reference: ${0}")]
    UnknownColorRef(String),

    #[error("Unknown style reference: ${0}")]
    UnknownStyleRef(String),

    #[error("Missing required section: {0}")]
    MissingSection(String),

    #[error("Theme file not found: {0}")]
    FileNotFound(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Runtime theme with direct field access for all UI elements
#[derive(Debug, Clone)]
pub struct Theme {
    // === Default Colors ===
    pub default_fg: Color,
    pub default_bg: Color,

    // === Panel Backgrounds ===
    pub clip_list_bg: Color,
    pub preview_bg: Color,

    // === Modal Backgrounds ===
    pub help_modal_bg: Color,
    pub error_modal_bg: Color,
    pub confirm_modal_bg: Color,

    // === Other Backgrounds ===
    pub selection_bg: Color,
    pub status_bar_bg: Color,
    pub search_bg: Color,
    pub search_focused_bg: Color,

    // === Indicators ===
    pub selection_indicator_compact: Option<String>,
    pub selection_indicator_comfortable: Option<String>,
    pub selection_indicator_repeats_comfortable: bool,
    pub selection_indicator_style: Style,
    pub pin_indicator: String,
    pub pin_indicator_style: Style,

    // === Clip List Elements ===
    pub clip_number: Style,
    pub clip_text: Style,
    pub clip_text_selected: Style,
    pub temp_register: Style,
    pub perm_register: Style,
    pub timestamp: Style,
    pub clip_list_header: Style,
    pub clip_list_item_count: Style,

    // === Preview Panel ===
    pub preview_text: Style,
    pub preview_loading: Style,
    pub preview_file_label: Style,
    pub preview_metadata_label: Style,
    pub preview_metadata_value: Style,

    // === Status Bar ===
    pub status_key: Style,
    pub status_desc: Style,

    // === Search Input ===
    pub search_input: Style,
    pub search_border: Style,
    pub search_title: Style,

    // === Help Modal ===
    pub help_title: Style,
    pub help_header: Style,
    pub help_key: Style,
    pub help_desc: Style,
    pub help_footer: Style,

    // === Error Modal ===
    pub error_title: Style,
    pub error_text: Style,
    pub error_border: Style,

    // === Confirm Modal ===
    pub confirm_text: Style,
    pub confirm_key: Style,

    // === Divider ===
    pub divider_compact: Option<String>,
    pub divider_comfortable: Option<String>,
    pub divider_style: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

impl Theme {
    /// Catppuccin Mocha theme (dark)
    pub fn catppuccin_mocha() -> Self {
        let fg = Color::Rgb(205, 214, 244);
        let bg = Color::Rgb(30, 30, 46);

        Theme {
            // Defaults
            default_fg: fg,
            default_bg: bg,

            // Panel backgrounds
            clip_list_bg: bg,
            preview_bg: bg,

            // Modal backgrounds
            help_modal_bg: Color::Rgb(24, 24, 37),
            error_modal_bg: Color::Rgb(24, 24, 37),
            confirm_modal_bg: Color::Rgb(24, 24, 37),

            // Other backgrounds
            selection_bg: Color::Rgb(214, 219, 237),
            status_bar_bg: Color::Rgb(214, 219, 237),
            search_bg: bg,
            search_focused_bg: Color::Rgb(227, 232, 250),

            // Indicators
            selection_indicator_compact: None,
            selection_indicator_comfortable: None,
            selection_indicator_repeats_comfortable: true,
            selection_indicator_style: Style::default().fg(Color::Rgb(137, 180, 250)),
            pin_indicator: DEFAULT_PIN_INDICATOR.to_string(),
            pin_indicator_style: Style::default().fg(Color::Rgb(249, 226, 175)),

            // Clip list
            clip_number: Style::default()
                .fg(Color::Rgb(245, 194, 231))
                .add_modifier(Modifier::BOLD),
            clip_text: Style::default().fg(fg),
            clip_text_selected: Style::default()
                .fg(Color::Rgb(137, 180, 250))
                .add_modifier(Modifier::BOLD),
            temp_register: Style::default().fg(Color::Rgb(137, 220, 235)),
            perm_register: Style::default().fg(Color::Rgb(245, 194, 231)),
            timestamp: Style::default().fg(Color::Rgb(166, 173, 200)),
            clip_list_header: Style::default().fg(Color::Rgb(166, 173, 200)),
            clip_list_item_count: Style::default()
                .fg(Color::Rgb(166, 173, 200))
                .add_modifier(Modifier::DIM),

            // Preview
            preview_text: Style::default().fg(fg),
            preview_loading: Style::default().fg(Color::Rgb(166, 173, 200)),
            preview_file_label: Style::default().fg(Color::Rgb(166, 173, 200)),
            preview_metadata_label: Style::default().fg(Color::Rgb(166, 173, 200)),
            preview_metadata_value: Style::default().fg(fg),

            // Status bar
            status_key: Style::default().add_modifier(Modifier::BOLD),
            status_desc: Style::default().fg(Color::Rgb(166, 173, 200)),

            // Search
            search_input: Style::default().fg(Color::Rgb(249, 226, 175)),
            search_border: Style::default().fg(fg),
            search_title: Style::default().fg(fg),

            // Help modal
            help_title: Style::default()
                .fg(Color::Rgb(137, 180, 250))
                .add_modifier(Modifier::BOLD),
            help_header: Style::default()
                .fg(Color::Rgb(137, 180, 250))
                .add_modifier(Modifier::BOLD),
            help_key: Style::default()
                .fg(Color::Rgb(249, 226, 175))
                .add_modifier(Modifier::BOLD),
            help_desc: Style::default().fg(fg),
            help_footer: Style::default().fg(Color::Rgb(166, 173, 200)),

            // Error modal
            error_title: Style::default()
                .fg(Color::Rgb(243, 139, 168))
                .add_modifier(Modifier::BOLD),
            error_text: Style::default().fg(fg),
            error_border: Style::default().fg(Color::Rgb(243, 139, 168)),

            // Confirm modal
            confirm_text: Style::default().fg(fg),
            confirm_key: Style::default()
                .fg(Color::Rgb(137, 220, 235))
                .add_modifier(Modifier::BOLD),

            // Divider
            divider_compact: Some("│".to_string()),
            divider_comfortable: None,
            divider_style: Style::default().fg(Color::Rgb(108, 112, 134)),
        }
    }

    /// Catppuccin Latte theme (light)
    pub fn catppuccin_latte() -> Self {
        let fg = Color::Rgb(76, 79, 105);
        let bg = Color::Rgb(239, 241, 245);

        Theme {
            default_fg: fg,
            default_bg: bg,

            clip_list_bg: bg,
            preview_bg: bg,

            help_modal_bg: Color::Rgb(230, 233, 239),
            error_modal_bg: Color::Rgb(230, 233, 239),
            confirm_modal_bg: Color::Rgb(230, 233, 239),

            selection_bg: Color::Rgb(188, 192, 204),
            status_bar_bg: Color::Rgb(188, 192, 204),
            search_bg: bg,
            search_focused_bg: Color::Rgb(214, 219, 237),

            selection_indicator_compact: None,
            selection_indicator_comfortable: None,
            selection_indicator_repeats_comfortable: false,
            selection_indicator_style: Style::default().fg(Color::Rgb(30, 102, 245)),
            pin_indicator: DEFAULT_PIN_INDICATOR.to_string(),
            pin_indicator_style: Style::default().fg(Color::Rgb(223, 142, 29)),

            clip_number: Style::default()
                .fg(Color::Rgb(234, 118, 203))
                .add_modifier(Modifier::BOLD),
            clip_text: Style::default().fg(fg),
            clip_text_selected: Style::default()
                .fg(Color::Rgb(30, 102, 245))
                .add_modifier(Modifier::BOLD),
            temp_register: Style::default().fg(Color::Rgb(4, 165, 229)),
            perm_register: Style::default().fg(Color::Rgb(234, 118, 203)),
            timestamp: Style::default().fg(Color::Rgb(108, 111, 133)),
            clip_list_header: Style::default().fg(Color::Rgb(108, 111, 133)),
            clip_list_item_count: Style::default()
                .fg(Color::Rgb(108, 111, 133))
                .add_modifier(Modifier::DIM),

            preview_text: Style::default().fg(fg),
            preview_loading: Style::default().fg(Color::Rgb(108, 111, 133)),
            preview_file_label: Style::default().fg(Color::Rgb(108, 111, 133)),
            preview_metadata_label: Style::default().fg(Color::Rgb(108, 111, 133)),
            preview_metadata_value: Style::default().fg(fg),

            status_key: Style::default().add_modifier(Modifier::BOLD),
            status_desc: Style::default().fg(Color::Rgb(108, 111, 133)),

            search_input: Style::default().fg(Color::Rgb(223, 142, 29)),
            search_border: Style::default().fg(fg),
            search_title: Style::default().fg(fg),

            help_title: Style::default()
                .fg(Color::Rgb(30, 102, 245))
                .add_modifier(Modifier::BOLD),
            help_header: Style::default()
                .fg(Color::Rgb(30, 102, 245))
                .add_modifier(Modifier::BOLD),
            help_key: Style::default()
                .fg(Color::Rgb(223, 142, 29))
                .add_modifier(Modifier::BOLD),
            help_desc: Style::default().fg(fg),
            help_footer: Style::default().fg(Color::Rgb(108, 111, 133)),

            error_title: Style::default()
                .fg(Color::Rgb(210, 15, 57))
                .add_modifier(Modifier::BOLD),
            error_text: Style::default().fg(fg),
            error_border: Style::default().fg(Color::Rgb(210, 15, 57)),

            confirm_text: Style::default().fg(fg),
            confirm_key: Style::default()
                .fg(Color::Rgb(4, 165, 229))
                .add_modifier(Modifier::BOLD),

            divider_compact: Some("│".to_string()),
            divider_comfortable: None,
            divider_style: Style::default().fg(Color::Rgb(156, 160, 176)),
        }
    }

    /// Tokyo Night theme (dark)
    pub fn tokyonight_night() -> Self {
        let fg = Color::Rgb(192, 202, 245);
        let bg = Color::Rgb(26, 27, 38);

        Theme {
            default_fg: fg,
            default_bg: bg,

            clip_list_bg: bg,

            preview_bg: bg,

            help_modal_bg: Color::Rgb(22, 24, 35),
            error_modal_bg: Color::Rgb(22, 24, 35),
            confirm_modal_bg: Color::Rgb(22, 24, 35),

            selection_bg: Color::Rgb(36, 40, 59),
            status_bar_bg: Color::Rgb(36, 40, 59),
            search_bg: bg,
            search_focused_bg: Color::Rgb(49, 50, 68),

            selection_indicator_compact: None,
            selection_indicator_comfortable: None,
            selection_indicator_repeats_comfortable: false,
            selection_indicator_style: Style::default().fg(Color::Rgb(125, 207, 255)),
            pin_indicator: DEFAULT_PIN_INDICATOR.to_string(),
            pin_indicator_style: Style::default().fg(Color::Rgb(224, 175, 104)),

            clip_number: Style::default()
                .fg(Color::Rgb(187, 154, 247))
                .add_modifier(Modifier::BOLD),
            clip_text: Style::default().fg(fg),
            clip_text_selected: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),
            temp_register: Style::default().fg(Color::Rgb(125, 207, 255)),
            perm_register: Style::default().fg(Color::Rgb(187, 154, 247)),
            timestamp: Style::default().fg(Color::Rgb(169, 177, 214)),
            clip_list_header: Style::default().fg(Color::Rgb(169, 177, 214)),
            clip_list_item_count: Style::default()
                .fg(Color::Rgb(169, 177, 214))
                .add_modifier(Modifier::DIM),

            preview_text: Style::default().fg(fg),
            preview_loading: Style::default().fg(Color::Rgb(169, 177, 214)),
            preview_file_label: Style::default().fg(Color::Rgb(169, 177, 214)),
            preview_metadata_label: Style::default().fg(Color::Rgb(169, 177, 214)),
            preview_metadata_value: Style::default().fg(fg),

            status_key: Style::default().add_modifier(Modifier::BOLD),
            status_desc: Style::default().fg(Color::Rgb(169, 177, 214)),

            search_input: Style::default().fg(Color::Rgb(224, 175, 104)),
            search_border: Style::default().fg(fg),
            search_title: Style::default().fg(fg),

            help_title: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),
            help_header: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),

            help_key: Style::default()
                .fg(Color::Rgb(224, 175, 104))
                .add_modifier(Modifier::BOLD),
            help_desc: Style::default().fg(fg),
            help_footer: Style::default().fg(Color::Rgb(169, 177, 214)),

            error_title: Style::default()
                .fg(Color::Rgb(247, 118, 142))
                .add_modifier(Modifier::BOLD),
            error_text: Style::default().fg(fg),
            error_border: Style::default().fg(Color::Rgb(247, 118, 142)),

            confirm_text: Style::default().fg(fg),
            confirm_key: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),

            divider_compact: Some("│".to_string()),
            divider_comfortable: None,
            divider_style: Style::default().fg(Color::Rgb(68, 75, 106)),
        }
    }

    /// Tokyo Night Storm theme (darker blue)
    pub fn tokyonight_storm() -> Self {
        let fg = Color::Rgb(192, 202, 245);
        let bg = Color::Rgb(24, 27, 38);

        Theme {
            default_fg: fg,
            default_bg: bg,

            clip_list_bg: bg,
            preview_bg: bg,

            help_modal_bg: Color::Rgb(20, 22, 32),
            error_modal_bg: Color::Rgb(20, 22, 32),
            confirm_modal_bg: Color::Rgb(20, 22, 32),

            selection_bg: Color::Rgb(36, 40, 59),
            status_bar_bg: Color::Rgb(36, 40, 59),
            search_bg: bg,
            search_focused_bg: Color::Rgb(49, 50, 68),

            selection_indicator_compact: None,
            selection_indicator_comfortable: None,
            selection_indicator_repeats_comfortable: false,
            selection_indicator_style: Style::default().fg(Color::Rgb(125, 207, 255)),
            pin_indicator: DEFAULT_PIN_INDICATOR.to_string(),
            pin_indicator_style: Style::default().fg(Color::Rgb(224, 175, 104)),

            clip_number: Style::default()
                .fg(Color::Rgb(187, 154, 247))
                .add_modifier(Modifier::BOLD),
            clip_text: Style::default().fg(fg),
            clip_text_selected: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),
            temp_register: Style::default().fg(Color::Rgb(125, 207, 255)),
            perm_register: Style::default().fg(Color::Rgb(187, 154, 247)),
            timestamp: Style::default().fg(Color::Rgb(169, 177, 214)),
            clip_list_header: Style::default().fg(Color::Rgb(169, 177, 214)),
            clip_list_item_count: Style::default()
                .fg(Color::Rgb(169, 177, 214))
                .add_modifier(Modifier::DIM),

            preview_text: Style::default().fg(fg),
            preview_loading: Style::default().fg(Color::Rgb(169, 177, 214)),
            preview_file_label: Style::default().fg(Color::Rgb(169, 177, 214)),
            preview_metadata_label: Style::default().fg(Color::Rgb(169, 177, 214)),
            preview_metadata_value: Style::default().fg(fg),

            status_key: Style::default().add_modifier(Modifier::BOLD),
            status_desc: Style::default().fg(Color::Rgb(169, 177, 214)),

            search_input: Style::default().fg(Color::Rgb(224, 175, 104)),
            search_border: Style::default().fg(fg),
            search_title: Style::default().fg(fg),

            help_title: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),
            help_header: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),

            help_key: Style::default()
                .fg(Color::Rgb(224, 175, 104))
                .add_modifier(Modifier::BOLD),
            help_desc: Style::default().fg(fg),
            help_footer: Style::default().fg(Color::Rgb(169, 177, 214)),

            error_title: Style::default()
                .fg(Color::Rgb(247, 118, 142))
                .add_modifier(Modifier::BOLD),
            error_text: Style::default().fg(fg),
            error_border: Style::default().fg(Color::Rgb(247, 118, 142)),

            confirm_text: Style::default().fg(fg),
            confirm_key: Style::default()
                .fg(Color::Rgb(125, 207, 255))
                .add_modifier(Modifier::BOLD),

            divider_compact: Some("│".to_string()),
            divider_comfortable: None,
            divider_style: Style::default().fg(Color::Rgb(68, 75, 106)),
        }
    }

    /// Tokyo Night Day theme (light)
    pub fn tokyonight_day() -> Self {
        let fg = Color::Rgb(52, 59, 88);
        let bg = Color::Rgb(230, 233, 244);

        Theme {
            default_fg: fg,
            default_bg: bg,

            clip_list_bg: bg,
            preview_bg: bg,

            help_modal_bg: Color::Rgb(243, 244, 249),
            error_modal_bg: Color::Rgb(243, 244, 249),
            confirm_modal_bg: Color::Rgb(243, 244, 249),

            selection_bg: Color::Rgb(214, 219, 237),
            status_bar_bg: Color::Rgb(214, 219, 237),
            search_bg: bg,
            search_focused_bg: Color::Rgb(227, 232, 250),

            selection_indicator_compact: None,
            selection_indicator_comfortable: None,
            selection_indicator_repeats_comfortable: false,
            selection_indicator_style: Style::default().fg(Color::Rgb(34, 94, 168)),
            pin_indicator: DEFAULT_PIN_INDICATOR.to_string(),
            pin_indicator_style: Style::default().fg(Color::Rgb(150, 80, 0)),

            clip_number: Style::default()
                .fg(Color::Rgb(136, 57, 239))
                .add_modifier(Modifier::BOLD),
            clip_text: Style::default().fg(fg),
            clip_text_selected: Style::default()
                .fg(Color::Rgb(34, 94, 168))
                .add_modifier(Modifier::BOLD),
            temp_register: Style::default().fg(Color::Rgb(34, 94, 168)),
            perm_register: Style::default().fg(Color::Rgb(136, 57, 239)),
            timestamp: Style::default().fg(Color::Rgb(78, 89, 131)),
            clip_list_header: Style::default().fg(Color::Rgb(78, 89, 131)),
            clip_list_item_count: Style::default()
                .fg(Color::Rgb(78, 89, 131))
                .add_modifier(Modifier::DIM),

            preview_text: Style::default().fg(fg),
            preview_loading: Style::default().fg(Color::Rgb(78, 89, 131)),
            preview_file_label: Style::default().fg(Color::Rgb(78, 89, 131)),
            preview_metadata_label: Style::default().fg(Color::Rgb(78, 89, 131)),
            preview_metadata_value: Style::default().fg(fg),

            status_key: Style::default().add_modifier(Modifier::BOLD),
            status_desc: Style::default().fg(Color::Rgb(78, 89, 131)),

            search_input: Style::default().fg(Color::Rgb(150, 80, 0)),
            search_border: Style::default().fg(fg),
            search_title: Style::default().fg(fg),

            help_title: Style::default()
                .fg(Color::Rgb(34, 94, 168))
                .add_modifier(Modifier::BOLD),
            help_header: Style::default()
                .fg(Color::Rgb(136, 57, 239))
                .add_modifier(Modifier::BOLD),
            help_key: Style::default()
                .fg(Color::Rgb(150, 80, 0))
                .add_modifier(Modifier::BOLD),
            help_desc: Style::default().fg(fg),
            help_footer: Style::default().fg(Color::Rgb(78, 89, 131)),

            error_title: Style::default()
                .fg(Color::Rgb(186, 33, 66))
                .add_modifier(Modifier::BOLD),
            error_text: Style::default().fg(fg),
            error_border: Style::default().fg(Color::Rgb(186, 33, 66)),

            confirm_text: Style::default().fg(fg),
            confirm_key: Style::default()
                .fg(Color::Rgb(34, 94, 168))
                .add_modifier(Modifier::BOLD),

            divider_compact: Some("│".to_string()),
            divider_comfortable: None,
            divider_style: Style::default().fg(Color::Rgb(165, 173, 203)),
        }
    }

    /// Load theme by name (custom overrides built-in)
    pub fn load(name: &str) -> Result<Self> {
        // Try loading custom theme from file first (custom themes override built-in)
        let config_dir = get_config_dir()?;
        let theme_path = config_dir.join("themes").join(format!("{}.toml", name));

        if theme_path.exists() {
            return Self::load_from_file(&theme_path);
        }

        // Fall back to built-in theme
        if let Some(built_in) = BuiltInTheme::from_name(name) {
            return Ok(built_in.to_theme());
        }

        // Theme not found
        Err(anyhow!(
            "Unknown theme '{}'. Available built-in themes: {}",
            name,
            BuiltInTheme::all()
                .iter()
                .map(|t| t.name())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }

    /// Get the path to a theme file
    /// Returns the custom theme path if it exists, or potential path for creating one
    pub fn get_theme_path(name: &str) -> Result<PathBuf> {
        let config_dir = get_config_dir()?;
        let theme_path = config_dir.join("themes").join(format!("{}.toml", name));
        Ok(theme_path)
    }

    /// Load theme from a TOML file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read theme file: {}", path.display()))?;

        let definition: ThemeDefinition = toml::from_str(&content)
            .with_context(|| format!("Failed to parse theme file: {}", path.display()))?;

        Self::from_definition(definition)
    }

    /// Convert ThemeDefinition to runtime Theme
    pub fn from_definition(def: ThemeDefinition) -> Result<Self> {
        // Start with default theme as base
        let mut theme = Theme::default();

        // Override defaults if provided in backgrounds
        if let Some(fg) = def.backgrounds.get("default_fg") {
            theme.default_fg = resolve_color_in_def(fg, &def)?;
        }
        if let Some(bg) = def.backgrounds.get("default_bg") {
            theme.default_bg = resolve_color_in_def(bg, &def)?;
        }

        // Apply background colors
        for (key, color_val) in &def.backgrounds {
            let color = resolve_color_in_def(color_val, &def)?;
            match key.as_str() {
                "clip_list_bg" => theme.clip_list_bg = color,
                "preview_bg" => theme.preview_bg = color,
                "help_modal_bg" => theme.help_modal_bg = color,
                "error_modal_bg" => theme.error_modal_bg = color,
                "confirm_modal_bg" => theme.confirm_modal_bg = color,
                "selection_bg" => theme.selection_bg = color,
                "status_bar_bg" => theme.status_bar_bg = color,
                "search_bg" => theme.search_bg = color,
                "search_focused_bg" => theme.search_focused_bg = color,
                _ => {} // Ignore unknown backgrounds
            }
        }

        // Apply indicators
        if let Some(selection) = &def.indicators.selection_indicator_compact {
            theme.selection_indicator_compact = Some(selection.clone());
        }
        if let Some(selection) = &def.indicators.selection_indicator_comfortable {
            theme.selection_indicator_comfortable = Some(selection.clone());
        }
        theme.selection_indicator_repeats_comfortable =
            def.indicators.selection_indicator_repeats_comfortable;
        if let Some(pin) = &def.indicators.pin {
            theme.pin_indicator = pin.clone();
        }
        // Unconditionally set divider fields (will be None if not in TOML)
        theme.divider_compact = def.indicators.divider_compact.clone();
        theme.divider_comfortable = def.indicators.divider_comfortable.clone();

        // Apply element styles
        for (key, style_def) in &def.elements {
            let default_style = Style::default();
            let style = resolve_element_style(style_def, default_style, &def)?;

            match key.as_str() {
                "selection_indicator_style" => theme.selection_indicator_style = style,
                "pin_indicator_style" => theme.pin_indicator_style = style,
                "clip_number" => theme.clip_number = style,
                "clip_text" => theme.clip_text = style,
                "clip_text_selected" => theme.clip_text_selected = style,
                "temp_register" => theme.temp_register = style,
                "perm_register" => theme.perm_register = style,
                "timestamp" => theme.timestamp = style,
                "clip_list_header" => theme.clip_list_header = style,
                "clip_list_item_count" => theme.clip_list_item_count = style,
                "preview_text" => theme.preview_text = style,
                "preview_loading" => theme.preview_loading = style,
                "preview_file_label" => theme.preview_file_label = style,
                "preview_metadata_label" => theme.preview_metadata_label = style,
                "preview_metadata_value" => theme.preview_metadata_value = style,
                "status_key" => theme.status_key = style,
                "status_desc" => theme.status_desc = style,
                "search_input" => theme.search_input = style,
                "search_border" => theme.search_border = style,
                "search_title" => theme.search_title = style,
                "help_title" => theme.help_title = style,
                "help_header" => theme.help_header = style,

                "help_key" => theme.help_key = style,
                "help_desc" => theme.help_desc = style,
                "help_footer" => theme.help_footer = style,
                "error_title" => theme.error_title = style,
                "error_text" => theme.error_text = style,
                "error_border" => theme.error_border = style,
                "confirm_text" => theme.confirm_text = style,
                "confirm_key" => theme.confirm_key = style,
                "divider_style" => theme.divider_style = style,
                _ => {} // Ignore unknown elements
            }
        }

        Ok(theme)
    }

    /// Export theme to TOML format
    pub fn to_toml(&self) -> String {
        let mut output = String::from("# clipr theme export\n");
        output.push_str("# Generated by 'clipr export-theme'\n\n");

        // Helper to format RGB color
        let fmt_rgb = |color: Color| -> String {
            let rgb = rgb_array(color);
            format!("[{}, {}, {}]", rgb[0], rgb[1], rgb[2])
        };

        // Helper to format Style to inline TOML
        let fmt_style = |style: Style| -> String {
            let mut parts = Vec::new();

            if let Some(fg) = style.fg {
                parts.push(format!("fg = {}", fmt_rgb(fg)));
            }
            if let Some(bg) = style.bg {
                parts.push(format!("bg = {}", fmt_rgb(bg)));
            }
            if style.add_modifier.contains(Modifier::BOLD) {
                parts.push("bold = true".to_string());
            }
            if style.add_modifier.contains(Modifier::ITALIC) {
                parts.push("italic = true".to_string());
            }
            if style.add_modifier.contains(Modifier::UNDERLINED) {
                parts.push("underline = true".to_string());
            }
            if style.add_modifier.contains(Modifier::DIM) {
                parts.push("dim = true".to_string());
            }
            if style.add_modifier.contains(Modifier::SLOW_BLINK) {
                parts.push("slow_blink = true".to_string());
            }
            if style.add_modifier.contains(Modifier::RAPID_BLINK) {
                parts.push("rapid_blink = true".to_string());
            }
            if style.add_modifier.contains(Modifier::REVERSED) {
                parts.push("reversed = true".to_string());
            }
            if style.add_modifier.contains(Modifier::HIDDEN) {
                parts.push("hidden = true".to_string());
            }
            if style.add_modifier.contains(Modifier::CROSSED_OUT) {
                parts.push("crossed_out = true".to_string());
            }

            if parts.is_empty() {
                "{}".to_string()
            } else {
                format!("{{ {} }}", parts.join(", "))
            }
        };

        // Backgrounds section
        output.push_str("[backgrounds]\n");
        output.push_str(&format!("default_fg = {}\n", fmt_rgb(self.default_fg)));
        output.push_str(&format!("default_bg = {}\n", fmt_rgb(self.default_bg)));
        output.push_str(&format!("clip_list_bg = {}\n", fmt_rgb(self.clip_list_bg)));
        output.push_str(&format!("preview_bg = {}\n", fmt_rgb(self.preview_bg)));
        output.push_str(&format!(
            "help_modal_bg = {}\n",
            fmt_rgb(self.help_modal_bg)
        ));
        output.push_str(&format!(
            "error_modal_bg = {}\n",
            fmt_rgb(self.error_modal_bg)
        ));
        output.push_str(&format!(
            "confirm_modal_bg = {}\n",
            fmt_rgb(self.confirm_modal_bg)
        ));
        output.push_str(&format!("selection_bg = {}\n", fmt_rgb(self.selection_bg)));
        output.push_str(&format!(
            "status_bar_bg = {}\n",
            fmt_rgb(self.status_bar_bg)
        ));
        output.push_str(&format!("search_bg = {}\n", fmt_rgb(self.search_bg)));
        output.push_str(&format!(
            "search_focused_bg = {}\n",
            fmt_rgb(self.search_focused_bg)
        ));
        output.push('\n');

        // Indicators section
        output.push_str("[indicators]\n");
        if let Some(ref sel) = self.selection_indicator_compact {
            output.push_str(&format!("selection_indicator_compact = \"{}\"\n", sel));
        }
        if let Some(ref sel) = self.selection_indicator_comfortable {
            output.push_str(&format!("selection_indicator_comfortable = \"{}\"\n", sel));
        }
        output.push_str(&format!(
            "selection_indicator_repeats_comfortable = {}\n",
            self.selection_indicator_repeats_comfortable
        ));
        output.push_str(&format!("pin = \"{}\"\n", self.pin_indicator));
        if let Some(ref div) = self.divider_compact {
            output.push_str(&format!("divider_compact = \"{}\"\n", div));
        }
        if let Some(ref div) = self.divider_comfortable {
            output.push_str(&format!("divider_comfortable = \"{}\"\n", div));
        }
        output.push('\n');

        // Elements section
        output.push_str("[elements]\n");
        output.push_str(&format!(
            "selection_indicator_style = {}\n",
            fmt_style(self.selection_indicator_style)
        ));
        output.push_str(&format!(
            "pin_indicator_style = {}\n",
            fmt_style(self.pin_indicator_style)
        ));
        output.push_str(&format!("clip_number = {}\n", fmt_style(self.clip_number)));
        output.push_str(&format!("clip_text = {}\n", fmt_style(self.clip_text)));
        output.push_str(&format!(
            "clip_text_selected = {}\n",
            fmt_style(self.clip_text_selected)
        ));
        output.push_str(&format!(
            "temp_register = {}\n",
            fmt_style(self.temp_register)
        ));
        output.push_str(&format!(
            "perm_register = {}\n",
            fmt_style(self.perm_register)
        ));
        output.push_str(&format!("timestamp = {}\n", fmt_style(self.timestamp)));
        output.push_str(&format!(
            "clip_list_header = {}\n",
            fmt_style(self.clip_list_header)
        ));
        output.push_str(&format!(
            "clip_list_item_count = {}\n",
            fmt_style(self.clip_list_item_count)
        ));
        output.push_str(&format!(
            "preview_text = {}\n",
            fmt_style(self.preview_text)
        ));
        output.push_str(&format!(
            "preview_loading = {}\n",
            fmt_style(self.preview_loading)
        ));
        output.push_str(&format!(
            "preview_file_label = {}\n",
            fmt_style(self.preview_file_label)
        ));
        output.push_str(&format!(
            "preview_metadata_label = {}\n",
            fmt_style(self.preview_metadata_label)
        ));
        output.push_str(&format!(
            "preview_metadata_value = {}\n",
            fmt_style(self.preview_metadata_value)
        ));
        output.push_str(&format!("status_key = {}\n", fmt_style(self.status_key)));
        output.push_str(&format!("status_desc = {}\n", fmt_style(self.status_desc)));
        output.push_str(&format!(
            "search_input = {}\n",
            fmt_style(self.search_input)
        ));
        output.push_str(&format!(
            "search_border = {}\n",
            fmt_style(self.search_border)
        ));
        output.push_str(&format!(
            "search_title = {}\n",
            fmt_style(self.search_title)
        ));
        output.push_str(&format!("help_title = {}\n", fmt_style(self.help_title)));
        output.push_str(&format!("help_header = {}\n", fmt_style(self.help_header)));
        output.push_str(&format!("help_header = {}\n", fmt_style(self.help_header)));
        output.push_str(&format!("help_key = {}\n", fmt_style(self.help_key)));
        output.push_str(&format!("help_desc = {}\n", fmt_style(self.help_desc)));
        output.push_str(&format!("help_footer = {}\n", fmt_style(self.help_footer)));
        output.push_str(&format!("error_title = {}\n", fmt_style(self.error_title)));
        output.push_str(&format!("error_text = {}\n", fmt_style(self.error_text)));
        output.push_str(&format!(
            "error_border = {}\n",
            fmt_style(self.error_border)
        ));
        output.push_str(&format!(
            "confirm_text = {}\n",
            fmt_style(self.confirm_text)
        ));
        output.push_str(&format!("confirm_key = {}\n", fmt_style(self.confirm_key)));
        output.push_str(&format!(
            "divider_style = {}\n",
            fmt_style(self.divider_style)
        ));

        output
    }

    /// Get all available theme names (built-in + custom from filesystem)
    pub fn get_all_theme_names() -> Vec<String> {
        let mut themes = Vec::new();

        // Add all built-in themes
        for builtin in BuiltInTheme::all() {
            themes.push(builtin.name().to_string());
        }

        // Add custom themes from ~/.config/clipr/themes/
        if let Ok(config_dir) = get_config_dir() {
            let themes_dir = config_dir.join("themes");
            if let Ok(entries) = fs::read_dir(&themes_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type()
                        && file_type.is_file()
                        && let Some(name) = entry.file_name().to_str()
                        && name.ends_with(".toml")
                    {
                        let theme_name = name.trim_end_matches(".toml");
                        // Only add if not already in built-in list
                        if !themes.contains(&theme_name.to_string()) {
                            themes.push(theme_name.to_string());
                        }
                    }
                }
            }
        }

        themes.sort();
        themes
    }
}

/// Built-in theme variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltInTheme {
    CatppuccinMocha,
    CatppuccinLatte,
    TokyonightNight,
    TokyonightStorm,
    TokyonightDay,
}

impl BuiltInTheme {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "catppuccin-mocha" => Some(Self::CatppuccinMocha),
            "catppuccin-latte" => Some(Self::CatppuccinLatte),
            "tokyonight-night" => Some(Self::TokyonightNight),
            "tokyonight-storm" => Some(Self::TokyonightStorm),
            "tokyonight-day" => Some(Self::TokyonightDay),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::CatppuccinMocha => "catppuccin-mocha",
            Self::CatppuccinLatte => "catppuccin-latte",
            Self::TokyonightNight => "tokyonight-night",
            Self::TokyonightStorm => "tokyonight-storm",
            Self::TokyonightDay => "tokyonight-day",
        }
    }

    pub fn all() -> &'static [BuiltInTheme] {
        &[
            Self::CatppuccinMocha,
            Self::CatppuccinLatte,
            Self::TokyonightNight,
            Self::TokyonightStorm,
            Self::TokyonightDay,
        ]
    }

    pub fn to_theme(&self) -> Theme {
        match self {
            Self::CatppuccinMocha => Theme::catppuccin_mocha(),
            Self::CatppuccinLatte => Theme::catppuccin_latte(),
            Self::TokyonightNight => Theme::tokyonight_night(),
            Self::TokyonightStorm => Theme::tokyonight_storm(),
            Self::TokyonightDay => Theme::tokyonight_day(),
        }
    }
}

/// TOML deserialization structure
#[derive(Debug, Deserialize, Serialize)]
pub struct ThemeDefinition {
    #[serde(default)]
    pub colors: HashMap<String, ColorValue>,

    #[serde(default)]
    pub styles: HashMap<String, StyleDef>,

    #[serde(default)]
    pub backgrounds: HashMap<String, ColorValue>,

    #[serde(default)]
    pub indicators: IndicatorsDef,

    #[serde(default)]
    pub elements: HashMap<String, ElementStyleDef>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct IndicatorsDef {
    #[serde(alias = "selection_compact")]
    pub selection_indicator_compact: Option<String>,
    #[serde(alias = "selection_comfortable")]
    pub selection_indicator_comfortable: Option<String>,
    #[serde(default)]
    pub selection_indicator_repeats_comfortable: bool,
    pub pin: Option<String>,
    pub divider_compact: Option<String>,
    pub divider_comfortable: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorValue {
    Rgb([u8; 3]),
    Reference(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StyleDef {
    Reference(String),
    Inline {
        fg: Option<ColorValue>,
        bg: Option<ColorValue>,
        #[serde(default)]
        bold: bool,
        #[serde(default)]
        italic: bool,
        #[serde(default)]
        underline: bool,
        #[serde(default)]
        dim: bool,
        #[serde(default)]
        slow_blink: bool,
        #[serde(default)]
        rapid_blink: bool,
        #[serde(default)]
        reversed: bool,
        #[serde(default)]
        hidden: bool,
        #[serde(default)]
        crossed_out: bool,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ElementStyleDef {
    Reference(String),
    Inline {
        fg: Option<ColorValue>,
        bg: Option<ColorValue>,
        #[serde(default)]
        bold: bool,
        #[serde(default)]
        italic: bool,
        #[serde(default)]
        underline: bool,
        #[serde(default)]
        dim: bool,
        #[serde(default)]
        slow_blink: bool,
        #[serde(default)]
        rapid_blink: bool,
        #[serde(default)]
        reversed: bool,
        #[serde(default)]
        hidden: bool,
        #[serde(default)]
        crossed_out: bool,
    },
}

/// Resolve a ColorValue to a ratatui Color within a ThemeDefinition context
fn resolve_color_in_def(value: &ColorValue, def: &ThemeDefinition) -> Result<Color> {
    match value {
        ColorValue::Rgb(rgb) => {
            // u8 values are always 0-255, so no validation needed
            Ok(Color::Rgb(rgb[0], rgb[1], rgb[2]))
        }
        ColorValue::Reference(name) => {
            let color_val = def
                .colors
                .get(name)
                .ok_or_else(|| ThemeError::UnknownColorRef(name.to_string()))?;
            resolve_color_in_def(color_val, def)
        }
    }
}

/// Resolve ElementStyleDef to a Style
fn resolve_element_style(
    style_def: &ElementStyleDef,
    default: Style,
    def: &ThemeDefinition,
) -> Result<Style> {
    match style_def {
        ElementStyleDef::Reference(name) => {
            let style = def
                .styles
                .get(name)
                .ok_or_else(|| ThemeError::UnknownStyleRef(name.to_string()))?;
            resolve_style_def(style, default, def)
        }
        ElementStyleDef::Inline {
            fg,
            bg,
            bold,
            italic,
            underline,
            dim,
            slow_blink,
            rapid_blink,
            reversed,
            hidden,
            crossed_out,
        } => {
            let mut style = default;
            if let Some(fg_val) = fg {
                style = style.fg(resolve_color_in_def(fg_val, def)?);
            }
            if let Some(bg_val) = bg {
                style = style.bg(resolve_color_in_def(bg_val, def)?);
            }
            if *bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if *italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            if *underline {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            if *dim {
                style = style.add_modifier(Modifier::DIM);
            }
            if *slow_blink {
                style = style.add_modifier(Modifier::SLOW_BLINK);
            }
            if *rapid_blink {
                style = style.add_modifier(Modifier::RAPID_BLINK);
            }
            if *reversed {
                style = style.add_modifier(Modifier::REVERSED);
            }
            if *hidden {
                style = style.add_modifier(Modifier::HIDDEN);
            }
            if *crossed_out {
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }
            Ok(style)
        }
    }
}

/// Resolve StyleDef to a Style
fn resolve_style_def(style_def: &StyleDef, default: Style, def: &ThemeDefinition) -> Result<Style> {
    match style_def {
        StyleDef::Reference(name) => {
            let style = def
                .styles
                .get(name)
                .ok_or_else(|| ThemeError::UnknownStyleRef(name.to_string()))?;
            resolve_style_def(style, default, def)
        }
        StyleDef::Inline {
            fg,
            bg,
            bold,
            italic,
            underline,
            dim,
            slow_blink,
            rapid_blink,
            reversed,
            hidden,
            crossed_out,
        } => {
            let mut style = default;
            if let Some(fg_val) = fg {
                style = style.fg(resolve_color_in_def(fg_val, def)?);
            }
            if let Some(bg_val) = bg {
                style = style.bg(resolve_color_in_def(bg_val, def)?);
            }
            if *bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            if *italic {
                style = style.add_modifier(Modifier::ITALIC);
            }
            if *underline {
                style = style.add_modifier(Modifier::UNDERLINED);
            }
            if *dim {
                style = style.add_modifier(Modifier::DIM);
            }
            if *slow_blink {
                style = style.add_modifier(Modifier::SLOW_BLINK);
            }
            if *rapid_blink {
                style = style.add_modifier(Modifier::RAPID_BLINK);
            }
            if *reversed {
                style = style.add_modifier(Modifier::REVERSED);
            }
            if *hidden {
                style = style.add_modifier(Modifier::HIDDEN);
            }
            if *crossed_out {
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }
            Ok(style)
        }
    }
}

/// Resolve a ColorValue to a ratatui Color
fn _resolve_color_value(value: &ColorValue, colors: &HashMap<String, ColorValue>) -> Result<Color> {
    match value {
        ColorValue::Rgb(rgb) => Ok(Color::Rgb(rgb[0], rgb[1], rgb[2])),
        ColorValue::Reference(name) => {
            let name = name.strip_prefix('$').unwrap_or(name);
            let color_val = colors
                .get(name)
                .ok_or_else(|| ThemeError::UnknownColorRef(name.to_string()))?;
            _resolve_color_value(color_val, colors)
        }
    }
}

/// Convert Color to RGB array for TOML export
fn rgb_array(color: Color) -> [u8; 3] {
    match color {
        Color::Rgb(r, g, b) => [r, g, b],
        _ => [128, 128, 128],
    }
}

/// Get clipr config directory using XDG specification
fn get_config_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME environment variable not set")?;
    let home_path = PathBuf::from(home);

    let config_dir = if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg_config).join("clipr")
    } else {
        home_path.join(".config/clipr")
    };

    Ok(config_dir)
}

/// Get list of available custom themes
pub fn list_custom_themes() -> Result<Vec<String>> {
    let config_dir = get_config_dir()?;
    let themes_dir = config_dir.join("themes");

    if !themes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut themes = Vec::new();
    for entry in fs::read_dir(&themes_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml")
            && let Some(name) = path.file_stem().and_then(|s| s.to_str())
        {
            themes.push(name.to_string());
        }
    }

    Ok(themes)
}
