pub mod clip_list;
pub mod error_modal;
pub mod help;
pub mod layout;
pub mod preview;
pub mod search;
pub mod status;
pub mod theme;
pub mod theme_picker;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Paragraph};

pub use clip_list::render_clip_list;
pub use error_modal::render_error_modal;
pub use help::render_help_overlay;
pub use layout::{centered_rect, create_main_layout};
pub use preview::render_preview;
pub use search::render_search_input;
pub use status::render_keyboard_hints;
pub use theme::{BuiltInTheme, Theme};
pub use theme_picker::render_theme_picker;

/// Render vertical divider line between history and preview panels
/// In comfortable mode, renders empty space (3 chars wide) or custom divider if specified
/// In compact mode, renders a single vertical line or custom divider if specified
pub fn render_divider(
    frame: &mut Frame,
    area: Rect,
    view_mode: crate::app::ViewMode,
    theme: &Theme,
) {
    use crate::app::ViewMode;

    // Get the appropriate divider character based on view mode
    let divider_char = match view_mode {
        ViewMode::Compact => theme.divider_compact.as_deref(),
        ViewMode::Comfortable => theme.divider_comfortable.as_deref(),
    };

    // Only render if there's a divider character specified
    if let Some(divider) = divider_char {
        let lines: Vec<Line> = (0..area.height)
            .map(|_| Line::from(Span::styled(divider, theme.divider_style)))
            .collect();

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }
    // Otherwise, don't render anything (just empty space)
}

/// Render confirmation dialog overlay for clear all operation
pub fn render_confirm_overlay(frame: &mut Frame, area: Rect, theme: &Theme) {
    // Create centered overlay (smaller than help)
    let overlay_area = centered_rect(50, 20, area);

    // Clear background
    frame.render_widget(Clear, overlay_area);

    // Create confirmation message
    let message = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Clear all unpinned clips?",
            theme.confirm_text.add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "(Pinned and registered clips will be kept)",
            theme.help_footer,
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", theme.confirm_key.add_modifier(Modifier::BOLD)),
            Span::styled(" - Yes, clear all  ", theme.confirm_text),
            Span::styled("n", theme.confirm_key.add_modifier(Modifier::BOLD)),
            Span::styled(" - No, cancel", theme.confirm_text),
        ]),
    ];

    let paragraph = Paragraph::new(message)
        .block(
            Block::default()
                .style(Style::default().bg(theme.confirm_modal_bg))
                .padding(ratatui::widgets::Padding::uniform(2)),
        )
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, overlay_area);
}
