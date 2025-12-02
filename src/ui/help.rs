use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::layout::centered_rect;
use super::colorscheme::colors;

/// Render help overlay with keybindings
pub fn render_help_overlay(frame: &mut Frame, area: Rect) {
    let help_text = r#" Help

 CLIPR KEYBINDINGS

 Navigation:
   j/k         Move down/up
   gg/G        Jump to top/bottom
   g<number>   Jump to clip number
   Ctrl-d/u    Half-page down/up

 Search & Filter:
   /           Start fuzzy search
   '           Filter by temporary registers
   "           Filter by permanent registers
   Esc         Clear search/filter

 Actions:
   Enter       Copy to clipboard (and exit if configured)
   m<letter>   Assign to temporary register (like vim marks)
   p           Toggle pin
   d           Delete entry
   D           Clear all unpinned (with confirmation)
   q/Esc       Quit

 Help:
   ?           Show/hide this help

 Press any key to close...
"#;

    let overlay_area = centered_rect(60, 80, area);
    let c = colors();

    // Clear the background area first to hide underlying content
    frame.render_widget(Clear, overlay_area);

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .style(Style::default().bg(c.mantle))
                .padding(ratatui::widgets::Padding::uniform(1)),
        )
        .style(Style::default().fg(c.text))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}
