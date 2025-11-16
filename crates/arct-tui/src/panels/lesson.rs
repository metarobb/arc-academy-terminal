//! Lesson panel - displays interactive lessons and tracks progress

use crate::icons;
use crate::theme::Theme;
use arct_core::{Lesson, LessonStep, StepType, ValidationResult, LessonValidator};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

/// Lesson panel state
pub struct LessonPanel {
    pub current_lesson: Option<Lesson>,
    current_step_index: usize,
    user_input: String,
    validator: LessonValidator,
    last_validation: Option<ValidationResult>,
    completed_steps: Vec<usize>,
}

impl LessonPanel {
    pub fn new() -> Self {
        Self {
            current_lesson: None,
            current_step_index: 0,
            user_input: String::new(),
            validator: LessonValidator::new(),
            last_validation: None,
            completed_steps: Vec::new(),
        }
    }

    /// Load a lesson
    pub fn load_lesson(&mut self, lesson: Lesson) {
        self.current_lesson = Some(lesson);
        self.current_step_index = 0;
        self.user_input.clear();
        self.last_validation = None;
        self.completed_steps.clear();
    }

    /// Get current step
    fn current_step(&self) -> Option<&LessonStep> {
        self.current_lesson
            .as_ref()
            .and_then(|lesson| lesson.steps.get(self.current_step_index))
    }

    /// Check if user input is valid for current step
    pub fn validate_current_step(&mut self, input: &str) -> ValidationResult {
        if let Some(step) = self.current_step() {
            let result = match &step.step_type {
                StepType::CommandExercise {
                    expected_command,
                    validation,
                    success_message,
                } => {
                    let validation_result =
                        self.validator.validate_command(input, expected_command, validation);

                    if validation_result.is_success() {
                        ValidationResult::Success {
                            message: success_message.clone(),
                        }
                    } else {
                        validation_result
                    }
                }
                StepType::MultipleChoice {
                    correct_index, ..
                } => {
                    if let Ok(choice) = input.parse::<usize>() {
                        self.validator.validate_multiple_choice(choice, *correct_index)
                    } else {
                        ValidationResult::Failure {
                            message: "Please enter a number.".to_string(),
                            hint: None,
                        }
                    }
                }
                StepType::Information { .. } => {
                    // Information steps just need any key press to continue
                    ValidationResult::Success {
                        message: "Continue to next step.".to_string(),
                    }
                }
                _ => ValidationResult::Success {
                    message: "Continue.".to_string(),
                },
            };

            self.last_validation = Some(result.clone());
            result
        } else {
            ValidationResult::Failure {
                message: "No active step.".to_string(),
                hint: None,
            }
        }
    }

    /// Move to next step
    pub fn next_step(&mut self) -> bool {
        if let Some(lesson) = &self.current_lesson {
            if !self.completed_steps.contains(&self.current_step_index) {
                self.completed_steps.push(self.current_step_index);
            }

            if self.current_step_index + 1 < lesson.steps.len() {
                self.current_step_index += 1;
                self.user_input.clear();
                self.last_validation = None;
                true
            } else {
                false // Lesson complete
            }
        } else {
            false
        }
    }

    /// Move to previous step
    pub fn previous_step(&mut self) {
        if self.current_step_index > 0 {
            self.current_step_index -= 1;
            self.user_input.clear();
            self.last_validation = None;
        }
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f32 {
        if let Some(lesson) = &self.current_lesson {
            let total = lesson.steps.len();
            if total == 0 {
                return 0.0;
            }
            (self.completed_steps.len() as f32 / total as f32) * 100.0
        } else {
            0.0
        }
    }

    /// Render the lesson panel
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let border_style = if focused {
            theme.style_border_focused()
        } else {
            theme.style_border()
        };

        if let Some(lesson) = &self.current_lesson {
            // Split area into header and content
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Header
                    Constraint::Min(0),    // Content
                ])
                .split(area);

            // Render header with progress
            self.render_header(frame, chunks[0], lesson, theme, border_style);

            // Render current step
            if let Some(step) = self.current_step() {
                self.render_step(frame, chunks[1], step, theme, border_style);
            }
        } else {
            // No lesson loaded - show lesson selection screen
            self.render_lesson_selection(frame, area, theme, border_style);
        }
    }

    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        lesson: &Lesson,
        theme: &Theme,
        border_style: Style,
    ) {
        let progress = self.completion_percentage();
        let step_count = format!(
            "Step {}/{}",
            self.current_step_index + 1,
            lesson.steps.len()
        );

        let title = format!(
            " {}{} | {} | {:.0}% ",
            icons::lesson().content, lesson.title, step_count, progress
        );

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        frame.render_widget(block, area);
    }

    fn render_step(
        &self,
        frame: &mut Frame,
        area: Rect,
        step: &LessonStep,
        theme: &Theme,
        border_style: Style,
    ) {
        let mut items = Vec::new();

        // Step title
        items.push(ListItem::new(Line::from(vec![
            Span::styled("━".repeat(50), theme.style_dim()),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("Step {}: ", step.step_number), theme.style_accent()),
            Span::styled(&step.title, theme.style_header()),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("━".repeat(50), theme.style_dim()),
        ])));
        items.push(ListItem::new(Line::from("")));

        // Render based on step type
        match &step.step_type {
            StepType::CommandExercise {
                success_message, ..
            } => {
                // Instruction
                if !step.instruction.is_empty() {
                    items.push(ListItem::new(Line::from(vec![
                        icons::note(),
                        Span::styled(&step.instruction, theme.style_normal()),
                    ])));
                    items.push(ListItem::new(Line::from("")));
                }

                // Hint
                if let Some(hint) = &step.hint {
                    items.push(ListItem::new(Line::from(vec![
                        icons::hint(),
                        Span::styled("Hint: ", theme.style_warning()),
                        Span::styled(hint, theme.style_dim()),
                    ])));
                    items.push(ListItem::new(Line::from("")));
                }

                // Validation result
                if let Some(validation) = &self.last_validation {
                    match validation {
                        ValidationResult::Success { message } => {
                            items.push(ListItem::new(Line::from(vec![
                                icons::success(),
                                Span::styled(message, theme.style_success()),
                            ])));
                            items.push(ListItem::new(Line::from("")));
                        }
                        ValidationResult::Failure { message, hint } => {
                            items.push(ListItem::new(Line::from(vec![
                                icons::error(),
                                Span::styled(message, theme.style_error()),
                            ])));
                            if let Some(h) = hint {
                                items.push(ListItem::new(Line::from(vec![
                                    Span::styled("   ", theme.style_normal()),
                                    icons::hint(),
                                    Span::styled(h, theme.style_dim()),
                                ])));
                            }
                            items.push(ListItem::new(Line::from("")));
                        }
                        ValidationResult::Partial { message, progress } => {
                            items.push(ListItem::new(Line::from(vec![
                                icons::warning(),
                                Span::styled(
                                    format!("{} ({:.0}%)", message, progress),
                                    theme.style_warning(),
                                ),
                            ])));
                            items.push(ListItem::new(Line::from("")));
                        }
                    }
                }

                items.push(ListItem::new(Line::from(vec![
                    Span::styled("▶ ", theme.style_accent()),
                    Span::styled(
                        "Type the command in the shell panel and press Enter",
                        theme.style_dim(),
                    ),
                ])));
            }
            StepType::MultipleChoice {
                question,
                options,
                explanation,
                ..
            } => {
                // Question
                items.push(ListItem::new(Line::from(vec![
                    icons::question(),
                    Span::styled(question, theme.style_header()),
                ])));
                items.push(ListItem::new(Line::from("")));

                // Options
                for (i, option) in options.iter().enumerate() {
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(format!("  {}. ", i), theme.style_accent()),
                        Span::styled(option, theme.style_normal()),
                    ])));
                }
                items.push(ListItem::new(Line::from("")));

                // Show explanation if answered correctly
                if let Some(ValidationResult::Success { .. }) = &self.last_validation {
                    items.push(ListItem::new(Line::from(vec![
                        icons::success(),
                        Span::styled(explanation, theme.style_success()),
                    ])));
                    items.push(ListItem::new(Line::from("")));
                }

                items.push(ListItem::new(Line::from(vec![
                    Span::styled(format!("{}  ", icons::ARROW_RIGHT), theme.style_accent()),
                    Span::styled("Enter the number of your answer", theme.style_dim()),
                ])));
            }
            StepType::Information { content } => {
                // Display information with nice formatting
                for line in content.lines() {
                    items.push(ListItem::new(Line::from(vec![Span::styled(
                        line,
                        theme.style_normal(),
                    )])));
                }
                items.push(ListItem::new(Line::from("")));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("▶ ", theme.style_accent()),
                    Span::styled("Press Enter to continue", theme.style_dim()),
                ])));
            }
            StepType::FillInBlank { template, .. } => {
                items.push(ListItem::new(Line::from(vec![
                    icons::note(),
                    Span::styled(&step.instruction, theme.style_normal()),
                ])));
                items.push(ListItem::new(Line::from("")));
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Template: ", theme.style_accent()),
                    Span::styled(template, theme.style_dim()),
                ])));
            }
            StepType::Practice { goal, hints, .. } => {
                items.push(ListItem::new(Line::from(vec![
                    icons::target(),
                    Span::styled("Goal: ", theme.style_info()),
                    Span::styled(goal, theme.style_normal()),
                ])));
                items.push(ListItem::new(Line::from("")));

                if !hints.is_empty() {
                    items.push(ListItem::new(Line::from(vec![
                        icons::hint(),
                        Span::styled("Hints:", theme.style_warning()),
                    ])));
                    for hint in hints {
                        items.push(ListItem::new(Line::from(vec![
                            Span::styled("  • ", theme.style_dim()),
                            Span::styled(hint, theme.style_dim()),
                        ])));
                    }
                }
            }
        }

        // Controls
        items.push(ListItem::new(Line::from("")));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("━".repeat(50), theme.style_dim()),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled("Controls: ", theme.style_dim()),
            Span::styled("Tab", theme.style_accent()),
            Span::styled(" = Next Panel | ", theme.style_dim()),
            Span::styled("Ctrl+L", theme.style_accent()),
            Span::styled(" = Next Step | ", theme.style_dim()),
            Span::styled("Ctrl+H", theme.style_accent()),
            Span::styled(" = Previous Step", theme.style_dim()),
        ])));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .padding(Padding::uniform(1));

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    fn render_lesson_selection(
        &self,
        frame: &mut Frame,
        area: Rect,
        theme: &Theme,
        border_style: Style,
    ) {
        let block = Block::default()
            .title(format!(" {}Interactive Lessons ", icons::lesson().content))
            .borders(Borders::ALL)
            .border_style(border_style);

        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                icons::welcome(),
                Span::styled("Welcome to Interactive Lessons!", theme.style_accent()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🎓 ", theme.style_accent()),
                Span::styled("10 comprehensive lessons", theme.style_normal()),
                Span::styled(" available", theme.style_dim()),
            ]),
            Line::from(vec![
                Span::styled("🏆 ", theme.style_accent()),
                Span::styled("Track progress & earn achievements", theme.style_normal()),
            ]),
            Line::from(vec![
                Span::styled("🛡️  ", theme.style_accent()),
                Span::styled("Safe virtual filesystem", theme.style_normal()),
                Span::styled(" for hands-on practice", theme.style_dim()),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("📚 Select a lesson:", theme.style_header()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press ", theme.style_dim()),
                Span::styled("m", theme.style_accent()),
                Span::styled(" to open the lesson menu", theme.style_dim()),
            ]),
            Line::from(vec![
                Span::styled("  Use ", theme.style_dim()),
                Span::styled("↑/↓", theme.style_accent()),
                Span::styled(" or ", theme.style_dim()),
                Span::styled("1-9,0", theme.style_accent()),
                Span::styled(" to select", theme.style_dim()),
            ]),
            Line::from(vec![
                Span::styled("  Press ", theme.style_dim()),
                Span::styled("Enter", theme.style_accent()),
                Span::styled(" to start learning!", theme.style_dim()),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(vec![
                Span::styled("💡 Tip: ", theme.style_accent()),
                Span::styled("Complete lessons to unlock achievements", theme.style_dim()),
            ]),
            Line::from(vec![
                Span::styled("       and build your learning streak!", theme.style_dim()),
            ]),
        ])
        .block(block)
        .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

impl Default for LessonPanel {
    fn default() -> Self {
        Self::new()
    }
}
