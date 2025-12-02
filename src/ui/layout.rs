use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Create main application layout with clip list, divider, preview, and keyboard hints
/// Returns [clip_list_area, divider_area, preview_area, keyboard_hints_area]
pub fn create_main_layout(area: Rect) -> Vec<Rect> {
    // Split vertically: content area + keyboard hints bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),      // Main content area
            Constraint::Length(1),   // Keyboard hints bar
        ])
        .split(area);

    // Split content horizontally: clip list + divider + preview
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Clip list (left)
            Constraint::Length(1),       // Divider line
            Constraint::Min(10),         // Preview (right - remaining space)
        ])
        .split(main_chunks[0]);

    vec![content_chunks[0], content_chunks[1], content_chunks[2], main_chunks[1]]
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
