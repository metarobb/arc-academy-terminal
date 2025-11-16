//! Panel definitions and implementations

pub mod shell;
pub mod context;
pub mod explanation;
pub mod help;
pub mod lesson;
pub mod lesson_menu;
pub mod onboarding;
pub mod settings;
pub mod achievements;
pub mod progress;
pub mod challenges;
pub mod notification;

use ratatui::{layout::Rect, Frame};

/// Panel identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelId {
    Shell,
    Output,
    Context,
    Explanation,
}

impl PanelId {
    /// Get the next panel in order
    pub fn next(self) -> Self {
        match self {
            Self::Shell => Self::Output,
            Self::Output => Self::Context,
            Self::Context => Self::Explanation,
            Self::Explanation => Self::Shell,
        }
    }

    /// Get the previous panel in order
    pub fn previous(self) -> Self {
        match self {
            Self::Shell => Self::Explanation,
            Self::Output => Self::Shell,
            Self::Context => Self::Output,
            Self::Explanation => Self::Context,
        }
    }

    /// Get panel title
    pub fn title(self) -> &'static str {
        match self {
            Self::Shell => "Shell",
            Self::Output => "Output",
            Self::Context => "Context",
            Self::Explanation => "Learning",
        }
    }
}

/// Trait for all panels
pub trait Panel {
    /// Render the panel
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool);

    /// Handle input (if focused)
    fn handle_input(&mut self, key: crossterm::event::KeyEvent) -> bool;
}
