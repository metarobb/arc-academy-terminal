//! Event handling for the TUI

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

/// Events that can occur in the application
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminal key press
    Key(KeyEvent),

    /// Terminal resize
    Resize(u16, u16),

    /// Application tick (for animations/updates)
    Tick,

    /// Request to quit
    Quit,
}

/// Key actions for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    NextPanel,
    PreviousPanel,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    Enter,
    Escape,
    Help,
    CommandPalette,
    ToggleTheme,
    ToggleAI,
    ToggleSettings,
    ToggleLesson,
    ShowLessonMenu,
    ShowAchievements,
    ShowProgress,
    ShowChallenges,
    DismissNotification,
    None,
}

/// Event handler that polls for terminal events
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    /// Start the event polling loop
    pub async fn start(&self) {
        let sender = self.sender.clone();

        tokio::spawn(async move {
            loop {
                // Poll for events with a timeout
                if event::poll(Duration::from_millis(100)).unwrap() {
                    match event::read().unwrap() {
                        CrosstermEvent::Key(key) => {
                            if sender.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        CrosstermEvent::Resize(width, height) => {
                            if sender.send(Event::Resize(width, height)).is_err() {
                                break;
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Send tick event
                    if sender.send(Event::Tick).is_err() {
                        break;
                    }
                }
            }
        });
    }

    /// Receive the next event
    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert key events to actions
pub fn key_to_action(key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Quit,
        (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,
        (KeyCode::Esc, _) => Action::Escape,

        // Navigation
        (KeyCode::Tab, KeyModifiers::NONE) => Action::NextPanel,
        (KeyCode::BackTab, KeyModifiers::SHIFT) => Action::PreviousPanel,

        // Scrolling
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => Action::ScrollUp,
        (KeyCode::Down, _) | (KeyCode::Char('j'), KeyModifiers::NONE) => Action::ScrollDown,
        (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => Action::PageUp,
        (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => Action::PageDown,

        // Actions
        (KeyCode::Enter, _) => Action::Enter,
        (KeyCode::Char('?'), KeyModifiers::NONE) => Action::Help,
        (KeyCode::Char('k'), KeyModifiers::CONTROL) => Action::CommandPalette,
        (KeyCode::Char('t'), KeyModifiers::CONTROL) => Action::ToggleTheme,
        (KeyCode::Char('a'), KeyModifiers::CONTROL) => Action::ToggleAI,
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => Action::ToggleSettings,
        (KeyCode::Char('l'), KeyModifiers::CONTROL) => Action::ToggleLesson,
        (KeyCode::Char('m'), KeyModifiers::NONE) => Action::ShowLessonMenu,

        // Gamification panels (Alt modifier to avoid conflicts with typing)
        (KeyCode::Char('a'), KeyModifiers::ALT) => Action::ShowAchievements,
        (KeyCode::Char('p'), KeyModifiers::ALT) => Action::ShowProgress,
        (KeyCode::Char('c'), KeyModifiers::ALT) => Action::ShowChallenges,

        _ => Action::None,
    }
}
