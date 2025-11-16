//! Achievements panel for displaying unlocked and locked achievements

use crate::icons;
use crate::theme::Theme;
use arct_core::{AchievementCategory, UserAchievements, all_achievements};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Achievements panel for displaying achievement progress
pub struct AchievementsPanel {
    scroll_offset: usize,
}

impl AchievementsPanel {
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    /// Scroll up
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Render the achievements overlay (centered popup)
    pub fn render(&self, frame: &mut Frame, theme: &Theme, user_achievements: &UserAchievements) {
        let area = Self::centered_rect(75, 70, frame.size());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(
                " {}Achievements ({}/{}) - {} Points ",
                icons::celebration().content,
                user_achievements.total_unlocked(),
                all_achievements().len(),
                user_achievements.total_points()
            ))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme.style_border_focused());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header info
                Constraint::Min(10),    // Achievement list
                Constraint::Length(2),  // Controls help
            ])
            .split(inner);

        // Header
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(
                    "Complete lessons, use commands, and maintain streaks to unlock achievements!",
                    theme.style_normal(),
                ),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Group achievements by category
        let all_achievements = all_achievements();
        let categories = vec![
            AchievementCategory::Lessons,
            AchievementCategory::Commands,
            AchievementCategory::Streaks,
            AchievementCategory::Challenges,
            AchievementCategory::Exploration,
        ];

        let mut items = Vec::new();

        for category in categories {
            // Category header
            let category_name = match category {
                AchievementCategory::Lessons => "Learning Journey",
                AchievementCategory::Commands => "Command Mastery",
                AchievementCategory::Streaks => "Consistency",
                AchievementCategory::Challenges => "Challenges",
                AchievementCategory::Exploration => "Explorer",
            };

            items.push(ListItem::new(vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled(
                        format!("━━━ {} ━━━", category_name),
                        theme.style_header(),
                    ),
                ]),
            ]));

            // Achievements in this category
            let category_achievements: Vec<_> = all_achievements
                .iter()
                .filter(|a| a.category == category)
                .collect();

            for achievement in category_achievements {
                let is_unlocked = user_achievements.is_unlocked(&achievement.id);

                let status_icon = if is_unlocked {
                    icons::success()
                } else {
                    Span::styled("  ", theme.style_dim())
                };

                let title_style = if is_unlocked {
                    theme.style_accent()
                } else {
                    theme.style_dim()
                };

                let points_text = format!("{}pts", achievement.points);

                let line_spans = vec![
                    Span::raw("  "),
                    status_icon,
                    Span::raw(" "),
                    Span::styled(achievement.icon, theme.style_accent()),
                    Span::raw("  "),
                    Span::styled(&achievement.title, title_style),
                    Span::raw("  "),
                    Span::styled(
                        format!("({})", points_text),
                        theme.style_dim(),
                    ),
                ];

                let description_style = if is_unlocked {
                    theme.style_secondary()
                } else {
                    theme.style_dim()
                };

                items.push(ListItem::new(vec![
                    Line::from(line_spans),
                    Line::from(vec![
                        Span::raw("      "),
                        Span::styled(&achievement.description, description_style),
                    ]),
                    Line::from(""),
                ]));
            }
        }

        let list = List::new(items);
        frame.render_widget(list, chunks[1]);

        // Controls help
        let controls = Paragraph::new(vec![Line::from(vec![
            Span::styled("↑/↓", theme.style_accent()),
            Span::raw(" to scroll  |  "),
            Span::styled("Esc", theme.style_accent()),
            Span::raw(" or "),
            Span::styled("a", theme.style_accent()),
            Span::raw(" to close"),
        ])])
        .alignment(Alignment::Center);
        frame.render_widget(controls, chunks[2]);
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

impl Default for AchievementsPanel {
    fn default() -> Self {
        Self::new()
    }
}
