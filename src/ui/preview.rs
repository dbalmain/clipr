use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Wrap};
use ratatui_image::StatefulImage;
use ratatui_image::protocol::StatefulProtocol;

use super::Theme;
use crate::models::{ClipContent, ClipEntry};

/// Render preview panel with content at top and metadata at bottom
pub fn render_preview(
    frame: &mut Frame,
    area: Rect,
    entry: Option<&ClipEntry>,
    cached_image: Option<&mut StatefulProtocol>,
    show_metadata: bool,
    theme: &Theme,
) {
    if let Some(entry) = entry {
        // Calculate metadata height if metadata is enabled
        let (content_area, metadata_area) = if show_metadata {
            // Fixed 4 rows for metadata to prevent jumping
            // Will expand if description is multiline
            let metadata_lines_count = if let Some(desc) = &entry.description {
                let desc_lines = desc.lines().count();
                if desc_lines > 1 {
                    3 + desc_lines // name+size, mime-type, description (multiline), registers
                } else {
                    4 // name+size, mime-type, description, registers
                }
            } else {
                4 // name+size, mime-type, (empty), registers
            };

            // Split area: content (top) + metadata (bottom, no separator)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),                              // Content area
                    Constraint::Length(metadata_lines_count as u16), // Metadata (fixed height)
                ])
                .split(area);

            (chunks[0], Some(chunks[1]))
        } else {
            // No metadata, use full area for content
            (area, None)
        };

        // === RENDER CONTENT ===
        let mut content_lines = Vec::new();
        let mut image_rendered = false;

        match &entry.content {
            ClipContent::Text(text) => {
                for line in text.lines() {
                    content_lines.push(Line::from(line.to_string()));
                }
            }
            ClipContent::Image { .. } => {
                // Check if we have a cached protocol image for this clip
                if let Some(protocol_image) = cached_image {
                    // Render the cached image
                    let image_widget = StatefulImage::new(None);
                    frame.render_stateful_widget(image_widget, content_area, protocol_image);
                    image_rendered = true;
                } else {
                    // No cached image yet - show loading message
                    content_lines.push(Line::from(Span::styled(
                        "[Loading image...]",
                        theme.preview_metadata_label,
                    )));
                }
            }
            ClipContent::File { path, .. } => {
                content_lines.push(Line::from(vec![
                    Span::styled("File: ", theme.preview_metadata_label),
                    Span::raw(path.to_string_lossy()),
                ]));
            }
        }

        // Only render text content if we didn't render an image
        if !image_rendered {
            let content_para = Paragraph::new(content_lines)
                .style(theme.preview_text.bg(theme.preview_bg))
                .wrap(Wrap { trim: false });
            frame.render_widget(content_para, content_area);
        }

        // === RENDER METADATA (if enabled) ===
        if let Some(metadata_area) = metadata_area {
            let mut metadata_lines = Vec::new();

            // Line 1: Name (bold) + size (right-aligned)
            let name = entry.name.as_deref().unwrap_or("[unnamed]");
            let size_info = match &entry.content {
                ClipContent::Text(text) => format!("{} bytes", text.len()),
                ClipContent::Image { data, .. } => format!("{} bytes", data.len()),
                ClipContent::File { .. } => "file".to_string(),
            };

            let available_width = area.width as usize;
            let name_width = name.len();
            let size_width = size_info.len();
            let padding = if name_width + size_width < available_width {
                available_width - name_width - size_width
            } else {
                1
            };

            metadata_lines.push(Line::from(vec![
                Span::styled(
                    name,
                    theme.preview_metadata_value.add_modifier(Modifier::BOLD),
                ),
                Span::raw(" ".repeat(padding)),
                Span::styled(size_info, theme.preview_metadata_label),
            ]));

            // Line 2: Mime-type
            let mime_type = match &entry.content {
                ClipContent::Text(_) => "text/plain",
                ClipContent::Image { mime_type, .. } => mime_type,
                ClipContent::File { mime_type, .. } => mime_type,
            };
            metadata_lines.push(Line::from(Span::styled(
                mime_type,
                theme.preview_metadata_label,
            )));

            // Line 3: Description (always present, may be empty or multiline)
            if let Some(desc) = &entry.description {
                // Handle multiline descriptions
                for line in desc.lines() {
                    metadata_lines
                        .push(Line::from(Span::styled(line, theme.preview_metadata_label)));
                }
            } else {
                // Empty line to maintain 4-row height
                metadata_lines.push(Line::from(""));
            }

            // Line 4: Registers (always present, may be empty)
            if !entry.temporary_registers.is_empty() || !entry.permanent_registers.is_empty() {
                let mut register_spans = Vec::new();

                // Temporary registers with single quotes
                for (i, &reg) in entry.temporary_registers.iter().enumerate() {
                    if i > 0 {
                        register_spans.push(Span::raw(" "));
                    }
                    register_spans.push(Span::styled(format!("'{}", reg), theme.temp_register));
                }

                if !entry.temporary_registers.is_empty() && !entry.permanent_registers.is_empty() {
                    register_spans.push(Span::raw("  "));
                }

                // Permanent registers with double quotes
                for (i, &reg) in entry.permanent_registers.iter().enumerate() {
                    if i > 0 {
                        register_spans.push(Span::raw(" "));
                    }
                    register_spans.push(Span::styled(format!("\"{}", reg), theme.perm_register));
                }

                metadata_lines.push(Line::from(register_spans));
            } else {
                // Empty line to maintain 4-row height
                metadata_lines.push(Line::from(""));
            }

            let metadata_para =
                Paragraph::new(metadata_lines).style(Style::default().bg(theme.preview_bg));
            frame.render_widget(metadata_para, metadata_area);
        }
    } else {
        let msg =
            Paragraph::new("No selection").style(theme.preview_metadata_label.bg(theme.preview_bg));
        frame.render_widget(msg, area);
    }
}
