use ratatui::layout::{Constraint, Direction, Layout, Position};
use ratatui::prelude::*;
use ratatui::widgets::{Cell, Paragraph, Row, Table};
use tui_input::Input;
use unicode_width::UnicodeWidthStr;

use crate::app::{AppMode, RegisterFilter, ViewMode};
use crate::models::ClipEntry;
use chrono::{DateTime, Local};

/// Format timestamp relative to now
fn format_timestamp(timestamp: i64) -> String {
    let dt = DateTime::from_timestamp(timestamp, 0)
        .map(|utc| utc.with_timezone(&Local))
        .unwrap_or_else(Local::now);

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

            // Column 1: Number - use clip_number style
            let number = format!("{:3}", i);
            let number_cell = Cell::from(Span::styled(number, theme.clip_number));

            // Column 2: Pin indicator
            let pin_text = if entry.pinned {
                &theme.pin_indicator
            } else {
                ""
            };
            let pin_cell = Cell::from(Span::styled(pin_text, theme.pin_indicator_style));

            // Column 3: Preview - use clip_text or clip_text_selected
            let preview = entry.preview(content_col_width);
            let preview_style = if is_selected {
                theme.clip_text_selected
            } else {
                theme.clip_text
            };
            let preview_cell = Cell::from(Span::styled(preview, preview_style));

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

/// Render table rows for comfortable mode (three rows per clip: preview, metadata, spacing)
fn render_comfortable_table_rows<'a>(
    entries: &[&ClipEntry],
    selected: usize,
    available_width: usize,
    theme: &'a super::Theme,
) -> Vec<Row<'a>> {
    // Pre-create pin spans to avoid cloning in the loop
    let pin_padding = " ".repeat(3usize.saturating_sub(theme.pin_indicator.width()));
    let pin_str = format!("{}{} ", pin_padding, theme.pin_indicator);

    // Get selection indicator settings
    let indicator = theme
        .selection_indicator_comfortable
        .as_deref()
        .unwrap_or("");
    let indicator_repeats = theme.selection_indicator_repeats_comfortable;

    let mut rows = Vec::new();

    for (i, entry) in entries.iter().enumerate() {
        let is_selected = i == selected;

        // Use clip_text or clip_text_selected for preview
        let preview_style = if is_selected {
            theme.clip_text_selected
        } else {
            theme.clip_text
        };

        // Metadata not affected by selection
        let metadata_color = theme.timestamp.fg.unwrap_or(theme.default_fg);

        // ROW 1: Indicator space (always) + Number + Preview
        let mut row1_spans = Vec::new();

        // Always add space for indicator (show indicator if selected, otherwise empty space)
        let indicator_width = if !indicator.is_empty() {
            indicator.len() + 1 // indicator + space
        } else {
            0
        };

        if !indicator.is_empty() {
            if is_selected {
                row1_spans.push(Span::styled(indicator, theme.selection_indicator_style));
            } else {
                // Add empty space for alignment
                let spacing = " ".repeat(indicator.width());
                row1_spans.push(Span::raw(spacing));
            }
        }

        let number = format!("{:3} ", i);
        row1_spans.push(Span::styled(number, theme.clip_number));

        // Get preview text (full width minus number, indicator, and spacing)
        let max_preview_len = available_width.saturating_sub(6 + indicator_width);
        let preview = entry.preview(max_preview_len);
        row1_spans.push(Span::styled(preview, preview_style));

        let row1_cell = Cell::from(Line::from(row1_spans));
        let row1 = Row::new(vec![row1_cell]);
        let row1 = if is_selected {
            row1.style(Style::default().bg(theme.selection_bg))
        } else {
            row1
        };
        rows.push(row1);

        // ROW 2: Indicator space (always) + Pin + Date + Registers
        let mut line2_spans = Vec::new();

        // Always add space for indicator (show indicator if selected AND repeats, otherwise empty space)
        if !indicator.is_empty() {
            if is_selected && indicator_repeats {
                line2_spans.push(Span::styled(indicator, theme.selection_indicator_style));
            } else {
                // Add empty space equal to indicator width + 1 for alignment
                let spacing = " ".repeat(indicator.width());
                line2_spans.push(Span::raw(spacing));
            }
        }

        // Pin directly under the clip number
        let pin_span = if entry.pinned {
            Span::styled(pin_str.clone(), theme.pin_indicator_style)
        } else {
            Span::raw("    ")
        };
        line2_spans.push(pin_span);

        // Add timestamp
        let timestamp_secs = entry
            .timestamp
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let timestamp_str = format_timestamp(timestamp_secs);
        line2_spans.push(Span::styled(
            timestamp_str,
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

        let row2_cell = Cell::from(Line::from(line2_spans));
        let row2 = Row::new(vec![row2_cell]);
        let row2 = if is_selected {
            row2.style(Style::default().bg(theme.selection_bg))
        } else {
            row2
        };
        rows.push(row2);

        // ROW 3: Empty spacing line (no background even when selected)
        let row3_cell = Cell::from("");
        let row3 = Row::new(vec![row3_cell]);
        rows.push(row3);
    }

    rows
}

/// Context for rendering the clip list
pub struct ClipListRenderContext<'a> {
    pub selected: usize,
    pub mode: AppMode,
    pub search_input: &'a Input,
    pub numeric_prefix: &'a str,
    pub register_filter: RegisterFilter,
    pub view_mode: ViewMode,
    pub scroll_offset: usize,
    pub theme: &'a super::Theme,
}

/// Render the clip list widget showing clipboard history entries
/// Always shows header at top (search or title or numeric prefix) with item count
pub fn render_clip_list(
    frame: &mut Frame,
    area: Rect,
    entries: &[&ClipEntry],
    ctx: ClipListRenderContext,
) {
    // Reserve lines for header (search or title or jump mode)
    // In comfortable mode: title, empty, count, empty (4 lines)
    // In compact mode: title + count on same line (1 line)
    let header_height = match ctx.view_mode {
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
    let search_query = ctx.search_input.value();
    let (header_left, header_style) = if !ctx.numeric_prefix.is_empty() {
        // Numeric prefix mode: show the prefix being typed with space
        (format!(": {}", ctx.numeric_prefix), ctx.theme.temp_register)
    } else if matches!(ctx.mode, AppMode::Search) || !search_query.is_empty() {
        // Search mode or active search query: show "/ <query>" with filter prefix if active
        let (prefix, style) = match ctx.register_filter {
            RegisterFilter::Temporary => ("[temp]/ ", ctx.theme.temp_register),
            RegisterFilter::Permanent => ("[perm]/ ", ctx.theme.perm_register),
            RegisterFilter::Pinned => ("[pinned]/ ", ctx.theme.pin_indicator_style),
            RegisterFilter::None => ("/ ", ctx.theme.search_input),
        };

        // Show search text (cursor will be set via frame.set_cursor_position)
        let header_text = format!("{}{}", prefix, search_query);

        (header_text, style)
    } else {
        // Normal mode with no search: show "Clipboard History" or filter status
        let header = match ctx.register_filter {
            RegisterFilter::Temporary => "Temporary Registers".to_string(),
            RegisterFilter::Permanent => "Permanent Registers".to_string(),
            RegisterFilter::Pinned => "Pinned Clips".to_string(),
            RegisterFilter::None => "Clipboard History".to_string(),
        };
        let style = match ctx.register_filter {
            RegisterFilter::Temporary => ctx.theme.temp_register,
            RegisterFilter::Permanent => ctx.theme.perm_register,
            RegisterFilter::Pinned => ctx.theme.pin_indicator_style,
            RegisterFilter::None => ctx.theme.clip_list_header,
        };
        (header, style)
    };

    // Determine background for search/title text (focused when in search mode)
    let search_line_bg = if matches!(ctx.mode, AppMode::Search) {
        ctx.theme.search_focused_bg
    } else {
        ctx.theme.clip_list_bg
    };

    // Render header based on view mode using separate paragraphs
    match ctx.view_mode {
        ViewMode::Comfortable => {
            // Split header into 4 lines: search, empty, count, empty
            let header_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1), // Search/title line
                    Constraint::Length(1), // Empty
                    Constraint::Length(1), // Count line
                    Constraint::Length(1), // Empty
                ])
                .split(header_area);

            // Render search/title line with focused background
            let search_para = Paragraph::new(Line::from(Span::styled(header_left, header_style)))
                .style(Style::default().bg(search_line_bg));
            frame.render_widget(search_para, header_chunks[0]);

            // Render count line with normal background
            let count_para = Paragraph::new(Line::from(Span::styled(count_text, ctx.theme.clip_list_item_count)))
                .style(Style::default().bg(ctx.theme.clip_list_bg));
            frame.render_widget(count_para, header_chunks[2]);
        }
        ViewMode::Compact => {
            // Split header horizontally: search on left, 1-space gap, count on right
            let header_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(1),                          // Search/title (takes remaining space)
                    Constraint::Length(1),                       // 1-space gap
                    Constraint::Length(count_text.len() as u16), // Count (fixed width)
                ])
                .split(header_area);

            // Render search/title with focused background
            let search_para = Paragraph::new(Line::from(Span::styled(header_left, header_style)))
                .style(Style::default().bg(search_line_bg));
            frame.render_widget(search_para, header_chunks[0]);

            // Render count with normal background
            let count_para = Paragraph::new(Line::from(Span::styled(count_text, ctx.theme.clip_list_item_count)))
                .style(Style::default().bg(ctx.theme.clip_list_bg));
            frame.render_widget(count_para, header_chunks[2]);
        }
    }

    // Render based on view mode
    let available_width = list_area.width as usize;

    match ctx.view_mode {
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
                ctx.theme.pin_indicator.width() as u16
            } else {
                0
            };

            // Calculate available width for preview column
            // Account for: number (3) + pin (2 if any) + register col + selection indicator + spacing (6)
            let highlight_width = ctx
                .theme
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
            let rows = render_compact_table_rows(
                entries,
                ctx.selected,
                content_col_width as usize,
                ctx.theme,
            );

            // Table with 4 columns: number, pin, preview (fills space), registers
            let widths = [
                Constraint::Length(3),                  // Number
                Constraint::Length(pin_col_width),      // Pin (0 if none pinned)
                Constraint::Min(10),                    // Preview (fills remaining)
                Constraint::Length(register_col_width), // Registers
            ];
            let table = Table::new(rows, widths)
                .style(Style::default().bg(ctx.theme.clip_list_bg))
                .highlight_symbol(
                    ctx.theme
                        .selection_indicator_compact
                        .as_deref()
                        .unwrap_or(""),
                );

            let mut table_state = ratatui::widgets::TableState::default();
            table_state.select(Some(ctx.selected));
            *table_state.offset_mut() = ctx.scroll_offset;

            frame.render_stateful_widget(table, list_area, &mut table_state);
        }
        ViewMode::Comfortable => {
            // Use Table for comfortable mode
            // Indicators are manually added to row content based on selection_indicator_repeats_comfortable
            let rows =
                render_comfortable_table_rows(entries, ctx.selected, available_width, ctx.theme);

            let table = Table::new(rows, [Constraint::Min(10)])
                .style(Style::default().bg(ctx.theme.clip_list_bg));

            // Select the first row of the selected clip for scrolling purposes
            // Each clip has 3 rows (preview, metadata, spacing)
            let mut table_state = ratatui::widgets::TableState::default();
            table_state.select(Some(ctx.selected * 3));
            *table_state.offset_mut() = ctx.scroll_offset;

            frame.render_stateful_widget(table, list_area, &mut table_state);
        }
    }

    // Set cursor position when in search mode
    if matches!(ctx.mode, AppMode::Search) {
        // Calculate prefix width based on register filter
        let prefix_width = match ctx.register_filter {
            RegisterFilter::Temporary => "[temp]/ ".len(),
            RegisterFilter::Permanent => "[perm]/ ".len(),
            RegisterFilter::Pinned => "[pinned]/ ".len(),
            RegisterFilter::None => "/ ".len(),
        };

        // Get cursor position accounting for unicode width
        let cursor_x =
            header_area.x + prefix_width as u16 + ctx.search_input.visual_cursor() as u16;
        let cursor_y = header_area.y;

        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }
}
