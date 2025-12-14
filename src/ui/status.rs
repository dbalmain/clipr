use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::Theme;
use crate::app::AppMode;

/// Render keyboard hints bar showing mode-specific shortcuts
pub fn render_keyboard_hints(frame: &mut Frame, area: Rect, mode: AppMode, theme: &Theme) {

    let hints = match mode {
        AppMode::Normal => {
            vec![
                Span::styled("j/k", theme.status_key),
                Span::raw(" move  "),
                Span::styled("/", theme.status_key),
                Span::raw(" search  "),
                Span::styled("'/\"", theme.status_key),
                Span::raw(" filter  "),
                Span::styled("m", theme.status_key),
                Span::raw(" mark  "),
                Span::styled("p", theme.status_key),
                Span::raw(" pin  "),
                Span::styled("d", theme.status_key),
                Span::raw(" delete  "),
                Span::styled("D", theme.status_key),
                Span::raw(" clear all  "),
                Span::styled("Enter", theme.status_key),
                Span::raw(" copy  "),
                Span::styled("q", theme.status_key),
                Span::raw(" quit  "),
                Span::styled("?", theme.status_key),
                Span::raw(" help"),
            ]
        }
        AppMode::Search => {
            vec![
                Span::raw("type to search  "),
                Span::styled("↑/↓", theme.status_key),
                Span::raw(" move  "),
                Span::styled("Esc", theme.status_key),
                Span::raw(" cancel  "),
                Span::styled("Enter", theme.status_key),
                Span::raw(" select"),
            ]
        }
        AppMode::RegisterAssign => {
            vec![
                Span::raw("type register key  "),
                Span::styled("Esc", theme.status_key),
                Span::raw(" cancel"),
            ]
        }
        AppMode::Confirm => {
            vec![
                Span::styled("y", theme.status_key),
                Span::raw(" confirm  "),
                Span::styled("n/Esc", theme.status_key),
                Span::raw(" cancel"),
            ]
        }
        AppMode::Help => {
            vec![Span::raw("press any key to close help")]
        }
        AppMode::Numeric => {
            vec![
                Span::styled("j/k", theme.status_key),
                Span::raw(" move  "),
                Span::styled("Ctrl-d/u", theme.status_key),
                Span::raw(" half-page  "),
                Span::styled("Enter", theme.status_key),
                Span::raw(" jump to line  "),
                Span::styled("Esc", theme.status_key),
                Span::raw(" cancel"),
            ]
        }
        AppMode::ThemePicker => {
            vec![
                Span::styled("j/k", theme.status_key),
                Span::raw(" navigate  "),
                Span::styled("Enter", theme.status_key),
                Span::raw(" select  "),
                Span::styled("Esc", theme.status_key),
                Span::raw(" cancel"),
            ]
        }
    };

    let paragraph = Paragraph::new(Line::from(hints)).style(theme.status_desc.bg(theme.status_bar_bg));

    frame.render_widget(paragraph, area);
}
