use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use super::colorscheme::colors;
use crate::app::{AppMode, RegisterFilter};
use crate::models::ClipEntry;

/// Render the clip list widget showing clipboard history entries
/// Always shows header at top (search or title or jump mode) with item count
pub fn render_clip_list(
    frame: &mut Frame,
    area: Rect,
    entries: &[&ClipEntry],
    selected: usize,
    mode: AppMode,
    search_query: &str,
    jump_number_input: &str,
    register_filter: RegisterFilter,
) {
    let c = colors();

    // Always reserve top line for header (search or title or jump mode)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(1),    // List
        ])
        .split(area);

    let header_area = chunks[0];
    let list_area = chunks[1];

    // Render header with item count (right-aligned)
    let item_count = entries.len();
    let count_text = format!("{} items", item_count);

    // Determine header text and style based on mode and filter
    let (header_left, header_style) = if !jump_number_input.is_empty() {
        // Jump mode: show "Go to: <number>"
        let number = jump_number_input.trim_start_matches('g');
        (
            format!("Go to: {}", number),
            Style::default().fg(c.temp_reg),
        )
    } else if matches!(mode, AppMode::Search) {
        // Search mode: show "/<query>" with filter if active
        let base = format!("/{}", search_query);
        let with_filter = match register_filter {
            RegisterFilter::Temporary => format!("{} [temp]", base),
            RegisterFilter::Permanent => format!("{} [perm]", base),
            RegisterFilter::None => base,
        };
        (
            with_filter,
            Style::default().fg(c.search_input),
        )
    } else {
        // Normal mode: show "Clipboard History" or filter status
        let header = match register_filter {
            RegisterFilter::Temporary => "Temporary Registers".to_string(),
            RegisterFilter::Permanent => "Permanent Registers".to_string(),
            RegisterFilter::None => "Clipboard History".to_string(),
        };
        let style = match register_filter {
            RegisterFilter::Temporary => Style::default().fg(c.temp_reg),
            RegisterFilter::Permanent => Style::default().fg(c.perm_reg),
            RegisterFilter::None => Style::default().fg(c.subtext),
        };
        (header, style)
    };

    let available_width = header_area.width as usize;
    let left_width = header_left.len();
    let right_width = count_text.len();
    let padding = if left_width + right_width + 1 <= available_width {
        available_width - left_width - right_width
    } else {
        1
    };

    let header_line = Line::from(vec![
        Span::styled(header_left, header_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(count_text, Style::default().fg(c.subtext)),
    ]);

    let header_para = Paragraph::new(header_line);
    frame.render_widget(header_para, header_area);

    // Create list items with numbering, indicators, and registers on right
    let available_width = list_area.width as usize;
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let mut spans = Vec::new();

            // Determine text/bg colors based on selection
            let (text_color, bg_color) = if i == selected {
                (c.base, Some(c.selection))
            } else {
                (c.text, None)
            };

            // Add clip number (starting from 0)
            let number = format!("{:2} ", i);
            spans.push(Span::styled(
                number.clone(),
                Style::default().fg(text_color),
            ));

            // Add pinned indicator
            if entry.pinned {
                spans.push(Span::styled("📌 ", Style::default().fg(text_color)));
            }

            // Calculate registers string and its length
            let mut register_strs = Vec::new();

            // Temporary registers with single quotes
            for &reg in &entry.temporary_registers {
                register_strs.push((format!("'{}", reg), c.temp_reg));
            }

            // Permanent registers with double quotes
            for &reg in &entry.permanent_registers {
                register_strs.push((format!("\"{}", reg), c.perm_reg));
            }

            let has_registers = !register_strs.is_empty();

            // Calculate total length: sum of all register strings + spaces between them
            let registers_len = if has_registers {
                register_strs.len() * 3 + 2
            } else {
                0
            };

            // Calculate available space for preview content
            let prefix_len = 5 + if entry.pinned { 3 } else { 0 }; // number + optional pin
            let max_preview_len = available_width
                .saturating_sub(prefix_len)
                .saturating_sub(registers_len)
                .saturating_sub(3);

            // Get truncated preview text
            let preview = entry.preview(max_preview_len);
            let preview_len = preview.chars().count();
            spans.push(Span::styled(preview, Style::default().fg(text_color)));

            // Add padding and registers on the right
            if has_registers {
                let padding_len = available_width
                    .saturating_sub(prefix_len)
                    .saturating_sub(preview_len)
                    .saturating_sub(registers_len);

                spans.push(Span::raw(" ".repeat(padding_len)));

                // Add register spans with their colors
                for (text, color) in register_strs.iter() {
                    spans.push(Span::raw(" "));
                    // Preserve register color even when selected
                    spans.push(Span::styled(text.clone(), Style::default().fg(*color)));
                }
            }

            let line = Line::from(spans);
            let item = ListItem::new(line);

            if let Some(bg) = bg_color {
                item.style(Style::default().bg(bg).add_modifier(Modifier::BOLD))
            } else {
                item
            }
        })
        .collect();

    // Create list without borders
    let list = List::new(items).highlight_symbol("► ");

    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    frame.render_stateful_widget(list, list_area, &mut list_state);
}
