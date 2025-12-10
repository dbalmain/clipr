use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::Theme;

/// Render search input bar at bottom of screen when in search mode
pub fn render_search_input(frame: &mut Frame, area: Rect, query: &str, theme: &Theme) {

    // Position at bottom, 3 lines tall
    let input_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(3),
        width: area.width,
        height: 3,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Search (Esc to cancel, Enter to select)")
        .padding(ratatui::widgets::Padding::horizontal(1));

    let input = Paragraph::new(format!("/{}", query))
        .block(block)
        .style(theme.search_input);

    frame.render_widget(input, input_area);
}
