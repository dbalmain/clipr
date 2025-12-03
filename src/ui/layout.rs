use ratatui::layout::{Constraint, Direction, Layout, Rect};
use crate::app::ViewMode;

/// Create main application layout with clip list, divider, preview, and keyboard hints
/// Returns [clip_list_area, divider_area, preview_area, keyboard_hints_area]
pub fn create_main_layout(area: Rect, view_mode: ViewMode) -> Vec<Rect> {
    let working_area = if matches!(view_mode, ViewMode::Comfortable) {
        // Add margins around the entire UI in comfortable mode
        let margin_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),   // Top margin
                Constraint::Min(3),      // Content
                Constraint::Length(1),   // Bottom margin
            ])
            .split(area);

        let horizontal_margin = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),   // Left margin
                Constraint::Min(10),     // Content
                Constraint::Length(2),   // Right margin
            ])
            .split(margin_chunks[1]);

        horizontal_margin[1]
    } else {
        // No margins in compact mode
        area
    };

    // Split vertically: content area + keyboard hints bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Main content area
            Constraint::Length(1),   // Spacing before hints
            Constraint::Length(1),   // Keyboard hints bar
        ])
        .split(working_area);

    // Split content horizontally: clip list + divider + preview
    let divider_width = if matches!(view_mode, ViewMode::Comfortable) {
        3 // More spacing in comfortable mode
    } else {
        1 // Single line in compact mode
    };

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),         // Clip list (left)
            Constraint::Length(divider_width), // Divider line
            Constraint::Min(10),                // Preview (right - remaining space)
        ])
        .split(main_chunks[0]);

    vec![content_chunks[0], content_chunks[1], content_chunks[2], main_chunks[2]]
}

/// Create centered rectangle for popups/overlays
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
