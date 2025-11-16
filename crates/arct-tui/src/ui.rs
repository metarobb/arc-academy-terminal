//! UI rendering and layout

use crate::app::App;
use crate::icons;
use crate::panels::{
    context::ContextPanel,
    explanation::ExplanationPanel,
    help::HelpPanel,
    lesson::LessonPanel,
    PanelId,
};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Main UI drawing function
pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.size();

    // Main layout: header + content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
        ])
        .split(size);

    // Draw header
    draw_header(frame, chunks[0], app);

    // Draw main content
    draw_content(frame, chunks[1], app);

    // Draw onboarding wizard if active (highest priority)
    if let Some(ref wizard) = app.onboarding {
        wizard.render(frame, &app.theme);
        return;  // Don't render anything else
    }

    // Draw help overlay if active
    if app.show_help {
        let help_panel = HelpPanel::new();
        help_panel.render(frame, &app.theme);
        return;  // Don't render other overlays
    }

    // Draw settings panel if active
    if let Some(ref panel) = app.settings_panel {
        panel.render(frame, &app.theme, &app.config);
    }

    // Draw lesson menu if active
    if let Some(ref mut menu) = app.lesson_menu {
        menu.render(
            frame,
            &app.theme,
            &app.completed_lessons,
            &app.user_stats,
            &app.recommendation_engine,
        );
    }

    // Draw gamification panels if active
    if let Some(ref panel) = app.achievements_panel {
        panel.render(frame, &app.theme, &app.user_stats.achievements);
    }

    if let Some(ref panel) = app.progress_panel {
        panel.render(frame, &app.theme, &app.user_stats);
    }

    if let Some(ref panel) = app.challenges_panel {
        panel.render(frame, &app.theme, &app.challenge_manager);
    }

    // Achievement notification has highest priority (render last)
    if let Some(ref notification) = app.showing_notification {
        notification.render(frame, &app.theme);
    }
}

/// Draw the header
fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = format!("  {}ARC ACADEMY TERMINAL ", icons::lightning().content);
    let mode_indicator = if app.lesson_mode {
        format!(" {}LESSON MODE ", icons::lesson().content)
    } else if app.ai_mode {
        format!(" {}AI MODE ", icons::ai().content)
    } else {
        String::new()
    };
    let version = format!("v{} | {} | arcacademy.sh ", env!("CARGO_PKG_VERSION"), app.theme.name);
    let help_text = if app.lesson_mode {
        " [? help] [^L lessons] [m menu] [Alt+A achievements] [Alt+P progress] [Alt+C challenges] [^A AI] [^T theme] [q quit] "
    } else {
        " [? help] [^L lessons] [Alt+A achievements] [Alt+P progress] [Alt+C challenges] [^A AI] [^T theme] [q quit] "
    };

    let title_len = title.len();
    let version_len = version.len();
    let mode_len = mode_indicator.len();

    let mut spans = vec![
        Span::styled(&title, app.theme.style_accent().add_modifier(Modifier::BOLD)),
    ];

    if !mode_indicator.is_empty() {
        spans.push(Span::styled(&mode_indicator, app.theme.style_success().add_modifier(Modifier::BOLD)));
    }

    spans.push(Span::styled(version, app.theme.style_info()));
    spans.push(Span::raw(" ".repeat(area.width.saturating_sub(
        title_len as u16 + mode_len as u16 + version_len as u16 + help_text.len() as u16
    ) as usize)));
    spans.push(Span::styled(help_text, app.theme.style_dim()));

    let header_text = Line::from(spans);

    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.theme.style_border_focused()),
        );

    frame.render_widget(header, area);
}

/// Draw the main content area
fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    // Layout: Left sidebar + Right main area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Context panel
            Constraint::Percentage(70), // Shell + Explanation
        ])
        .split(area);

    // Left: Context panel
    let context_panel = ContextPanel::new();

    // Get analytics summary if available
    let analytics_summary = app.analytics.as_ref()
        .and_then(|a| a.get_summary().ok());

    context_panel.render(
        frame,
        main_chunks[0],
        app.active_panel == PanelId::Context,
        &app.context,
        &app.theme,
        app.config.ai.enabled,
        analytics_summary.as_ref(),
        app.lesson_mode,
        app.virtual_fs.as_ref(),
        &app.user_stats,
        &app.challenge_manager,
    );

    // Right side: Split into shell, output, and explanation
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Shell input
            Constraint::Percentage(40), // Command output
            Constraint::Min(0),     // Explanation
        ])
        .split(main_chunks[1]);

    // Shell panel
    draw_shell_panel(
        frame,
        right_chunks[0],
        app.active_panel == PanelId::Shell,
        &app.command_buffer,
        &app.completion_suggestions,
        &app.theme,
        app.ai_mode,
        &app.ai_input_buffer,
        app.ai_loading,
    );

    // Output panel
    draw_output_panel(
        frame,
        right_chunks[1],
        app.active_panel == PanelId::Output,
        &app.last_output,
        app.output_scroll,
        &app.theme,
    );

    // Explanation panel OR Lesson panel
    if app.lesson_mode {
        // Lesson mode - show interactive lessons
        if let Some(ref lesson_panel) = app.lesson_panel {
            lesson_panel.render(
                frame,
                right_chunks[2],
                app.active_panel == PanelId::Explanation,
                &app.theme,
            );
        }
    } else {
        // Normal mode - show explanations
        let explanation_panel = ExplanationPanel::new();
        explanation_panel.render(
            frame,
            right_chunks[2],
            app.active_panel == PanelId::Explanation,
            app.last_explanation.as_ref(),
            &app.theme,
            app.ai_mode,
            app.ai_response.as_deref(),
        );
    }
}

/// Draw the shell panel
fn draw_shell_panel(
    frame: &mut Frame,
    area: Rect,
    focused: bool,
    command: &str,
    completions: &[String],
    theme: &Theme,
    ai_mode: bool,
    ai_input: &str,
    ai_loading: bool,
) {
    let border_style = if focused {
        theme.style_border_focused()
    } else {
        theme.style_border()
    };

    let title = if ai_mode {
        if focused {
            format!(" {}AI Assistant (Active - Ctrl+A to exit, Enter to ask) ", icons::ai().content)
        } else {
            format!(" {}AI Assistant ", icons::ai().content)
        }
    } else if focused {
        format!(" {}Shell (Active - Ctrl+A for AI, Tab to complete, ↑↓ for history) ", icons::shell().content)
    } else {
        format!(" {}Shell ", icons::shell().content)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // Build lines for the panel
    let mut lines = Vec::new();

    // First line: prompt and command (or AI input)
    if ai_mode {
        let prompt = icons::ai();
        let input_text = if ai_loading {
            Span::styled(format!("{}Thinking...", icons::loading().content), theme.style_dim())
        } else {
            Span::styled(ai_input, theme.style_normal())
        };

        let mut input_line = vec![prompt, input_text];

        // Add cursor if focused and not loading
        if focused && !ai_loading {
            input_line.push(Span::styled("█", theme.style_accent()));
        }

        lines.push(Line::from(input_line));
    } else {
        let prompt = Span::styled("$ ", theme.style_accent());
        let command_text = Span::styled(command, theme.style_normal());

        let mut command_line = vec![prompt, command_text];

        // Add cursor if focused
        if focused {
            command_line.push(Span::styled("█", theme.style_accent()));
        }

        lines.push(Line::from(command_line));
    }

    // Add completion suggestions if any (only in shell mode)
    if !ai_mode && !completions.is_empty() {
        lines.push(Line::from(""));  // Empty line
        lines.push(Line::from(vec![
            icons::hint(),
            Span::styled("Suggestions:", theme.style_dim()),
        ]));

        for completion in completions.iter().take(5) {
            lines.push(Line::from(vec![
                Span::styled("  • ", theme.style_dim()),
                Span::styled(completion, theme.style_success()),
            ]));
        }

        if completions.len() > 5 {
            lines.push(Line::from(vec![
                Span::styled(format!("  ...and {} more", completions.len() - 5), theme.style_dim()),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Draw the output panel
fn draw_output_panel(
    frame: &mut Frame,
    area: Rect,
    focused: bool,
    output: &str,
    scroll_offset: usize,
    theme: &Theme,
) {
    let border_style = if focused {
        theme.style_border_focused()
    } else {
        theme.style_border()
    };

    let title = if focused {
        format!(" {}Output (Active - ↑↓ to scroll) ", icons::output().content)
    } else {
        format!(" {}Output ", icons::output().content)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_height = area.height.saturating_sub(2) as usize; // Subtract borders

    let text = if output.is_empty() {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Ready to execute commands!", theme.style_dim()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Type a command above and press ", theme.style_dim()),
                Span::styled("Enter", theme.style_accent()),
            ]),
        ]
    } else {
        // Parse ANSI codes for colored output!
        let all_lines: Vec<Line> = crate::ansi::parse_ansi(output);
        let total_lines = all_lines.len();

        // Show scroll indicator if there are more lines than can fit
        let mut visible_lines: Vec<Line> = all_lines
            .into_iter()
            .skip(scroll_offset)
            .take(inner_height)
            .collect();

        // Add scroll indicator at bottom if not at end
        if scroll_offset + inner_height < total_lines {
            let remaining = total_lines - (scroll_offset + inner_height);
            visible_lines.push(Line::from(vec![
                Span::styled(
                    format!("  ↓ {} more lines (press ↓ or j to scroll)", remaining),
                    theme.style_dim(),
                ),
            ]));
        }

        // Add scroll indicator at top if not at beginning
        if scroll_offset > 0 {
            visible_lines.insert(0, Line::from(vec![
                Span::styled(
                    format!("  ↑ {} lines above (press ↑ or k to scroll)", scroll_offset),
                    theme.style_dim(),
                ),
            ]));
        }

        visible_lines
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
