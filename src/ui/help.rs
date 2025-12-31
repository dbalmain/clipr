use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use unicode_width::UnicodeWidthStr;

use super::Theme;
use super::layout::centered_rect;

struct HelpSection {
    title: &'static str,
    items: &'static [(&'static [&'static str], &'static str)],
}

const HELP_SECTIONS: &[HelpSection] = &[
    HelpSection {
        title: "Navigation",
        items: &[
            (&["k", "↑", "j", "↓"], "Move up/down"),
            (&["Home", "End"], "Jump to top/bottom"),
            (&["PgUp", "PgDn"], "Page up/down"),
            (&["Ctrl-u", "d"], "Half-page up/down"),
            (&[], ""),
            (&["<number>"], "Enter numeric mode (shows command palette)"),
            (&["  5j"], "move down 5 lines"),
            (&["  3Ctrl-d"], "3 half-pages down"),
            (&["  15Enter"], "jump to line 15"),
        ],
    },
    HelpSection {
        title: "Look & Feel",
        items: &[
            (&["v"], "Toggle between Compact/Comfortable view modes"),
            (&["T"], "Open theme picker"),
            (&["Ctrl-T"], "Cycle to next theme"),
            (&["Alt-T"], "Save current theme as default"),
        ],
    },
    HelpSection {
        title: "Search & Filter",
        items: &[
            (&["/"], "Start fuzzy search"),
            (&["'"], "Filter by temporary registers"),
            (&["\""], "Filter by permanent registers"),
            (&["P"], "Toggle pin filter"),
            (&["Esc"], "Clear search/filter"),
        ],
    },
    HelpSection {
        title: "Actions",
        items: &[
            (&["Enter"], "Copy to clipboard"),
            (&["Ctrl-Space"], "Paste via Ctrl-V"),
            (
                &["m<letter>"],
                "Assign to temporary register (like vim marks)",
            ),
            (&["p"], "Toggle pin"),
            (&["d"], "Delete entry"),
            (&["D"], "Clear all unpinned (with confirmation)"),
            (&["q", "Esc"], "Quit"),
        ],
    },
    HelpSection {
        title: "Help",
        items: &[(&["?"], "Show/hide this help")],
    },
];

/// Add help content with proper styling and fixed-width columns
fn add_help_content<'a>(content: &mut Vec<Line<'a>>, section: &HelpSection, theme: &Theme) {
    const KEY_COLUMN_WIDTH: usize = 20; // Fixed width for keys column

    // Add section title
    content.push(Line::from(vec![Span::styled(
        section.title,
        theme.help_header,
    )]));
    content.push(Line::default());

    // Add section items
    for (keys, description) in section.items {
        if keys.is_empty() && description.is_empty() {
            // Empty line for spacing
            content.push(Line::default());
            continue;
        }

        if keys.is_empty() {
            // Just a description line (like the numeric mode examples)
            content.push(Line::from(vec![
                Span::raw("                     "), // Indent for continuation lines
                Span::styled(*description, theme.help_desc),
            ]));
        } else {
            // Build keys string with dimmed separators
            let mut line_spans = Vec::new();
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    line_spans.push(Span::styled(
                        "/",
                        theme.help_desc.add_modifier(Modifier::DIM),
                    ));
                }
                line_spans.push(Span::styled(*key, theme.help_key));
            }

            // Calculate padding needed to align descriptions
            let keys_str: String = keys
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<_>>()
                .join("/");
            let padding = KEY_COLUMN_WIDTH.saturating_sub(keys_str.width());

            line_spans.push(Span::raw(" ".repeat(padding)));
            line_spans.push(Span::styled(*description, theme.help_desc));

            content.push(Line::from(line_spans));
        }
    }

    content.push(Line::default());
}

/// Render help overlay with keybindings
pub fn render_help_overlay(frame: &mut Frame, area: Rect, theme: &Theme) {
    let overlay_area = centered_rect(60, 80, area);

    // Clear the background area first to hide underlying content
    frame.render_widget(Clear, overlay_area);

    // Build help content from sections
    let mut content = Vec::new();

    // Add title
    content.push(Line::from(vec![Span::styled("Help", theme.help_title)]));
    content.push(Line::default());

    // Add sections
    for section in HELP_SECTIONS {
        add_help_content(&mut content, section, theme);
    }

    // Add footer
    content.push(Line::from(vec![Span::styled(
        "Press any key to close...",
        theme.help_footer,
    )]));

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .style(Style::default().bg(theme.help_modal_bg))
                .padding(ratatui::widgets::Padding::uniform(2)),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, overlay_area);
}
