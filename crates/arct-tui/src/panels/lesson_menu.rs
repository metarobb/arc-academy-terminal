//! Lesson selection menu panel

use crate::icons;
use crate::theme::Theme;
use arct_core::{Lesson, LessonLibrary};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use std::collections::HashSet;

/// Lesson menu panel for selecting lessons
pub struct LessonMenuPanel {
    library: LessonLibrary,
    selected_index: usize,
    scroll_offset: usize,
}

impl LessonMenuPanel {
    pub fn new() -> Self {
        Self {
            library: LessonLibrary::new(),
            selected_index: 0,
            scroll_offset: 0,
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        let total = self.library.all().len();
        if total > 0 {
            self.selected_index = if self.selected_index == 0 {
                total - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        let total = self.library.all().len();
        if total > 0 {
            self.selected_index = (self.selected_index + 1) % total;
        }
    }

    /// Ensure selected item is visible within the given height
    fn ensure_visible(&mut self, visible_height: usize) {
        // Each lesson takes 3 lines (title + description + blank line)
        let lines_per_lesson = 3;
        let visible_lessons = visible_height / lines_per_lesson;

        if visible_lessons == 0 {
            return;
        }

        // If selected is below visible area, scroll down
        if self.selected_index >= self.scroll_offset + visible_lessons {
            self.scroll_offset = self.selected_index.saturating_sub(visible_lessons - 1);
        }

        // If selected is above visible area, scroll up
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    /// Select lesson by number (1-indexed for user display)
    pub fn select_by_number(&mut self, number: usize) {
        let total = self.library.all().len();
        if number > 0 && number <= total {
            self.selected_index = number - 1;
        }
    }

    /// Get currently selected lesson
    pub fn get_selected_lesson(&self) -> Option<Lesson> {
        let lessons = self.library.all();
        lessons.get(self.selected_index).map(|&l| l.clone())
    }

    /// Render the lesson menu overlay (centered popup)
    pub fn render(
        &mut self,
        frame: &mut Frame,
        theme: &Theme,
        completed_lessons: &HashSet<String>,
        user_stats: &arct_core::UserStats,
        recommendation_engine: &arct_core::RecommendationEngine,
    ) {
        let area = Self::centered_rect(70, 60, frame.size());

        // Clear the background
        frame.render_widget(Clear, area);

        let block = Block::default()
            .title(format!(" {}Lesson Selection Menu ", icons::lesson().content))
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
                Constraint::Length(3),   // Header info
                Constraint::Length(8),   // Recommended lessons
                Constraint::Min(10),     // Lesson list
                Constraint::Length(4),   // Controls help
            ])
            .split(inner);

        // Header
        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Select a lesson to begin your learning journey!", theme.style_normal()),
            ]),
            Line::from(vec![
                Span::styled("Completed lessons are marked with ", theme.style_dim()),
                icons::celebration(),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(header, chunks[0]);

        // Recommended lessons section
        let recommendations = recommendation_engine.get_recommendations(
            completed_lessons,
            user_stats,
            3,  // Show top 3 recommendations
        );

        let mut rec_items = Vec::new();
        rec_items.push(ListItem::new(Line::from(vec![
            icons::target(),
            Span::styled(" Recommended for You", theme.style_header()),
        ])));
        rec_items.push(ListItem::new(Line::from("")));

        if recommendations.is_empty() {
            rec_items.push(ListItem::new(Line::from(vec![
                Span::styled("  No recommendations yet. Complete some lessons to get started!", theme.style_dim()),
            ])));
        } else {
            for rec in recommendations {
                let reason_text = match rec.reason {
                    arct_core::RecommendationReason::NextInSequence => "Next in sequence",
                    arct_core::RecommendationReason::PrerequisiteSatisfied => "Prerequisites met!",
                    arct_core::RecommendationReason::SameDifficulty => "Matches your level",
                    arct_core::RecommendationReason::SkillLevelMatch => "Perfect for you",
                    arct_core::RecommendationReason::RelatedTopic => "Related topic",
                    _ => "Recommended",
                };

                let lesson_title = rec.lesson.title.clone();
                rec_items.push(ListItem::new(Line::from(vec![
                    Span::raw("  "),
                    icons::lightning(),
                    Span::styled(lesson_title, theme.style_accent()),
                    Span::raw("  "),
                    Span::styled(format!("({})", reason_text), theme.style_dim()),
                ])));
            }
        }

        let rec_list = List::new(rec_items);
        frame.render_widget(rec_list, chunks[1]);

        // Lesson list with scrolling
        let list_area_height = chunks[2].height as usize;

        // Ensure selected item is visible (do this before borrowing lessons)
        self.ensure_visible(list_area_height);

        let lessons = self.library.all();

        let mut items = Vec::new();

        // Calculate visible range
        let lines_per_lesson = 3;
        let visible_lessons = list_area_height / lines_per_lesson;
        let end_idx = (self.scroll_offset + visible_lessons).min(lessons.len());

        // Add scroll indicator at top if scrolled down
        if self.scroll_offset > 0 {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("▲ ▲ ▲  Scroll up for more lessons  ▲ ▲ ▲", theme.style_dim()),
            ])));
        }

        for (idx, lesson) in lessons.iter().enumerate().skip(self.scroll_offset).take(end_idx - self.scroll_offset) {
            let is_selected = idx == self.selected_index;
            let is_completed = completed_lessons.contains(&lesson.id);

            let number = format!("[{}] ", idx + 1);
            let status_icon = if is_completed {
                icons::celebration()
            } else {
                icons::lesson()
            };

            let difficulty_text = format!("{:?}", lesson.difficulty);
            let time_text = format!("{}min", lesson.estimated_minutes);

            let mut line_spans = vec![
                Span::styled(number, theme.style_accent()),
                status_icon,
                Span::styled(&lesson.title, if is_selected {
                    theme.style_accent().add_modifier(ratatui::style::Modifier::BOLD)
                } else {
                    theme.style_normal()
                }),
            ];

            // Add difficulty and time
            let padding = " ".repeat(40_usize.saturating_sub(lesson.title.len()));
            line_spans.push(Span::raw(padding));
            line_spans.push(Span::styled(
                format!(" {} | {} ", difficulty_text, time_text),
                theme.style_dim(),
            ));

            let mut description_line = vec![Span::raw("    ")];
            if is_completed {
                description_line.push(icons::success());
                description_line.push(Span::styled("Completed  ", theme.style_success()));
            }
            description_line.push(Span::styled(&lesson.description, theme.style_dim()));

            items.push(ListItem::new(vec![
                Line::from(line_spans),
                Line::from(description_line),
                Line::from(""),
            ]));
        }

        // Add scroll indicator at bottom if there are more lessons below
        if end_idx < lessons.len() {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("▼ ▼ ▼  Scroll down for more lessons  ▼ ▼ ▼", theme.style_dim()),
            ])));
        }

        let list = List::new(items);
        frame.render_widget(list, chunks[2]);

        // Controls help
        let total_lessons = lessons.len();
        let quick_select_text = if total_lessons <= 10 {
            vec![
                Span::styled("1-9", theme.style_accent()),
                Span::raw(" for lessons 1-9, "),
                Span::styled("0", theme.style_accent()),
                Span::raw(" for lesson 10  |  "),
            ]
        } else {
            vec![
                Span::styled("1-9,0", theme.style_accent()),
                Span::raw(" for first 10, use "),
                Span::styled("↑/↓", theme.style_accent()),
                Span::raw(" for others  |  "),
            ]
        };

        let mut control_line1 = vec![
            Span::styled("↑/↓", theme.style_accent()),
            Span::raw(" or "),
            Span::styled("j/k", theme.style_accent()),
            Span::raw(" navigate  |  "),
        ];
        control_line1.extend(quick_select_text);
        control_line1.push(Span::styled("Enter", theme.style_accent()));
        control_line1.push(Span::raw(" start"));

        let controls = Paragraph::new(vec![
            Line::from(control_line1),
            Line::from(vec![
                Span::styled("Esc", theme.style_accent()),
                Span::raw(" or "),
                Span::styled("q", theme.style_accent()),
                Span::raw(" close menu"),
            ]),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(controls, chunks[3]);
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

impl Default for LessonMenuPanel {
    fn default() -> Self {
        Self::new()
    }
}
