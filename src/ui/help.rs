use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};

use super::Theme;
use super::layout::centered_rect;

/// Render help overlay with keybindings
pub fn render_help_overlay(frame: &mut Frame, area: Rect, theme: &Theme) {
    let help_text = r#" Help

 Navigation:
   j/k         Move down/up
   Home/End    Jump to top/bottom
   PgUp/PgDn   Page up/down
   Ctrl-d/u    Half-page down/up

   <number>    Enter numeric mode (shows command palette)
               5j = move down 5 lines
               3Ctrl-d = 3 half-pages down
               15Enter = jump to line 15

 View:
   v           Toggle between Compact/Comfortable view modes

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

    // Clear the background area first to hide underlying content
    frame.render_widget(Clear, overlay_area);

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .style(Style::default().bg(theme.help_modal_bg))
                .padding(ratatui::widgets::Padding::uniform(2)),
        )
        .style(Style::default().fg(theme.help_desc.fg.unwrap_or(theme.default_fg)))
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}
