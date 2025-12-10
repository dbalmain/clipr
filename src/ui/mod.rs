pub mod clip_list;
pub mod error_modal;
pub mod help;
pub mod layout;
pub mod preview;
pub mod search;
pub mod status;
pub mod theme;

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

/// Render vertical divider line between history and preview panels
/// In comfortable mode, renders empty space (3 chars wide)
/// In compact mode, renders a single vertical line
pub fn render_divider(frame: &mut Frame, area: Rect, view_mode: crate::app::ViewMode, theme: &Theme) {
    use crate::app::ViewMode;

    // Only render divider line in compact mode
    // In comfortable mode, the area is just empty space
    if matches!(view_mode, ViewMode::Compact) {
        let divider_char = "│";

        let lines: Vec<Line> = (0..area.height)
            .map(|_| Line::from(Span::styled(divider_char, theme.divider)))
            .collect();

        let paragraph = Paragraph::new(lines);
        frame.render_widget(paragraph, area);
    }
    // In comfortable mode, don't render anything (just empty space)
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
