//! Help overlay panel

use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Help panel
pub struct HelpPanel;

impl HelpPanel {
    pub fn new() -> Self {
        Self
    }

    /// Render the help overlay (centered popup)
    pub fn render(&self, frame: &mut Frame, theme: &Theme) {
        let area = Self::centered_rect(60, 80, frame.size());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(" Arc Academy Terminal - Help ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme.style_border_focused())
            .style(theme.style_block());  // Set background for light themes

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Navigation
                Constraint::Length(6),  // Editing
                Constraint::Length(10), // Actions
                Constraint::Min(1),     // Info
            ])
            .split(inner);

        // Navigation section
        let nav_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("Navigation", theme.style_header()),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Tab", theme.style_accent()),
                Span::raw(" / "),
                Span::styled("Shift+Tab", theme.style_accent()),
                Span::raw("  →  Switch panels"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Ctrl+↑/↓", theme.style_accent()),
                Span::raw("         →  Scroll output (any panel)"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  ↑/↓", theme.style_accent()),
                Span::raw(" or "),
                Span::styled("j/k", theme.style_accent()),
                Span::raw("       →  Scroll (when Output focused)"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  PgUp/PgDn", theme.style_accent()),
                Span::raw("        →  Page scroll output"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Mouse", theme.style_accent()),
                Span::raw("             →  Click to focus, wheel to scroll"),
            ])),
        ];
        frame.render_widget(List::new(nav_items), chunks[0]);

        // Editing section
        let edit_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("Command Editing", theme.style_header()),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Type", theme.style_accent()),
                Span::raw("              →  Enter command"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Enter", theme.style_accent()),
                Span::raw("            →  Explain command"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Backspace", theme.style_accent()),
                Span::raw("        →  Delete character"),
            ])),
        ];
        frame.render_widget(List::new(edit_items), chunks[1]);

        // Actions section
        let action_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("Actions", theme.style_header()),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Ctrl+k", theme.style_accent()),
                Span::raw("           →  Command palette (all actions)"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Ctrl+t", theme.style_accent()),
                Span::raw("           →  Toggle theme"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Ctrl+l", theme.style_accent()),
                Span::raw(" / "),
                Span::styled("m", theme.style_accent()),
                Span::raw("       →  Lesson mode / lesson map"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Alt+←", theme.style_accent()),
                Span::raw(" / "),
                Span::styled("Alt+r", theme.style_accent()),
                Span::raw("    →  Previous lesson step / restart lesson"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  Alt+a/p/c", theme.style_accent()),
                Span::raw("        →  Achievements / Progress / Challenges"),
            ])),
            ListItem::new(Line::from(vec![
                Span::styled("  q", theme.style_accent()),
                Span::raw(" or "),
                Span::styled("Ctrl+c", theme.style_accent()),
                Span::raw("    →  Quit"),
            ])),
        ];
        frame.render_widget(List::new(action_items), chunks[2]);

        // Info section
        let info = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", theme.style_dim()),
                Span::styled("?", theme.style_accent()),
                Span::styled(" or ", theme.style_dim()),
                Span::styled("Esc", theme.style_accent()),
                Span::styled(" to close this help", theme.style_dim()),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(info, chunks[3]);
    }

    /// Helper function to create a centered rectangle
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
}

impl Default for HelpPanel {
    fn default() -> Self {
        Self::new()
    }
}
