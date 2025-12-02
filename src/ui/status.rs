use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::AppMode;
use super::colorscheme::colors;

/// Render keyboard hints bar showing mode-specific shortcuts
pub fn render_keyboard_hints(
    frame: &mut Frame,
    area: Rect,
    mode: AppMode,
) {
    let c = colors();

    let hints = match mode {
        AppMode::Normal => {
            vec![
                Span::styled("j/k", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":move  "),
                Span::styled("/", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":search  "),
                Span::styled("'/\"", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":filter  "),
                Span::styled("m", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":mark  "),
                Span::styled("p", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":pin  "),
                Span::styled("d", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":delete  "),
                Span::styled("D", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":clear  "),
                Span::styled("Enter", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":copy  "),
                Span::styled("q", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":quit  "),
                Span::styled("?", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":help"),
            ]
        }
        AppMode::Search => {
            vec![
                Span::raw("type to search  "),
                Span::styled("↑/↓", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":move  "),
                Span::styled("Esc", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":cancel  "),
                Span::styled("Enter", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":select"),
            ]
        }
        AppMode::RegisterAssign => {
            vec![
                Span::raw("type register key  "),
                Span::styled("Esc", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":cancel"),
            ]
        }
        AppMode::Confirm => {
            vec![
                Span::styled("y", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":confirm  "),
                Span::styled("n/Esc", Style::default().fg(c.temp_reg).add_modifier(Modifier::BOLD)),
                Span::raw(":cancel"),
            ]
        }
        AppMode::Help => {
            vec![
                Span::raw("press any key to close help"),
            ]
        }
    };

    let paragraph = Paragraph::new(Line::from(hints))
        .style(Style::default().bg(c.mantle).fg(c.subtext));

    frame.render_widget(paragraph, area);
}
