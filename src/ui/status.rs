use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::Theme;
use crate::app::{App, AppMode, RegisterFilter};

const NORMAL_HINTS: &[(&[&str], &str)] = &[
    (&["j", "k"], "move"),
    (&["/"], "search"),
    (&["'", "\""], "filter"),
    (&["m"], "mark"),
    (&["p"], "pin"),
    (&["d"], "delete"),
    (&["D"], "clear all"),
    (&["Enter"], "copy"),
    (&["Ctrl-Spc"], "paste"),
    (&["?"], "help"),
];

const SEARCH_HINTS: &[(&[&str], &str)] = &[
    (&["↑", "↓"], "move"),
    (&["Esc"], "cancel"),
    (&["Enter"], "select"),
    (&["Ctrl-Spc"], "paste"),
];

const REGISTER_ASSIGN_HINTS: &[(&[&str], &str)] = &[(&["Esc"], "cancel")];

const CONFIRM_HINTS: &[(&[&str], &str)] = &[(&["y"], "confirm"), (&["n", "Esc"], "cancel")];

const HELP_HINTS: &[(&[&str], &str)] = &[(&["ESC"], "press any key to close help")];

const NUMERIC_HINTS: &[(&[&str], &str)] = &[
    (&["j", "k"], "move"),
    (&["Ctrl-d", "u"], "half-page"),
    (&["Enter"], "jump to line"),
    (&["Esc"], "cancel"),
];

const THEME_PICKER_HINTS: &[(&[&str], &str)] = &[
    (&["j", "k"], "navigate"),
    (&["Enter"], "select"),
    (&["Esc"], "cancel"),
];

/// Add a hint with keys and description to the hints vector
fn add_hint<'a>(hints: &mut Vec<Span<'a>>, keys: &[&'a str], description: &'a str, theme: &Theme) {
    // Add keys with styled separators
    for (i, key) in keys.iter().enumerate() {
        if i > 0 {
            hints.push(Span::styled(
                "/",
                theme.status_desc.add_modifier(Modifier::DIM),
            ));
        }
        hints.push(Span::styled(*key, theme.status_key));
    }

    hints.push(Span::raw(" "));
    hints.push(Span::styled(description, theme.status_desc));
    hints.push(Span::raw("  "));
}

/// Render keyboard hints bar showing mode-specific shortcuts
pub fn render_keyboard_hints(frame: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let mut hints = Vec::new();

    let hint_data = match app.mode {
        AppMode::Normal => NORMAL_HINTS,
        AppMode::Search => SEARCH_HINTS,
        AppMode::RegisterAssign => REGISTER_ASSIGN_HINTS,
        AppMode::Confirm => CONFIRM_HINTS,
        AppMode::Help => HELP_HINTS,
        AppMode::Numeric => NUMERIC_HINTS,
        AppMode::ThemePicker => THEME_PICKER_HINTS,
    };

    // Add static hints
    for (keys, description) in hint_data {
        add_hint(&mut hints, keys, description, theme);
    }

    // Add dynamic q/Esc behavior for normal mode
    if app.mode == AppMode::Normal {
        if !app.search_input.value().is_empty() {
            add_hint(&mut hints, &["q"], "quit", theme);
            add_hint(&mut hints, &["Esc"], "clear search", theme);
        } else if app.register_filter != RegisterFilter::None {
            add_hint(&mut hints, &["q"], "quit", theme);
            add_hint(&mut hints, &["Esc"], "clear filter", theme);
        } else {
            add_hint(&mut hints, &["q", "Esc"], "quit", theme);
        }
    }

    let paragraph =
        Paragraph::new(Line::from(hints)).style(theme.status_desc.bg(theme.status_bar_bg));

    frame.render_widget(paragraph, area);
}
