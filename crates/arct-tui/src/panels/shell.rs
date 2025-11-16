//! Shell panel - interactive command input

use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Shell panel state
pub struct ShellPanel {
    /// Command buffer
    pub command: String,

    /// Cursor position
    pub cursor: usize,

    /// Command history
    pub history: Vec<String>,

    /// History position (for up/down arrows)
    pub history_pos: Option<usize>,
}

impl ShellPanel {
    pub fn new() -> Self {
        Self {
            command: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_pos: None,
        }
    }

    /// Add a character at cursor position
    pub fn insert_char(&mut self, c: char) {
        self.command.insert(self.cursor, c);
        self.cursor += 1;
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.command.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor < self.command.len() {
            self.cursor += 1;
        }
    }

    /// Clear the command buffer
    pub fn clear(&mut self) {
        self.command.clear();
        self.cursor = 0;
    }

    /// Get the current command
    pub fn get_command(&self) -> &str {
        &self.command
    }

    /// Render the shell panel
    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool, theme: &Theme) {
        let border_style = if focused {
            theme.style_border_focused()
        } else {
            theme.style_border()
        };

        let title = if focused {
            " Shell (Active) "
        } else {
            " Shell "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        // Build the prompt line
        let prompt = Span::styled("$ ", theme.style_accent());
        let command_text = Span::styled(&self.command, theme.style_normal());

        let mut line = vec![prompt, command_text];

        // Add cursor if focused
        if focused {
            line.push(Span::styled("█", theme.style_accent()));
        }

        let paragraph = Paragraph::new(Line::from(line))
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }
}

impl Default for ShellPanel {
    fn default() -> Self {
        Self::new()
    }
}
