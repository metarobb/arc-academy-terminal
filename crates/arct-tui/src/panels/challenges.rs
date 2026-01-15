//! Challenges panel for displaying daily and weekly challenges

use crate::icons;
use crate::theme::Theme;
use arct_core::{Challenge, ChallengeManager, ChallengeType};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Challenges panel for displaying active challenges
pub struct ChallengesPanel;

impl ChallengesPanel {
    pub fn new() -> Self {
        Self
    }

    /// Render the challenges overlay (centered popup)
    pub fn render(&self, frame: &mut Frame, theme: &Theme, challenge_manager: &ChallengeManager) {
        let area = Self::centered_rect(70, 55, frame.size());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(" {}Daily & Weekly Challenges ", icons::target().content))
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
                Constraint::Length(3),  // Header
                Constraint::Length(10), // Daily challenge
                Constraint::Length(10), // Weekly challenge
                Constraint::Min(2),     // Info
                Constraint::Length(2),  // Controls
            ])
            .split(inner);

        // Header
        let header = Paragraph::new(vec![Line::from(vec![Span::styled(
            "Complete challenges to earn extra points and unlock special achievements!",
            theme.style_normal(),
        )])])
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Daily challenge
        let mut daily_challenge = challenge_manager.daily_challenge.clone();
        if daily_challenge.is_none() {
            // Generate one if not exists (should already be done)
            daily_challenge = Some(Challenge {
                id: "daily_placeholder".to_string(),
                title: "Loading...".to_string(),
                description: "Generating today's challenge...".to_string(),
                difficulty: arct_core::Difficulty::Beginner,
                challenge_type: ChallengeType::DailyCommand {
                    command: "ls".to_string(),
                    task_description: "List files".to_string(),
                    validation: arct_core::CommandValidation::Exact,
                },
                points: 10,
            });
        }

        let daily = daily_challenge.as_ref().unwrap();
        let daily_completed = challenge_manager.is_daily_completed();

        let daily_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("  TODAY'S DAILY CHALLENGE", theme.style_header()),
                if daily_completed {
                    icons::success()
                } else {
                    Span::raw("")
                },
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                icons::target(),
                Span::styled(&daily.title, theme.style_accent()),
                Span::raw("  "),
                Span::styled(
                    format!("(+{} pts)", daily.points),
                    theme.style_dim(),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled(&daily.description, theme.style_secondary()),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::raw("    "),
                if daily_completed {
                    icons::celebration()
                } else {
                    icons::hint()
                },
                Span::styled(
                    if daily_completed {
                        "Completed! Come back tomorrow for a new challenge."
                    } else {
                        "Complete this challenge in lesson mode to earn points!"
                    },
                    if daily_completed {
                        theme.style_success()
                    } else {
                        theme.style_dim()
                    },
                ),
            ])),
        ];

        let daily_list = List::new(daily_items);
        frame.render_widget(daily_list, chunks[1]);

        // Weekly challenge
        let mut weekly_challenge = challenge_manager.weekly_challenge.clone();
        if weekly_challenge.is_none() {
            weekly_challenge = Some(Challenge {
                id: "weekly_placeholder".to_string(),
                title: "Loading...".to_string(),
                description: "Generating this week's challenge...".to_string(),
                difficulty: arct_core::Difficulty::Intermediate,
                challenge_type: ChallengeType::WeeklyScenario {
                    scenario: "Practice basic navigation".to_string(),
                    steps: vec![],
                },
                points: 50,
            });
        }

        let weekly = weekly_challenge.as_ref().unwrap();
        let weekly_completed = challenge_manager.is_weekly_completed();

        let weekly_items = vec![
            ListItem::new(Line::from(vec![
                Span::styled("  THIS WEEK'S CHALLENGE", theme.style_header()),
                if weekly_completed {
                    icons::success()
                } else {
                    Span::raw("")
                },
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::raw("  "),
                icons::lightning(),
                Span::styled(&weekly.title, theme.style_accent()),
                Span::raw("  "),
                Span::styled(
                    format!("(+{} pts)", weekly.points),
                    theme.style_dim(),
                ),
            ])),
            ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled(&weekly.description, theme.style_secondary()),
            ])),
            ListItem::new(Line::from("")),
            ListItem::new(Line::from(vec![
                Span::raw("    "),
                if weekly_completed {
                    icons::celebration()
                } else {
                    icons::hint()
                },
                Span::styled(
                    if weekly_completed {
                        "Completed! New challenge arrives next week."
                    } else {
                        "Complete this multi-step challenge for big rewards!"
                    },
                    if weekly_completed {
                        theme.style_success()
                    } else {
                        theme.style_dim()
                    },
                ),
            ])),
        ];

        let weekly_list = List::new(weekly_items);
        frame.render_widget(weekly_list, chunks[2]);

        // Info
        let info = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Tip: ", theme.style_accent()),
                Span::styled(
                    "Complete challenges to unlock special achievements and climb the leaderboard!",
                    theme.style_dim(),
                ),
            ]),
        ]);
        frame.render_widget(info, chunks[3]);

        // Controls
        let controls = Paragraph::new(vec![Line::from(vec![
            Span::styled("Esc", theme.style_accent()),
            Span::raw(" or "),
            Span::styled("c", theme.style_accent()),
            Span::raw(" to close"),
        ])])
        .alignment(Alignment::Center);
        frame.render_widget(controls, chunks[4]);
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

impl Default for ChallengesPanel {
    fn default() -> Self {
        Self::new()
    }
}
