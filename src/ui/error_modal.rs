use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::layout::centered_rect;
use super::Theme;

/// Render error modal with dismissal instructions
pub fn render_error_modal(frame: &mut Frame, area: Rect, error_msg: &str, theme: &Theme) {
    let overlay_area = centered_rect(70, 30, area);

    // Clear the background area first to hide underlying content
    frame.render_widget(Clear, overlay_area);

    let error_text = format!("ERROR\n\n{}\n\nPress any key to dismiss...", error_msg);

    let paragraph = Paragraph::new(error_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.error_border)
                .title("⚠ Configuration Error")
                .style(Style::default().bg(theme.error_modal_bg))
                .padding(ratatui::widgets::Padding::uniform(2)),
        )
        .style(theme.error_text)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}
