//! Achievement notification popup for celebrating unlocks

use crate::icons;
use crate::theme::Theme;
use arct_core::Achievement;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Notification panel for displaying achievement unlocks
pub struct NotificationPanel {
    achievement: Achievement,
}

impl NotificationPanel {
    pub fn new(achievement: Achievement) -> Self {
        Self { achievement }
    }

    /// Render the notification as a centered overlay popup
    pub fn render(&self, frame: &mut Frame, theme: &Theme) {
        let area = Self::centered_rect(50, 30, frame.size());

        // Clear the background
        frame.render_widget(Clear, area);

        // Use a double border with celebration theme
        let block = Block::default()
            .title(" ACHIEVEMENT UNLOCKED! ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme.style_accent().add_modifier(ratatui::style::Modifier::BOLD));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Celebration header
                Constraint::Length(4),  // Achievement info
                Constraint::Min(2),     // Description
                Constraint::Length(3),  // Points and dismiss
            ])
            .split(inner);

        // Celebration header
        let celebration = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("", theme.style_accent()),
                Span::raw("  "),
                Span::styled("", theme.style_accent()),
                Span::raw("  "),
                Span::styled("", theme.style_accent()),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(celebration, chunks[0]);

        // Achievement info (icon + title)
        let achievement_info = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    self.achievement.icon,
                    theme.style_accent().add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::styled(
                    &self.achievement.title,
                    theme.style_accent().add_modifier(ratatui::style::Modifier::BOLD),
                ),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(achievement_info, chunks[1]);

        // Description
        let description = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                &self.achievement.description,
                theme.style_secondary(),
            )]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(description, chunks[2]);

        // Points and dismiss instruction
        let footer = Paragraph::new(vec![
            Line::from(vec![
                icons::celebration(),
                Span::styled(
                    format!("+{} points!", self.achievement.points),
                    theme.style_accent(),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press any key to continue", theme.style_dim()),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[3]);

        // Add decorative border effect
        // Draw corner decorations
        if area.width > 4 && area.height > 2 {
            // Top corners
            frame.render_widget(
                Paragraph::new("").block(
                    Block::default()
                        .borders(Borders::NONE)
                        .border_style(theme.style_accent()),
                ),
                Rect::new(area.x, area.y, 2, 1),
            );
            frame.render_widget(
                Paragraph::new("").block(
                    Block::default()
                        .borders(Borders::NONE)
                        .border_style(theme.style_accent()),
                ),
                Rect::new(area.x + area.width - 2, area.y, 2, 1),
            );
        }
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
