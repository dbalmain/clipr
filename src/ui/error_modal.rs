use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use super::layout::centered_rect;
use super::colorscheme::ColorScheme;

/// Render error modal with dismissal instructions
pub fn render_error_modal(frame: &mut Frame, area: Rect, error_msg: &str, color_scheme: &ColorScheme) {
    let overlay_area = centered_rect(70, 30, area);
    let c = color_scheme;

    // Clear the background area first to hide underlying content
    frame.render_widget(Clear, overlay_area);

    let error_text = format!("ERROR\n\n{}\n\nPress any key to dismiss...", error_msg);

    let paragraph = Paragraph::new(error_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(c.danger))
                .title("⚠ Configuration Error")
                .style(Style::default().bg(c.mantle))
                .padding(ratatui::widgets::Padding::uniform(2)),
        )
        .style(Style::default().fg(c.text))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}
