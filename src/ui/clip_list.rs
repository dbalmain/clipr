use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Cell, List, ListItem, ListState, Paragraph, Row, Table};
use unicode_width::UnicodeWidthStr;

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

/// Render table rows for compact mode (two columns: content and registers)
fn render_compact_table_rows<'a>(
    entries: &[&ClipEntry],
    selected: usize,
    content_col_width: usize,
    theme: &'a super::Theme,
) -> Vec<Row<'a>> {
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == selected;
            let text_style = if is_selected {
                Style::default()
                    .fg(theme.clip_text_selected.fg.unwrap_or(theme.default_fg))
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.clip_text
            };

            // Column 1: Number
            let number = format!("{:3}", i);
            let number_cell = Cell::from(Span::styled(number, text_style));

            // Column 2: Pin indicator
            let pin_text = if entry.pinned {
                &theme.pin_indicator
            } else {
                ""
            };
            let pin_style = theme.pin_indicator_style;
            let pin_cell = Cell::from(Span::styled(pin_text, pin_style));

            // Column 3: Preview (use full content_col_width since number and pin are separate)
            let preview = entry.preview(content_col_width);
            let preview_cell = Cell::from(Span::styled(preview, text_style));

            // Column 4: Registers
            let mut register_spans = Vec::new();

            // Temporary registers
            for (idx, &reg) in entry.temporary_registers.iter().enumerate() {
                if idx > 0 {
                    register_spans.push(Span::raw(" "));
                }
                let style = if is_selected {
                    theme.temp_register.bg(theme.selection_bg)
                } else {
                    theme.temp_register
                };
                register_spans.push(Span::styled(format!("'{}", reg), style));
            }

            if !entry.temporary_registers.is_empty() && !entry.permanent_registers.is_empty() {
                register_spans.push(Span::raw("  "));
            }

            // Permanent registers
            for (idx, &reg) in entry.permanent_registers.iter().enumerate() {
                if idx > 0 {
                    register_spans.push(Span::raw(" "));
                }
                let style = if is_selected {
                    theme.perm_register.bg(theme.selection_bg)
                } else {
                    theme.perm_register
                };
                register_spans.push(Span::styled(format!("\"{}", reg), style));
            }

            let register_cell = Cell::from(Line::from(register_spans));

            // Create row with 4 columns
            let row = Row::new(vec![number_cell, pin_cell, preview_cell, register_cell]);
            if is_selected {
                row.style(Style::default().bg(theme.selection_bg))
            } else {
                row
            }
        })
        .collect()
}

/// Render list items in comfortable mode (two lines per clip)
fn render_comfortable_items(
    entries: &[&ClipEntry],
    selected: usize,
    available_width: usize,
    theme: &super::Theme,
) -> Vec<ListItem<'static>> {
    // Pre-create pin spans to avoid cloning in the loop
    let pin_padding = " ".repeat(3usize.saturating_sub(theme.pin_indicator.width()));
    let pin_str = format!("{}{} ", pin_padding, theme.pin_indicator);
    let pin_span = Span::styled(pin_str, theme.pin_indicator_style);
    let no_pin_span = Span::raw("    ");

    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            // Determine text color and styling based on selection
            let is_selected = i == selected;
            let text_style = if is_selected {
                Style::default()
                    .fg(theme.clip_text_selected.fg.unwrap_or(theme.default_fg))
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.clip_text
            };

            // Metadata not affected by selection
            let metadata_color = theme.timestamp.fg.unwrap_or(theme.default_fg);

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
                line2_spans.push(pin_span.clone());
            } else {
                line2_spans.push(no_pin_span.clone());
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
                register_strs.push((format!("'{}", reg), theme.temp_register));
            }
            for &reg in &entry.permanent_registers {
                register_strs.push((format!("\"{}", reg), theme.perm_register));
            }

            // Add registers 2 spaces after the date
            if !register_strs.is_empty() {
                line2_spans.push(Span::raw("  "));

                for (text, style) in register_strs.iter() {
                    line2_spans.push(Span::raw(" "));
                    line2_spans.push(Span::styled(text.clone(), *style));
                }
            }

            // Add empty line for spacing
            let lines = vec![
                Line::from(line1_spans),
                Line::from(line2_spans),
                Line::from(vec![Span::raw("")]),
            ];
            let item = ListItem::new(lines);

            // Apply selection background to entire item (all 3 lines) if selected
            if is_selected {
                item.style(Style::default().bg(theme.selection_bg))
            } else {
                item
            }
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
    theme: &super::Theme,
) {
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
        (format!(": {}", numeric_prefix), theme.temp_register)
    } else if matches!(mode, AppMode::Search) {
        // Search mode: show "/ <query>" with filter if active
        let base = format!("/ {}", search_query);
        let with_filter = match register_filter {
            RegisterFilter::Temporary => format!("{} [temp]", base),
            RegisterFilter::Permanent => format!("{} [perm]", base),
            RegisterFilter::None => base,
        };
        (with_filter, theme.search_input)
    } else {
        // Normal mode: show "Clipboard History" or filter status
        let header = match register_filter {
            RegisterFilter::Temporary => "Temporary Registers".to_string(),
            RegisterFilter::Permanent => "Permanent Registers".to_string(),
            RegisterFilter::None => "Clipboard History".to_string(),
        };
        let style = match register_filter {
            RegisterFilter::Temporary => theme.temp_register,
            RegisterFilter::Permanent => theme.perm_register,
            RegisterFilter::None => theme.clip_list_header,
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
                Line::from(Span::styled(count_text, theme.clip_list_item_count)),
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
                Span::styled(count_text, theme.clip_list_item_count),
            ])]
        }
    };

    // Use search_bg when in search mode, otherwise clip_list_bg
    let header_bg = if matches!(mode, AppMode::Search) {
        theme.search_bg
    } else {
        theme.clip_list_bg
    };
    let header_para = Paragraph::new(header_lines).style(Style::default().bg(header_bg));
    frame.render_widget(header_para, header_area);

    // Render based on view mode
    let available_width = list_area.width as usize;

    match view_mode {
        ViewMode::Compact => {
            // Calculate max register count across all entries (capped at 4)
            let max_register_count = entries
                .iter()
                .map(|e| e.temporary_registers.len() + e.permanent_registers.len())
                .max()
                .unwrap_or(0)
                .min(4);

            // Calculate register column width: ~3 chars per register + spacing
            let register_col_width = if max_register_count > 0 {
                (max_register_count * 3 + 2) as u16
            } else {
                0
            };

            // Check if any entries are pinned
            let has_pinned = entries.iter().any(|e| e.pinned);
            let pin_col_width = if has_pinned {
                theme.pin_indicator.width() as u16
            } else {
                0
            };

            // Calculate available width for preview column
            // Account for: number (3) + pin (2 if any) + register col + selection indicator + spacing (6)
            let highlight_width = theme
                .selection_indicator_compact
                .as_ref()
                .map(|s| s.len())
                .unwrap_or(0) as u16;
            let content_col_width = list_area
                .width
                .saturating_sub(pin_col_width) // Pin column
                .saturating_sub(register_col_width) // Register column
                .saturating_sub(highlight_width) // Selection indicator
                .saturating_sub(6); // Table spacing

            // Use Table for compact mode with 4 columns
            let rows =
                render_compact_table_rows(entries, selected, content_col_width as usize, theme);

            // Table with 4 columns: number, pin, preview (fills space), registers
            let widths = [
                Constraint::Length(3),                  // Number
                Constraint::Length(pin_col_width),      // Pin (0 if none pinned)
                Constraint::Min(10),                    // Preview (fills remaining)
                Constraint::Length(register_col_width), // Registers
            ];
            let table = Table::new(rows, widths)
                .style(Style::default().bg(theme.clip_list_bg))
                .highlight_symbol(theme.selection_indicator_compact.as_deref().unwrap_or(""));

            let mut table_state = ratatui::widgets::TableState::default();
            table_state.select(Some(selected));

            frame.render_stateful_widget(table, list_area, &mut table_state);
        }
        ViewMode::Comfortable => {
            // Use List for comfortable mode
            let items = render_comfortable_items(entries, selected, available_width, theme);

            let highlight = theme
                .selection_indicator_comfortable
                .as_deref()
                .unwrap_or("");
            let list = List::new(items)
                .highlight_symbol(highlight)
                .scroll_padding(1)
                .style(Style::default().bg(theme.clip_list_bg));

            let mut list_state = ListState::default();
            list_state.select(Some(selected));

            frame.render_stateful_widget(list, list_area, &mut list_state);
        }
    }
}
