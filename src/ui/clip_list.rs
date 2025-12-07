use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::{AppMode, RegisterFilter, ViewMode};
use crate::models::ClipEntry;
use chrono::{DateTime, Local};

/// Format timestamp relative to now
fn format_timestamp(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp, 0)
        .map(|utc| utc.with_timezone(&Local))
        .unwrap_or_else(|| Local::now());

    let now = Local::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_weeks() < 4 {
        format!("{}w ago", duration.num_weeks())
    } else {
        dt.format("%b %d").to_string()
    }
}

/// Render list items in compact mode (single line per clip)
fn render_compact_items(
    entries: &[&ClipEntry],
    selected: usize,
    available_width: usize,
    c: &super::colorscheme::ColorScheme,
) -> Vec<ListItem<'static>> {
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let mut spans = Vec::new();

            // Determine text color and styling based on selection
            let is_selected = i == selected;
            let text_style = if is_selected {
                Style::default()
                    .fg(c.selection)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(c.text)
            };

            // Add clip number (starting from 0)
            let number = format!("{:3} ", i);
            spans.push(Span::styled(number.clone(), text_style));

            // Add pinned indicator
            if entry.pinned {
                spans.push(Span::styled(" ", text_style));
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
            spans.push(Span::styled(preview, text_style));

            // Add padding and registers on the right
            if has_registers {
                let padding_len = available_width
                    .saturating_sub(prefix_len)
                    .saturating_sub(preview_len)
                    .saturating_sub(registers_len);

                spans.push(Span::raw(" ".repeat(padding_len)));

                // Add register spans with their colors (not affected by selection)
                for (text, color) in register_strs.iter() {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(text.clone(), Style::default().fg(*color)));
                }
            }

            let line = Line::from(spans);
            ListItem::new(line)
        })
        .collect()
}

/// Render list items in comfortable mode (two lines per clip)
fn render_comfortable_items(
    entries: &[&ClipEntry],
    selected: usize,
    available_width: usize,
    c: &super::colorscheme::ColorScheme,
) -> Vec<ListItem<'static>> {
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            // Determine text color and styling based on selection
            let is_selected = i == selected;
            let text_style = if is_selected {
                Style::default()
                    .fg(c.selection)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(c.text)
            };

            // Metadata not affected by selection
            let metadata_color = c.subtext;

            // LINE 1: Number + Preview
            let mut line1_spans = Vec::new();
            let number = format!("{:3} ", i);
            line1_spans.push(Span::styled(number.clone(), text_style));

            // Get preview text (full width minus number)
            let max_preview_len = available_width.saturating_sub(3);
            let preview = entry.preview(max_preview_len);
            line1_spans.push(Span::styled(preview, text_style));

            // LINE 2: Pin + Date + Registers
            let mut line2_spans = Vec::new();

            // Pin directly under the clip number
            if entry.pinned {
                line2_spans.push(Span::styled("   ", Style::default().fg(metadata_color)));
            } else {
                line2_spans.push(Span::raw("    "));
            }

            // Add timestamp
            let timestamp_secs = entry
                .timestamp
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            let timestamp_str = format_timestamp(timestamp_secs);
            line2_spans.push(Span::styled(
                timestamp_str.clone(),
                Style::default().fg(metadata_color),
            ));

            // Calculate registers
            let mut register_strs = Vec::new();
            for &reg in &entry.temporary_registers {
                register_strs.push((format!("'{}", reg), c.temp_reg));
            }
            for &reg in &entry.permanent_registers {
                register_strs.push((format!("\"{}", reg), c.perm_reg));
            }

            // Add registers 2 spaces after the date
            if !register_strs.is_empty() {
                line2_spans.push(Span::raw("  "));

                for (text, color) in register_strs.iter() {
                    line2_spans.push(Span::raw(" "));
                    line2_spans.push(Span::styled(text.clone(), Style::default().fg(*color)));
                }
            }

            // Add empty line for spacing
            let lines = vec![
                Line::from(line1_spans),
                Line::from(line2_spans),
                Line::from(vec![Span::raw("")]),
            ];
            ListItem::new(lines)
        })
        .collect()
}

/// Render the clip list widget showing clipboard history entries
/// Always shows header at top (search or title or numeric prefix) with item count
pub fn render_clip_list(
    frame: &mut Frame,
    area: Rect,
    entries: &[&ClipEntry],
    selected: usize,
    mode: AppMode,
    search_query: &str,
    numeric_prefix: &str,
    register_filter: RegisterFilter,
    view_mode: ViewMode,
    color_scheme: &super::colorscheme::ColorScheme,
) {
    let c = color_scheme;

    // Reserve lines for header (search or title or jump mode)
    // In comfortable mode: title, empty, count, empty (4 lines)
    // In compact mode: title + count on same line (1 line)
    let header_height = match view_mode {
        ViewMode::Comfortable => 4, // Title, empty, count, empty
        ViewMode::Compact => 1,     // Title + count (same line)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height), // Header (+ spacing in comfortable)
            Constraint::Min(1),                // List
        ])
        .split(area);

    let header_area = chunks[0];
    let list_area = chunks[1];

    // Render header with item count (right-aligned)
    let item_count = entries.len();
    let count_text = format!("{} items", item_count);

    // Determine header text and style based on mode and filter
    let (header_left, header_style) = if !numeric_prefix.is_empty() {
        // Numeric prefix mode: show the prefix being typed with space
        (
            format!(": {}", numeric_prefix),
            Style::default().fg(c.temp_reg),
        )
    } else if matches!(mode, AppMode::Search) {
        // Search mode: show "/ <query>" with filter if active
        let base = format!("/ {}", search_query);
        let with_filter = match register_filter {
            RegisterFilter::Temporary => format!("{} [temp]", base),
            RegisterFilter::Permanent => format!("{} [perm]", base),
            RegisterFilter::None => base,
        };
        (with_filter, Style::default().fg(c.search_input))
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

    // Build header lines based on view mode
    let header_lines = match view_mode {
        ViewMode::Comfortable => {
            // Comfortable: title, empty, count, empty (each on separate lines)
            vec![
                Line::from(Span::styled(header_left, header_style)),
                Line::from(""),
                Line::from(Span::styled(count_text, Style::default().fg(c.subtext))),
                Line::from(""),
            ]
        }
        ViewMode::Compact => {
            // Compact: title and count on same line (right-aligned)
            let available_width = header_area.width as usize;
            let left_width = header_left.len();
            let right_width = count_text.len();
            let padding = if left_width + right_width + 1 <= available_width {
                available_width - left_width - right_width
            } else {
                1
            };

            vec![Line::from(vec![
                Span::styled(header_left, header_style),
                Span::raw(" ".repeat(padding)),
                Span::styled(count_text, Style::default().fg(c.subtext)),
            ])]
        }
    };

    let header_para = Paragraph::new(header_lines);
    frame.render_widget(header_para, header_area);

    // Create list items based on view mode
    let available_width = list_area.width as usize;
    let items: Vec<ListItem> = match view_mode {
        ViewMode::Compact => render_compact_items(entries, selected, available_width, &c),
        ViewMode::Comfortable => render_comfortable_items(entries, selected, available_width, &c),
    };

    // Create list without borders
    // scroll_padding keeps 3 items visible above/below selection when scrolling
    let list = List::new(items).highlight_symbol("►").scroll_padding(1);

    let mut list_state = ListState::default();
    list_state.select(Some(selected));

    frame.render_stateful_widget(list, list_area, &mut list_state);
}
