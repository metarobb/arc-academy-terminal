//! Progress dashboard panel for displaying user statistics

use crate::icons;
use crate::theme::Theme;
use arct_core::{UserStats, Difficulty};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Progress panel for displaying user stats and progress
pub struct ProgressPanel;

impl ProgressPanel {
    pub fn new() -> Self {
        Self
    }

    /// Render the progress dashboard overlay (centered popup)
    pub fn render(&self, frame: &mut Frame, theme: &Theme, stats: &UserStats) {
        let area = Self::centered_rect(70, 60, frame.size());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(" {}Your Progress Dashboard ", icons::target().content))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(theme.style_border_focused());

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Overview stats
                Constraint::Length(10), // Progress bars by difficulty
                Constraint::Min(3),     // Commands section
                Constraint::Length(2),  // Controls help
            ])
            .split(inner);

        // Overview stats
        let overview_items = vec![
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled("", theme.style_accent()),
                Span::raw("  Current Streak: "),
                Span::styled(
                    format!("{} days", stats.current_streak),
                    theme.style_accent().add_modifier(ratatui::style::Modifier::BOLD),
                ),
                Span::raw("  (Best: "),
                Span::styled(
                    format!("{}", stats.longest_streak),
                    theme.style_dim(),
                ),
                Span::raw(" days)"),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                icons::lesson(),
                Span::raw("  Lessons Completed: "),
                Span::styled(
                    format!("{}", stats.lessons_completed.len()),
                    theme.style_accent(),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                icons::shell(),
                Span::raw("  Commands Mastered: "),
                Span::styled(
                    format!("{}", stats.commands_used.len()),
                    theme.style_accent(),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                icons::celebration(),
                Span::raw("  Achievements Unlocked: "),
                Span::styled(
                    format!("{}", stats.achievements.total_unlocked()),
                    theme.style_accent(),
                ),
                Span::raw(" ("),
                Span::styled(
                    format!("{} points", stats.achievements.total_points()),
                    theme.style_dim(),
                ),
                Span::raw(")"),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                Span::styled(icons::TIMER, theme.style_accent()),
                Span::raw("  Time Invested: "),
                Span::styled(
                    format_time(stats.total_time_seconds),
                    theme.style_dim(),
                ),
            ])),
        ];

        let overview_list = List::new(overview_items);
        frame.render_widget(overview_list, chunks[0]);

        // Progress by difficulty
        let difficulty_section = Self::render_difficulty_progress(theme, stats);
        frame.render_widget(difficulty_section, chunks[1]);

        // Commands section
        let command_text = if stats.commands_used.is_empty() {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  No commands used yet. ", theme.style_dim()),
                    Span::styled("Start practicing to see your progress!", theme.style_normal()),
                ]),
            ]
        } else {
            let mut lines = vec![
                Line::from(vec![
                    Span::styled("  Most Used Commands:", theme.style_header()),
                ]),
                Line::from(""),
            ];

            // Show first 5 commands (alphabetically sorted)
            let mut commands: Vec<_> = stats.commands_used.iter().collect();
            commands.sort();
            for cmd in commands.iter().take(5) {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    icons::shell(),
                    Span::styled(cmd.to_string(), theme.style_accent()),
                ]));
            }

            if commands.len() > 5 {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(
                        format!("... and {} more", commands.len() - 5),
                        theme.style_dim(),
                    ),
                ]));
            }

            lines
        };

        let commands_para = Paragraph::new(command_text);
        frame.render_widget(commands_para, chunks[2]);

        // Controls help
        let controls = Paragraph::new(vec![Line::from(vec![
            Span::styled("Esc", theme.style_accent()),
            Span::raw(" or "),
            Span::styled("p", theme.style_accent()),
            Span::raw(" to close"),
        ])])
        .alignment(Alignment::Center);
        frame.render_widget(controls, chunks[3]);
    }

    /// Render progress bars by difficulty level
    fn render_difficulty_progress(theme: &Theme, stats: &UserStats) -> Paragraph<'static> {
        // Assume 10 lessons per difficulty for now (would need lesson library to get actual count)
        let total_per_difficulty = 10;

        let difficulties = vec![
            (Difficulty::Beginner, "Beginner"),
            (Difficulty::Intermediate, "Intermediate"),
            (Difficulty::Advanced, "Advanced"),
        ];

        let mut lines = vec![
            Line::from(vec![
                Span::styled("  Progress by Difficulty:", theme.style_header()),
            ]),
            Line::from(""),
        ];

        for (difficulty, name) in difficulties {
            let completed = stats
                .lessons_by_difficulty
                .get(&difficulty)
                .copied()
                .unwrap_or(0);

            let percentage = (completed as f64 / total_per_difficulty as f64 * 100.0).min(100.0);
            let bar_width = 30;
            let filled = (percentage / 100.0 * bar_width as f64) as usize;
            let empty = bar_width - filled;

            let bar_filled = "█".repeat(filled);
            let bar_empty = "░".repeat(empty);

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("{:12}", name), theme.style_normal()),
                Span::raw("  ["),
                Span::styled(bar_filled, theme.style_accent()),
                Span::styled(bar_empty, theme.style_dim()),
                Span::raw("]  "),
                Span::styled(
                    format!("{}/{} ({:.0}%)", completed, total_per_difficulty, percentage),
                    theme.style_dim(),
                ),
            ]));
        }

        Paragraph::new(lines)
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

impl Default for ProgressPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Format seconds into human-readable time
fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
