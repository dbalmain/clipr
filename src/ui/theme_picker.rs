use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

use super::Theme;
use super::layout::centered_rect;

/// Render theme picker modal
pub fn render_theme_picker(
    frame: &mut Frame,
    area: Rect,
    themes: &[String],
    selected: usize,
    current_theme: &str,
    theme: &Theme,
) {
    let overlay_area = centered_rect(60, 70, area);

    // Clear background
    frame.render_widget(Clear, overlay_area);

    // Create list items
    let items: Vec<ListItem> = themes
        .iter()
        .map(|name| {
            let prefix = if name == current_theme { "● " } else { "  " };
            ListItem::new(format!("{}{}", prefix, name))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Theme Picker ")
                .style(Style::default().bg(theme.help_modal_bg)),
        )
        .highlight_symbol("► ")
        .highlight_style(theme.clip_text_selected)
        .style(Style::default().fg(theme.default_fg));

    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    frame.render_stateful_widget(list, overlay_area, &mut list_state);
}
