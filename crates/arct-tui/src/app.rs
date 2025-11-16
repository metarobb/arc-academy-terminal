//! Main application state and logic

use crate::events::{Action, Event, EventHandler, key_to_action};
use crate::icons;
use crate::panels::PanelId;
use crate::shell::ShellExecutor;
use crate::theme::Theme;
use crate::ui;
use anyhow::Result;
use arct_core::{CommandAnalyzer, Context, ContextDetector, Educator, Session};
use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::collections::HashMap;
use std::io;

/// Main application state
pub struct App {
    /// Whether the application should quit
    pub should_quit: bool,

    /// Currently active panel
    pub active_panel: PanelId,

    /// User session
    pub session: Session,

    /// Current context (project detection, etc.)
    pub context: Context,

    /// Command analyzer
    pub analyzer: CommandAnalyzer,

    /// Educator for explanations
    pub educator: Educator,

    /// Current theme
    pub theme: Theme,

    /// Event handler
    event_handler: EventHandler,

    /// Show help overlay
    pub show_help: bool,

    /// Command buffer for shell input
    pub command_buffer: String,

    /// Last command explanation
    pub last_explanation: Option<arct_core::Explanation>,

    /// Shell executor
    shell_executor: ShellExecutor,

    /// Last command output
    pub last_output: String,

    /// Output panel scroll offset
    pub output_scroll: usize,

    /// Command history
    command_history: Vec<String>,

    /// Current position in history (0 = most recent, None = not browsing)
    history_position: Option<usize>,

    /// Environment variables set by export command
    pub environment_vars: HashMap<String, String>,

    /// Command aliases (name -> command)
    pub aliases: HashMap<String, String>,

    /// Application configuration
    pub config: arct_config::Config,

    /// Autocompleter
    autocompleter: crate::autocomplete::Autocompleter,

    /// Current completion suggestions (shown below shell input)
    pub completion_suggestions: Vec<String>,

    /// AI assistant provider (if enabled)
    ai_provider: Option<Box<dyn arct_ai::AIProvider>>,

    /// AI conversation history
    pub ai_conversation: Vec<arct_ai::Message>,

    /// AI input buffer (when in AI mode)
    pub ai_input_buffer: String,

    /// Last AI response
    pub ai_response: Option<String>,

    /// AI loading state
    pub ai_loading: bool,

    /// AI mode enabled (toggle between shell and AI)
    pub ai_mode: bool,

    /// Onboarding wizard (shown on first run)
    pub onboarding: Option<crate::panels::onboarding::OnboardingWizard>,

    /// Settings panel (interactive)
    pub settings_panel: Option<crate::panels::settings::SettingsPanel>,

    /// Analytics tracker
    pub analytics: Option<crate::analytics::Analytics>,

    /// Current session ID
    session_id: String,

    /// Lesson panel for interactive lessons
    pub lesson_panel: Option<crate::panels::lesson::LessonPanel>,

    /// Lesson mode enabled (toggle between explanation and lesson)
    pub lesson_mode: bool,

    /// Virtual filesystem for lesson sandboxing
    pub virtual_fs: Option<arct_core::VirtualFileSystem>,

    /// Lesson menu for selecting lessons
    pub lesson_menu: Option<crate::panels::lesson_menu::LessonMenuPanel>,

    /// Completed lesson IDs
    pub completed_lessons: std::collections::HashSet<String>,

    /// User statistics and progress tracking
    pub user_stats: arct_core::UserStats,

    /// Challenge manager for daily/weekly challenges
    pub challenge_manager: arct_core::ChallengeManager,

    /// Recommendation engine for suggesting lessons
    pub recommendation_engine: arct_core::RecommendationEngine,

    /// Achievements panel
    pub achievements_panel: Option<crate::panels::achievements::AchievementsPanel>,

    /// Progress panel
    pub progress_panel: Option<crate::panels::progress::ProgressPanel>,

    /// Challenges panel
    pub challenges_panel: Option<crate::panels::challenges::ChallengesPanel>,

    /// Pending achievements to show notifications for
    pub pending_achievements: Vec<arct_core::Achievement>,

    /// Currently showing achievement notification
    pub showing_notification: Option<crate::panels::notification::NotificationPanel>,
}

impl App {
    /// Create a new application
    pub fn new() -> Result<Self> {
        let session = Session::new();
        let working_dir = session.state.working_directory.clone();
        let context = ContextDetector::detect(&working_dir)?;

        // Load configuration
        let config = arct_config::Config::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load config, using defaults: {}", e);
            arct_config::Config::default()
        });

        // Load session data (history, stats, progress) from disk
        let session_data = match crate::persistence::load_session() {
            Ok(data) => {
                tracing::info!("Loaded session with {} commands, {} completed lessons",
                    data.command_history.len(),
                    data.completed_lessons.len()
                );
                data
            }
            Err(e) => {
                tracing::warn!("Failed to load session: {}", e);
                crate::persistence::SessionData::new()
            }
        };

        let command_history = session_data.command_history;

        // Load aliases and environment variables from config
        let aliases = config.shell.aliases.clone();
        let environment_vars = config.shell.environment.clone();

        // Select theme based on config
        let theme = Theme::from_name(&config.theme.default_theme);

        // Initialize AI provider if enabled
        let ai_provider = if config.ai.enabled {
            match Self::create_ai_provider(&config.ai) {
                Ok(provider) => {
                    tracing::info!("AI provider initialized: {}", provider.name());
                    Some(provider)
                }
                Err(e) => {
                    tracing::warn!("Failed to initialize AI provider: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Check if first run (show onboarding)
        let onboarding = if !config.general.setup_complete {
            Some(crate::panels::onboarding::OnboardingWizard::new())
        } else {
            None
        };

        // Create welcome message for returning users
        let welcome_message = if config.general.setup_complete {
            let name = config.general.user_name.as_deref().unwrap_or("there");
            let mut msg = format!("{}Welcome back, {}!\n\n", icons::welcome().content, name);

            // Add quick tips
            msg.push_str("Quick reminders:\n");
            if config.ai.enabled {
                msg.push_str("  • Press Ctrl+A to ask the AI for help\n");
            }
            msg.push_str("  • Press ? for help\n");
            msg.push_str("  • Press Ctrl+S for settings\n");
            msg.push_str("  • Tab to autocomplete commands\n\n");
            msg.push_str("Start typing a command to begin!\n");
            msg
        } else {
            String::new()
        };

        // Load user stats from session and update streak
        let mut user_stats = session_data.user_stats;
        user_stats.update_streak(); // Update streak on app start

        // Load challenge manager from session
        let mut challenge_manager = session_data.challenge_manager;
        // Generate today's challenges (will use cached if same day/week)
        challenge_manager.get_daily_challenge();
        challenge_manager.get_weekly_challenge();

        // Load completed lessons from session
        let completed_lessons = session_data.completed_lessons;

        Ok(Self {
            should_quit: false,
            active_panel: PanelId::Shell,
            session,
            context,
            analyzer: CommandAnalyzer::new(),
            educator: Educator::new(),
            theme,
            event_handler: EventHandler::new(),
            show_help: false,
            command_buffer: String::new(),
            last_explanation: None,
            shell_executor: ShellExecutor::new()?,
            last_output: welcome_message,
            output_scroll: 0,
            command_history,
            history_position: None,
            environment_vars,
            aliases,
            config,
            autocompleter: crate::autocomplete::Autocompleter::new(),
            completion_suggestions: Vec::new(),
            ai_provider,
            ai_conversation: Vec::new(),
            ai_input_buffer: String::new(),
            ai_response: None,
            ai_loading: false,
            ai_mode: false,
            onboarding,
            settings_panel: None,
            analytics: crate::analytics::Analytics::new().ok(),
            session_id: uuid::Uuid::new_v4().to_string(),
            lesson_panel: Self::initialize_lesson_panel(),
            lesson_mode: false,
            virtual_fs: None,
            lesson_menu: None,
            completed_lessons,
            user_stats,
            challenge_manager,
            recommendation_engine: arct_core::RecommendationEngine::new(),
            achievements_panel: None,
            progress_panel: None,
            challenges_panel: None,
            pending_achievements: Vec::new(),
            showing_notification: None,
        })
    }

    /// Initialize empty lesson panel (user will select lesson from menu)
    fn initialize_lesson_panel() -> Option<crate::panels::lesson::LessonPanel> {
        Some(crate::panels::lesson::LessonPanel::new())
    }

    /// Create AI provider from configuration
    fn create_ai_provider(config: &arct_config::AIConfig) -> Result<Box<dyn arct_ai::AIProvider>> {
        let ai_config = match config.provider.as_str() {
            "anthropic" => {
                let api_key = config.api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("Anthropic API key not set"))?;
                let model = config.model.clone()
                    .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());
                arct_ai::AIConfig::Anthropic { api_key, model }
            }
            "openai" => {
                let api_key = config.api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("OpenAI API key not set"))?;
                let model = config.model.clone()
                    .unwrap_or_else(|| "gpt-4-turbo-preview".to_string());
                arct_ai::AIConfig::OpenAI { api_key, model }
            }
            "local" => {
                let endpoint = config.endpoint.clone()
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                let model = config.model.clone();
                arct_ai::AIConfig::Local { endpoint, model }
            }
            "managed" => {
                let auth_token = config.api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("Managed API token not set"))?;
                arct_ai::AIConfig::Managed { auth_token }
            }
            "claude-cli" => {
                // Claude Code CLI - no API key needed
                let model = config.model.clone();
                arct_ai::AIConfig::ClaudeCLI { model }
            }
            _ => arct_ai::AIConfig::Disabled,
        };

        arct_ai::AIFactory::create(&ai_config)
            .map_err(|e| anyhow::anyhow!("Failed to create AI provider: {}", e))
    }

    /// Show ASCII art splash screen
    fn show_splash_screen() -> Result<()> {
        use crossterm::{
            cursor,
            style::{Color, Print, SetForegroundColor, ResetColor},
            terminal::{Clear, ClearType},
        };
        use std::io::Write;

        let mut stdout = io::stdout();

        // Clear screen
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        // ASCII art logo
        let logo = r#"
 ▄▄▄       ██▀███   ▄████▄      ▄▄▄       ▄████▄   ▄▄▄      ▓█████▄ ▓█████  ███▄ ▄███▓▓██   ██▓
▒████▄    ▓██ ▒ ██▒▒██▀ ▀█     ▒████▄    ▒██▀ ▀█  ▒████▄    ▒██▀ ██▌▓█   ▀ ▓██▒▀█▀ ██▒ ▒██  ██▒
▒██  ▀█▄  ▓██ ░▄█ ▒▒▓█    ▄    ▒██  ▀█▄  ▒▓█    ▄ ▒██  ▀█▄  ░██   █▌▒███   ▓██    ▓██░  ▒██ ██░
░██▄▄▄▄██ ▒██▀▀█▄  ▒▓▓▄ ▄██▒   ░██▄▄▄▄██ ▒▓▓▄ ▄██▒░██▄▄▄▄██ ░▓█▄   ▌▒▓█  ▄ ▒██    ▒██   ░ ▐██▓░
 ▓█   ▓██▒░██▓ ▒██▒▒ ▓███▀ ░    ▓█   ▓██▒▒ ▓███▀ ░ ▓█   ▓██▒░▒████▓ ░▒████▒▒██▒   ░██▒  ░ ██▒▓░
 ▒▒   ▓▒█░░ ▒▓ ░▒▓░░ ░▒ ▒  ░    ▒▒   ▓▒█░░ ░▒ ▒  ░ ▒▒   ▓▒█░ ▒▒▓  ▒ ░░ ▒░ ░░ ▒░   ░  ░   ██▒▒▒
  ▒   ▒▒ ░  ░▒ ░ ▒░  ░  ▒        ▒   ▒▒ ░  ░  ▒     ▒   ▒▒ ░ ░ ▒  ▒  ░ ░  ░░  ░      ░ ▓██ ░▒░
  ░   ▒     ░░   ░ ░             ░   ▒   ░          ░   ▒    ░ ░  ░    ░   ░      ░    ▒ ▒ ░░
      ░  ░   ░     ░ ░               ░  ░░ ░            ░  ░   ░       ░  ░       ░    ░ ░
                   ░                     ░                   ░                         ░ ░
"#;

        let tagline = "Λ° Learn Shell Commands Interactively with AI";
        let version = format!("v{}", env!("CARGO_PKG_VERSION"));

        // Get terminal size for centering
        let (width, height) = crossterm::terminal::size()?;
        let start_row = (height / 2).saturating_sub(7); // Center vertically

        // Calculate logo width (longest line is ~92 chars)
        let logo_width = 92;
        let logo_col = (width / 2).saturating_sub(logo_width / 2);

        // Print logo with orange color, centered
        for (i, line) in logo.lines().enumerate() {
            let row = start_row + i as u16;
            execute!(
                stdout,
                cursor::MoveTo(logo_col, row),
                SetForegroundColor(Color::Rgb { r: 255, g: 140, b: 0 }), // Arc Academy Orange
                Print(line),
                ResetColor
            )?;
        }

        // Print tagline centered below logo
        let tagline_row = start_row + 11;
        let tagline_col = (width / 2).saturating_sub((tagline.len() / 2) as u16);
        execute!(
            stdout,
            cursor::MoveTo(tagline_col, tagline_row),
            SetForegroundColor(Color::White),
            Print(tagline),
            ResetColor
        )?;

        // Print version
        let version_row = tagline_row + 1;
        let version_col = (width / 2).saturating_sub((version.len() / 2) as u16);
        execute!(
            stdout,
            cursor::MoveTo(version_col, version_row),
            SetForegroundColor(Color::DarkGrey),
            Print(version),
            ResetColor
        )?;

        stdout.flush()?;

        // Pause for 1.5 seconds
        std::thread::sleep(std::time::Duration::from_millis(1500));

        // Clear screen before entering TUI
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        Ok(())
    }

    /// Run the application
    pub async fn run(&mut self) -> Result<()> {
        // Show splash screen
        Self::show_splash_screen()?;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Start event handler
        self.event_handler.start().await;

        // Main loop
        let result = self.main_loop(&mut terminal).await;

        // Save all progress before exiting
        self.save_session();

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    /// Main application loop
    async fn main_loop<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            // Draw UI
            terminal.draw(|f| ui::draw(f, self))?;

            // Handle events
            if let Some(event) = self.event_handler.next().await {
                self.handle_event(event).await?;
            }

            // Check for quit
            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle an event
    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) => {
                // If onboarding is active, handle onboarding events
                if self.onboarding.is_some() {
                    return self.handle_onboarding_event(key).await;
                }

                // If settings panel is open, handle settings events
                if self.settings_panel.is_some() {
                    return self.handle_settings_event(key).await;
                }

                // If lesson menu is open, handle menu events
                if let Some(ref mut menu) = self.lesson_menu {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            menu.select_previous();
                            return Ok(());
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            menu.select_next();
                            return Ok(());
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            if let Some(digit) = c.to_digit(10) {
                                // Map 1-9 to lessons 1-9, and 0 to lesson 10
                                let lesson_num = if digit == 0 { 10 } else { digit as usize };
                                menu.select_by_number(lesson_num);
                            }
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            // Load selected lesson
                            if let Some(lesson) = menu.get_selected_lesson() {
                                if let Some(ref mut panel) = self.lesson_panel {
                                    panel.load_lesson(lesson);
                                    self.lesson_menu = None;
                                    self.last_output = format!("{}Lesson loaded! Follow the instructions in the lesson panel.\n", icons::lesson().content);
                                }
                            }
                            return Ok(());
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.lesson_menu = None;
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // If in AI mode and in Shell panel, handle AI input
                if self.ai_mode && self.active_panel == PanelId::Shell && !self.show_help {
                    match key.code {
                        KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT => {
                            self.ai_input_buffer.push(c);
                            return Ok(());
                        }
                        KeyCode::Backspace => {
                            self.ai_input_buffer.pop();
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            if !self.ai_input_buffer.is_empty() {
                                let question = self.ai_input_buffer.clone();
                                self.ai_input_buffer.clear();
                                self.ask_ai(question).await?;
                            }
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // If in shell panel and not a special action, handle as text input or history
                if self.active_panel == PanelId::Shell && !self.show_help && !self.ai_mode {
                    match key.code {
                        KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT => {
                            self.command_buffer.push(c);
                            // Reset history position when typing
                            self.history_position = None;
                            // Clear completion suggestions when typing
                            self.completion_suggestions.clear();
                            return Ok(());
                        }
                        KeyCode::Backspace => {
                            self.command_buffer.pop();
                            // Reset history position when editing
                            self.history_position = None;
                            // Clear completion suggestions when editing
                            self.completion_suggestions.clear();
                            return Ok(());
                        }
                        KeyCode::Tab if key.modifiers == KeyModifiers::NONE => {
                            // Tab in shell panel triggers autocomplete
                            self.handle_autocomplete()?;
                            return Ok(());
                        }
                        KeyCode::Up => {
                            // Navigate backward in history (older commands)
                            self.history_previous();
                            return Ok(());
                        }
                        KeyCode::Down => {
                            // Navigate forward in history (newer commands)
                            self.history_next();
                            return Ok(());
                        }
                        _ => {}
                    }
                }

                // Handle as action
                let action = key_to_action(key);
                self.handle_action(action).await?;
            }
            Event::Resize(_, _) => {
                // Terminal will automatically redraw on next iteration
            }
            Event::Tick => {
                // Update any time-based state here
            }
            Event::Quit => {
                self.should_quit = true;
            }
        }

        Ok(())
    }

    /// Handle an action
    async fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                if self.show_help {
                    self.show_help = false;
                } else {
                    self.should_quit = true;
                }
            }
            Action::NextPanel => {
                self.active_panel = self.active_panel.next();
                // Reset scroll when switching panels
                self.output_scroll = 0;
            }
            Action::PreviousPanel => {
                self.active_panel = self.active_panel.previous();
                // Reset scroll when switching panels
                self.output_scroll = 0;
            }
            Action::ScrollUp => {
                // Only scroll when Output panel is focused
                if self.active_panel == PanelId::Output {
                    self.output_scroll = self.output_scroll.saturating_sub(1);
                }
            }
            Action::ScrollDown => {
                // Only scroll when Output panel is focused
                if self.active_panel == PanelId::Output {
                    let total_lines = self.last_output.lines().count();
                    if self.output_scroll < total_lines.saturating_sub(1) {
                        self.output_scroll += 1;
                    }
                }
            }
            Action::PageUp => {
                // Only scroll when Output panel is focused
                if self.active_panel == PanelId::Output {
                    self.output_scroll = self.output_scroll.saturating_sub(10);
                }
            }
            Action::PageDown => {
                // Only scroll when Output panel is focused
                if self.active_panel == PanelId::Output {
                    let total_lines = self.last_output.lines().count();
                    self.output_scroll = (self.output_scroll + 10).min(total_lines.saturating_sub(1));
                }
            }
            Action::Help => {
                self.show_help = !self.show_help;
            }
            Action::ToggleTheme => {
                self.theme = self.theme.cycle_next();
            }
            Action::ToggleAI => {
                self.toggle_ai_mode();
            }
            Action::ToggleSettings => {
                if self.settings_panel.is_some() {
                    self.settings_panel = None;
                } else {
                    self.settings_panel = Some(crate::panels::settings::SettingsPanel::new());
                }
            }
            Action::ToggleLesson => {
                self.toggle_lesson_mode();
            }
            Action::ShowLessonMenu => {
                if self.lesson_mode {
                    // Toggle menu on/off
                    if self.lesson_menu.is_some() {
                        self.lesson_menu = None;
                    } else {
                        self.lesson_menu = Some(crate::panels::lesson_menu::LessonMenuPanel::new());
                    }
                }
            }
            Action::Escape => {
                if self.showing_notification.is_some() {
                    self.dismiss_notification();
                } else if self.show_help {
                    self.show_help = false;
                } else if self.achievements_panel.is_some() {
                    self.achievements_panel = None;
                } else if self.progress_panel.is_some() {
                    self.progress_panel = None;
                } else if self.challenges_panel.is_some() {
                    self.challenges_panel = None;
                } else if self.settings_panel.is_some() {
                    self.settings_panel = None;
                } else if self.lesson_menu.is_some() {
                    self.lesson_menu = None;
                } else if self.ai_mode {
                    self.ai_mode = false;
                }
            }
            Action::Enter => {
                if self.showing_notification.is_some() {
                    self.dismiss_notification();
                } else if !self.ai_mode {
                    self.execute_command().await?;
                }
            }
            Action::ShowAchievements => {
                self.toggle_achievements_panel();
            }
            Action::ShowProgress => {
                self.toggle_progress_panel();
            }
            Action::ShowChallenges => {
                self.toggle_challenges_panel();
            }
            Action::DismissNotification => {
                self.dismiss_notification();
            }
            _ => {}
        }

        Ok(())
    }

    /// Helper: Check if a flag exists in the command
    fn has_flag(cmd: &arct_core::Command, flag: &str) -> bool {
        cmd.flags.iter().any(|f| {
            f.raw == flag ||
            f.short == Some(flag.chars().nth(1).unwrap_or(' ')) ||
            f.long.as_ref().map(|l| l == flag.trim_start_matches("--")).unwrap_or(false)
        })
    }

    /// Execute command against virtual filesystem and return output
    fn execute_virtual_fs_command(&mut self, cmd: &arct_core::Command) -> Option<String> {
        if !self.lesson_mode {
            return None;
        }

        let vfs = self.virtual_fs.as_mut()?;
        let program = cmd.program.as_str();

        match program {
            "pwd" => {
                let current = vfs.get_current_dir().display().to_string();
                Some(format!("{}\n", current))
            }

            "ls" => {
                // Check for -a flag (show hidden files)
                let show_hidden = Self::has_flag(cmd, "-a") || Self::has_flag(cmd, "--all");

                match vfs.list_directory(None) {
                    Ok(mut entries) => {
                        // Filter out hidden files if -a not specified
                        if !show_hidden {
                            entries.retain(|e| !e.name.starts_with('.'));
                        }

                        let mut output = String::new();
                        for entry in entries {
                            if entry.is_dir {
                                output.push_str(&format!("{}{}/\n", icons::folder().content, entry.name));
                            } else {
                                output.push_str(&format!("{}{}\n", icons::file().content, entry.name));
                            }
                        }

                        if output.is_empty() {
                            output = "Empty directory\n".to_string();
                        }

                        Some(output)
                    }
                    Err(e) => Some(format!("{}ls: {}\n", icons::error().content, e)),
                }
            }

            "cd" => {
                let target = cmd.args.first().map(|s| s.as_str()).unwrap_or("~");
                match vfs.change_directory(target) {
                    Ok(new_path) => {
                        Some(format!("{}Changed to: {}\n", icons::folder().content, new_path))
                    }
                    Err(e) => Some(format!("{}cd: {}\n", icons::error().content, e)),
                }
            }

            "cat" => {
                if cmd.args.is_empty() {
                    return Some(format!("{}cat: missing file argument\n", icons::error().content));
                }

                let mut output = String::new();
                for file in &cmd.args {
                    match vfs.read_file(file) {
                        Ok(content) => output.push_str(&content),
                        Err(e) => output.push_str(&format!("{}cat: {}\n", icons::error().content, e)),
                    }
                }
                Some(output)
            }

            "mkdir" => {
                if cmd.args.is_empty() {
                    return Some(format!("{}mkdir: missing directory name\n", icons::error().content));
                }

                let parents = Self::has_flag(cmd, "-p") || Self::has_flag(cmd, "--parents");
                let mut output = String::new();

                for dir in &cmd.args {
                    match vfs.create_directory(dir, parents) {
                        Ok(_) => output.push_str(&format!("{}Created directory: {}\n", icons::folder().content, dir)),
                        Err(e) => output.push_str(&format!("{}mkdir: {}\n", icons::error().content, e)),
                    }
                }
                Some(output)
            }

            "touch" => {
                if cmd.args.is_empty() {
                    return Some(format!("{}touch: missing file argument\n", icons::error().content));
                }

                let mut output = String::new();
                for file in &cmd.args {
                    match vfs.touch_file(file) {
                        Ok(_) => output.push_str(&format!("{}Created/updated file: {}\n", icons::file().content, file)),
                        Err(e) => output.push_str(&format!("{}touch: {}\n", icons::error().content, e)),
                    }
                }
                Some(output)
            }

            "rm" => {
                if cmd.args.is_empty() {
                    return Some(format!("{}rm: missing file argument\n", icons::error().content));
                }

                let recursive = Self::has_flag(cmd, "-r") || Self::has_flag(cmd, "-R") || Self::has_flag(cmd, "--recursive");
                let force = Self::has_flag(cmd, "-f") || Self::has_flag(cmd, "--force");
                let mut output = String::new();

                for item in &cmd.args {
                    match vfs.remove(item, recursive, force) {
                        Ok(_) => output.push_str(&format!("{}Removed: {}\n", icons::success().content, item)),
                        Err(e) => output.push_str(&format!("{}rm: {}\n", icons::error().content, e)),
                    }
                }
                Some(output)
            }

            "mv" => {
                if cmd.args.len() < 2 {
                    return Some(format!("{}mv: missing source or destination\n", icons::error().content));
                }

                let source = &cmd.args[0];
                let dest = &cmd.args[1];

                match vfs.move_item(source, dest) {
                    Ok(_) => Some(format!("{}Moved {} to {}\n", icons::success().content, source, dest)),
                    Err(e) => Some(format!("{}mv: {}\n", icons::error().content, e)),
                }
            }

            "cp" => {
                if cmd.args.len() < 2 {
                    return Some(format!("{}cp: missing source or destination\n", icons::error().content));
                }

                let recursive = Self::has_flag(cmd, "-r") || Self::has_flag(cmd, "-R") || Self::has_flag(cmd, "--recursive");
                let source = &cmd.args[0];
                let dest = &cmd.args[1];

                match vfs.copy(source, dest, recursive) {
                    Ok(_) => Some(format!("{}Copied {} to {}\n", icons::success().content, source, dest)),
                    Err(e) => Some(format!("{}cp: {}\n", icons::error().content, e)),
                }
            }

            // Commands that don't manipulate the filesystem
            _ => None,
        }
    }

    /// Execute the current command
    async fn execute_command(&mut self) -> Result<()> {
        // In lesson mode, allow empty commands (for Information steps that just need Enter)
        if self.command_buffer.is_empty() && !self.lesson_mode {
            return Ok(());
        }

        let mut command_str = self.command_buffer.clone();

        // If in lesson mode with empty command, skip parsing and go to validation
        if self.lesson_mode && command_str.is_empty() {
            // Extract lesson completion info outside the borrow scope
            let mut lesson_completed_info: Option<(String, arct_core::Difficulty)> = None;

            if let Some(ref mut lesson_panel) = self.lesson_panel {
                let validation = lesson_panel.validate_current_step(&command_str);

                if validation.is_success() {
                    // Success! Move to next step
                    self.last_output = format!("{}{}\n\nMoving to next step...\n",
                        icons::success().content,
                        match &validation {
                            arct_core::ValidationResult::Success { message } => message,
                            _ => "Success!",
                        }
                    );

                    if !lesson_panel.next_step() {
                        // Lesson complete! Extract info for later processing
                        if let Some(lesson) = lesson_panel.current_lesson.as_ref() {
                            lesson_completed_info = Some((lesson.id.clone(), lesson.difficulty));
                        }
                        self.last_output.push_str(&format!("\n{}Congratulations! You've completed this lesson!\n\nPress Ctrl+L to exit lesson mode or 'm' to select another lesson.\n", icons::celebration().content));
                    }
                } else {
                    // Information steps should always succeed with Enter
                    self.last_output = "Press Enter to continue...\n".to_string();
                }

                self.command_buffer.clear();
                self.add_to_history(command_str.clone());
            }

            // Process lesson completion outside the borrow scope
            if let Some((lesson_id, difficulty)) = lesson_completed_info {
                self.record_lesson_completion(lesson_id, difficulty);
            }

            return Ok(());
        }

        // Parse command
        let cmd = self.analyzer.parse(&command_str)?;

        // If in lesson mode, execute against virtual FS and validate
        if self.lesson_mode {
            // Execute command against virtual filesystem and get output
            let vfs_output = self.execute_virtual_fs_command(&cmd);

            // Extract lesson completion info outside the borrow scope
            let mut lesson_completed_info: Option<(String, arct_core::Difficulty)> = None;

            if let Some(ref mut lesson_panel) = self.lesson_panel {
                let validation = lesson_panel.validate_current_step(&command_str);

                // Build output: virtual FS output FIRST, then validation feedback
                let mut output = String::new();

                // Show the command output from virtual filesystem
                if let Some(vfs_out) = vfs_output {
                    output.push_str(&vfs_out);
                    output.push('\n');
                }

                // Then show validation feedback
                if validation.is_success() {
                    // Success! Move to next step
                    output.push_str(&format!("{}{}\n\n",
                        icons::success().content,
                        match &validation {
                            arct_core::ValidationResult::Success { message } => message,
                            _ => "Correct!",
                        }
                    ));

                    if !lesson_panel.next_step() {
                        // Lesson complete! Extract info for later processing
                        if let Some(lesson) = lesson_panel.current_lesson.as_ref() {
                            lesson_completed_info = Some((lesson.id.clone(), lesson.difficulty));
                        }
                        output.push_str(&format!("{}Congratulations! You've completed this lesson!\n\nPress Ctrl+L to exit lesson mode or 'm' to select another lesson.\n", icons::celebration().content));
                    } else {
                        output.push_str("Moving to next step...\n");
                    }
                } else {
                    // Show validation failure
                    output.push_str(&match validation {
                        arct_core::ValidationResult::Failure { message, hint } => {
                            let mut fail_output = format!("{}{}\n", icons::error().content, message);
                            if let Some(h) = hint {
                                fail_output.push_str(&format!("\n{}Hint: {}\n", icons::hint().content, h));
                            }
                            fail_output.push_str("\nTry again!\n");
                            fail_output
                        }
                        arct_core::ValidationResult::Partial { message, progress } => {
                            format!("{}{} ({:.0}% correct)\n\nKeep trying!\n", icons::warning().content, message, progress)
                        }
                        _ => "Try again!\n".to_string(),
                    });
                }

                self.last_output = output;
                self.command_buffer.clear();
                self.add_to_history(command_str.clone());
            }

            // Process lesson completion outside the borrow scope
            if let Some((lesson_id, difficulty)) = lesson_completed_info {
                self.record_lesson_completion(lesson_id, difficulty);
            }

            return Ok(());
        }

        // Generate explanation
        let explanation = self.educator.explain(&cmd)?;
        self.last_explanation = Some(explanation);

        // Check if this is a shell builtin command
        match cmd.program.as_str() {
            "cd" => {
                // Add to history before clearing buffer
                self.add_to_history(command_str.clone());
                self.handle_cd_command(&cmd)?;
                self.command_buffer.clear();
                return Ok(());
            }
            "history" => {
                // Add to history before clearing buffer
                self.add_to_history(command_str.clone());
                self.handle_history_command(&cmd)?;
                self.command_buffer.clear();
                return Ok(());
            }
            "export" => {
                // Add to history before clearing buffer
                self.add_to_history(command_str.clone());
                self.handle_export_command(&cmd)?;
                self.command_buffer.clear();
                return Ok(());
            }
            "alias" => {
                // Add to history before clearing buffer
                self.add_to_history(command_str.clone());
                self.handle_alias_command(&cmd)?;
                self.command_buffer.clear();
                return Ok(());
            }
            _ => {
                // Check if the command is an alias and expand it
                if let Some(aliased_command) = self.aliases.get(cmd.program.as_str()) {
                    // Replace the alias with the full command
                    let args_str = if !cmd.args.is_empty() {
                        format!(" {}", cmd.args.join(" "))
                    } else {
                        String::new()
                    };
                    command_str = format!("{}{}", aliased_command, args_str);
                }
            }
        }

        // Show executing status
        self.last_output = format!("{}Executing: {}\n", icons::loading().content, command_str);

        // Execute the command for real with timeout
        let start_time = std::time::Instant::now();

        // Use tokio timeout to prevent hanging
        let timeout_duration = std::time::Duration::from_secs(5);
        let env_vars = self.environment_vars.clone();
        let output_result = tokio::time::timeout(
            timeout_duration,
            self.shell_executor.execute(command_str.clone(), env_vars)
        ).await;

        let output = match output_result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => format!("{}Error: {}", icons::error().content, e),
            Err(_) => format!("Command timed out after {} seconds", timeout_duration.as_secs()),
        };

        let duration = start_time.elapsed();

        // Store output
        self.last_output = output.clone();

        // Determine if command was successful
        let success = !output.starts_with(icons::error().content.as_ref()) && !output.contains("timed out");

        // Reset scroll to top for new output
        self.output_scroll = 0;

        // Record in session
        self.session.record_command(
            command_str.clone(),
            Some(0),
            Some(duration.as_millis() as u64),
        );

        // Track in analytics database
        if let Some(ref analytics) = self.analytics {
            let working_dir = self.session.state.working_directory.to_string_lossy().to_string();
            let _ = analytics.record_command(
                &command_str,
                success,
                &working_dir,
                &self.session_id,
            );
        }

        // Add to history
        self.add_to_history(command_str.clone());

        // Track command use for stats and achievements
        self.record_command_for_stats(&command_str);
        self.check_and_unlock_achievements();

        // Clear buffer
        self.command_buffer.clear();

        Ok(())
    }

    /// Handle cd command specially (it's a shell builtin)
    fn handle_cd_command(&mut self, cmd: &arct_core::Command) -> Result<()> {
        use std::path::PathBuf;

        // If in lesson mode, use virtual filesystem
        if self.lesson_mode {
            if let Some(ref mut vfs) = self.virtual_fs {
                let target_str = if cmd.args.is_empty() {
                    "~"
                } else {
                    &cmd.args[0]
                };

                match vfs.change_directory(target_str) {
                    Ok(new_path) => {
                        self.last_output = format!(
                            "{}Changed directory to:\n  {}\n\n{}You're in the virtual lesson filesystem\n",
                            icons::success().content, new_path, icons::hint().content
                        );
                        self.output_scroll = 0;
                        return Ok(());
                    }
                    Err(e) => {
                        self.last_output = format!("{}cd: {}\n", icons::error().content, e);
                        self.output_scroll = 0;
                        return Ok(());
                    }
                }
            }
        }

        // Normal mode - use real filesystem
        // Determine target directory
        let target = if cmd.args.is_empty() {
            // cd with no args goes to home directory
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        } else {
            let target_str = &cmd.args[0];

            // Expand ~ to home directory
            if target_str == "~" {
                dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            } else if target_str.starts_with("~/") {
                let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
                home.join(&target_str[2..])
            } else {
                PathBuf::from(target_str)
            }
        };

        // Try to change directory
        match std::env::set_current_dir(&target) {
            Ok(_) => {
                // Update session working directory
                self.session.state.working_directory = std::env::current_dir()?;

                // Update context
                self.update_context()?;

                // Show success message
                let new_dir = std::env::current_dir()?;
                self.last_output = format!(
                    "{}Changed directory to:\n  {}\n",
                    icons::success().content, new_dir.display()
                );

                // Reset scroll
                self.output_scroll = 0;

                // Record in session
                self.session.record_command(
                    format!("cd {}", cmd.args.join(" ")),
                    Some(0),
                    Some(0),
                );

                Ok(())
            }
            Err(e) => {
                // Show error message
                self.last_output = format!(
                    "{}cd: {}\n  Cannot change to: {}\n",
                    icons::error().content, e, target.display()
                );
                self.output_scroll = 0;

                // Record in session as failed
                self.session.record_command(
                    format!("cd {}", cmd.args.join(" ")),
                    Some(1),
                    Some(0),
                );

                Ok(())
            }
        }
    }

    /// Update context (e.g., when directory changes)
    pub fn update_context(&mut self) -> Result<()> {
        let working_dir = &self.session.state.working_directory;
        self.context = ContextDetector::detect(working_dir)?;
        Ok(())
    }

    /// Navigate to previous command in history (Up arrow)
    fn history_previous(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        match self.history_position {
            None => {
                // Start browsing history from most recent
                self.history_position = Some(0);
                self.command_buffer = self.command_history[0].clone();
            }
            Some(pos) => {
                // Move to older command if possible
                if pos < self.command_history.len() - 1 {
                    let new_pos = pos + 1;
                    self.history_position = Some(new_pos);
                    self.command_buffer = self.command_history[new_pos].clone();
                }
            }
        }
    }

    /// Navigate to next command in history (Down arrow)
    fn history_next(&mut self) {
        match self.history_position {
            None => {
                // Not browsing history, do nothing
            }
            Some(0) => {
                // At most recent, clear buffer
                self.history_position = None;
                self.command_buffer.clear();
            }
            Some(pos) => {
                // Move to newer command
                let new_pos = pos - 1;
                self.history_position = Some(new_pos);
                self.command_buffer = self.command_history[new_pos].clone();
            }
        }
    }

    /// Add a command to history
    fn add_to_history(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }

        // Don't add duplicate of most recent command
        if let Some(last) = self.command_history.first() {
            if last == &command {
                return;
            }
        }

        // Add to beginning (most recent first)
        self.command_history.insert(0, command);

        // Limit history size to 1000 commands
        if self.command_history.len() > 1000 {
            self.command_history.truncate(1000);
        }

        // Save all progress to disk
        self.save_session();
    }

    /// Save all session data (history, stats, progress) to disk
    fn save_session(&self) {
        let session_data = crate::persistence::SessionData {
            command_history: self.command_history.clone(),
            last_updated: chrono::Local::now().to_rfc3339(),
            user_stats: self.user_stats.clone(),
            completed_lessons: self.completed_lessons.clone(),
            challenge_manager: self.challenge_manager.clone(),
        };

        if let Err(e) = crate::persistence::save_session(&session_data) {
            tracing::warn!("Failed to save session: {}", e);
        }
    }

    /// Handle history command (show command history)
    fn handle_history_command(&mut self, cmd: &arct_core::Command) -> Result<()> {
        // Parse optional argument for number of commands to show
        let limit = if cmd.args.is_empty() {
            50 // Default: show last 50 commands
        } else {
            cmd.args[0].parse::<usize>().unwrap_or(50)
        };

        if self.command_history.is_empty() {
            self.last_output = "No commands in history yet.\n".to_string();
        } else {
            let mut output = String::new();
            let total = self.command_history.len();

            // Show commands in reverse chronological order (oldest to newest on screen)
            // but numbered from oldest to newest (like bash)
            for (i, cmd) in self.command_history.iter().rev().enumerate().take(limit) {
                let index = total - self.command_history.len() + i + 1;
                output.push_str(&format!("{:5}  {}\n", index, cmd));
            }

            self.last_output = output;
        }

        // Reset scroll
        self.output_scroll = 0;

        // Record in session
        self.session.record_command(
            format!("history {}", if cmd.args.is_empty() { String::new() } else { cmd.args.join(" ") }),
            Some(0),
            Some(0),
        );

        Ok(())
    }

    /// Handle export command (set environment variables)
    fn handle_export_command(&mut self, cmd: &arct_core::Command) -> Result<()> {
        if cmd.args.is_empty() {
            // No arguments - show all exported variables
            if self.environment_vars.is_empty() {
                self.last_output = "No environment variables set.\n".to_string();
            } else {
                let mut output = String::new();
                output.push_str("Exported environment variables:\n\n");
                let mut vars: Vec<_> = self.environment_vars.iter().collect();
                vars.sort_by_key(|(k, _)| *k);
                for (key, value) in vars {
                    output.push_str(&format!("  {}={}\n", key, value));
                }
                self.last_output = output;
            }
        } else {
            // Parse VAR=value format
            for arg in &cmd.args {
                if let Some((key, value)) = arg.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim().to_string();

                    // Remove quotes if present
                    let value = if (value.starts_with('"') && value.ends_with('"')) ||
                                   (value.starts_with('\'') && value.ends_with('\'')) {
                        value[1..value.len()-1].to_string()
                    } else {
                        value
                    };

                    self.environment_vars.insert(key.clone(), value.clone());
                    self.last_output = format!("{}Exported: {}={}\n", icons::success().content, key, value);

                    // Save to config
                    self.config.shell.environment = self.environment_vars.clone();
                    if let Err(e) = self.config.save() {
                        tracing::warn!("Failed to save config: {}", e);
                    }
                } else {
                    self.last_output = format!("{}Invalid export syntax: {}\n  Usage: export VAR=value\n", icons::error().content, arg);
                    break;
                }
            }
        }

        // Reset scroll
        self.output_scroll = 0;

        // Record in session
        self.session.record_command(
            format!("export {}", cmd.args.join(" ")),
            Some(0),
            Some(0),
        );

        Ok(())
    }

    /// Handle alias command (create command shortcuts)
    fn handle_alias_command(&mut self, cmd: &arct_core::Command) -> Result<()> {
        if cmd.args.is_empty() {
            // No arguments - show all aliases
            if self.aliases.is_empty() {
                self.last_output = "No aliases defined.\n".to_string();
            } else {
                let mut output = String::new();
                output.push_str("Defined aliases:\n\n");
                let mut aliases: Vec<_> = self.aliases.iter().collect();
                aliases.sort_by_key(|(k, _)| *k);
                for (name, command) in aliases {
                    output.push_str(&format!("  {}='{}'\n", name, command));
                }
                self.last_output = output;
            }
        } else {
            // Parse name=command format
            let arg = cmd.args.join(" ");
            if let Some((name, command)) = arg.split_once('=') {
                let name = name.trim().to_string();
                let command = command.trim().to_string();

                // Remove quotes if present
                let command = if (command.starts_with('"') && command.ends_with('"')) ||
                                 (command.starts_with('\'') && command.ends_with('\'')) {
                    command[1..command.len()-1].to_string()
                } else {
                    command
                };

                self.aliases.insert(name.clone(), command.clone());
                self.last_output = format!("{}Alias created: {}='{}'\n", icons::success().content, name, command);

                // Save to config
                self.config.shell.aliases = self.aliases.clone();
                if let Err(e) = self.config.save() {
                    tracing::warn!("Failed to save config: {}", e);
                }
            } else {
                self.last_output = format!("{}Invalid alias syntax: {}\n  Usage: alias name='command'\n", icons::error().content, arg);
            }
        }

        // Reset scroll
        self.output_scroll = 0;

        // Record in session
        self.session.record_command(
            format!("alias {}", cmd.args.join(" ")),
            Some(0),
            Some(0),
        );

        Ok(())
    }

    /// Handle Tab key autocompletion
    fn handle_autocomplete(&mut self) -> Result<()> {
        if self.command_buffer.is_empty() {
            return Ok(());
        }

        // Get completion results
        let working_dir = &self.session.state.working_directory;
        let result = self.autocompleter.complete(&self.command_buffer, working_dir)?;

        // If there's a unique completion or common prefix, apply it
        if !result.common_prefix.is_empty() && result.common_prefix != self.command_buffer {
            // Update the buffer with the common prefix
            // We need to replace the last token with the completion
            let tokens: Vec<&str> = self.command_buffer.split_whitespace().collect();

            if tokens.is_empty() {
                self.command_buffer = result.common_prefix.clone();
            } else if tokens.len() == 1 && !self.command_buffer.ends_with(' ') {
                // Completing first token (command)
                self.command_buffer = result.common_prefix.clone();
            } else {
                // Completing a path - replace last token
                let last_token = tokens.last().unwrap_or(&"");
                if let Some(idx) = self.command_buffer.rfind(last_token) {
                    self.command_buffer.truncate(idx);
                    self.command_buffer.push_str(&result.common_prefix);
                }
            }
        }

        // Store suggestions for display (limit to 10)
        self.completion_suggestions = result.completions.into_iter().take(10).collect();

        Ok(())
    }

    /// Toggle AI assistant mode
    pub fn toggle_ai_mode(&mut self) {
        if self.ai_provider.is_some() {
            self.ai_mode = !self.ai_mode;
            if self.ai_mode {
                // Clear AI input when entering AI mode
                self.ai_input_buffer.clear();
                self.ai_loading = false;
            }
        } else {
            self.last_output = format!("{}AI is not enabled. Configure it in ~/.config/arct/config.toml\n", icons::error().content);
        }
    }

    /// Toggle lesson mode
    pub fn toggle_lesson_mode(&mut self) {
        self.lesson_mode = !self.lesson_mode;

        if self.lesson_mode {
            // Initialize virtual filesystem
            match arct_core::VirtualFileSystem::new("nav-basics", &self.session_id) {
                Ok(vfs) => {
                    self.virtual_fs = Some(vfs);
                    self.last_output = format!("{}Lesson mode activated! You're now in a safe virtual filesystem.\n\nPress Ctrl+L again to return to normal mode.\n\nNavigate through lessons using the Learning panel on the right.\n", icons::lesson().content);
                }
                Err(e) => {
                    self.last_output = format!("{}Failed to initialize lesson environment: {}\n", icons::error().content, e);
                    self.lesson_mode = false;
                    return;
                }
            }

            // Initialize lesson panel if not already done
            if self.lesson_panel.is_none() {
                self.lesson_panel = Self::initialize_lesson_panel();
            }

            // Show lesson menu to let user choose a lesson
            if self.lesson_menu.is_none() {
                self.lesson_menu = Some(crate::panels::lesson_menu::LessonMenuPanel::new());
            }
        } else {
            // Clean up virtual filesystem
            self.virtual_fs = None;
            self.last_output = format!("{}Lesson mode deactivated. Back to normal shell mode and real filesystem.\n", icons::learning().content);
        }
    }

    /// Ask the AI assistant a question
    pub async fn ask_ai(&mut self, question: String) -> Result<()> {
        if self.ai_provider.is_none() {
            return Ok(());
        }

        if question.trim().is_empty() {
            return Ok(());
        }

        self.ai_loading = true;

        // Add user message to conversation
        self.ai_conversation.push(arct_ai::Message::user(question.clone()));

        // Build conversation with system prompt
        let user_name = self.config.general.user_name.as_deref().unwrap_or("there");
        let system_prompt = format!(
            "You are an AI teaching assistant integrated into Arc Academy Terminal, \
             an interactive terminal learning application. Your role is to help users \
             learn shell commands and terminal skills.\n\n\
             You're helping {}, so address them by name occasionally to make the \
             interaction personal and engaging.\n\n\
             Guidelines:\n\
             - Teach shell commands with clear, executable examples\n\
             - Explain concepts in beginner-friendly language\n\
             - Provide commands the user can type themselves in the terminal\n\
             - Keep responses concise (3-4 sentences or a short example)\n\
             - Focus on common Linux/Unix commands (bash, grep, find, etc.)\n\
             - Suggest safer alternatives when appropriate\n\
             - You are NOT Claude Code - you cannot execute commands or use tools\n\
             - You are a teaching assistant helping someone learn the terminal\n\
             - Be encouraging and supportive in your teaching approach",
            user_name
        );

        let mut messages = vec![
            arct_ai::Message::system(system_prompt),
        ];
        messages.extend(self.ai_conversation.clone());

        // Get response from AI
        // SAFETY: ai_provider is guaranteed to be Some when ai_mode is true
        // because toggle_ai_mode() checks ai_provider.is_some() before enabling
        let provider = self.ai_provider.as_ref()
            .expect("BUG: ai_provider must exist when ai_mode is true - this is a logic error");
        let response = provider.complete(&messages, None).await;

        self.ai_loading = false;

        match response {
            Ok(ai_response) => {
                // Strip markdown formatting for terminal display
                let cleaned_content = Self::strip_markdown(&ai_response.content);

                // Add assistant response to conversation
                self.ai_conversation.push(arct_ai::Message::assistant(ai_response.content.clone()));
                self.ai_response = Some(cleaned_content);
                Ok(())
            }
            Err(e) => {
                self.ai_response = Some(format!("{}Error: {}", icons::error().content, e));
                Err(anyhow::anyhow!("AI request failed: {}", e))
            }
        }
    }

    /// Clear AI conversation
    pub fn clear_ai_conversation(&mut self) {
        self.ai_conversation.clear();
        self.ai_response = None;
        self.ai_input_buffer.clear();
    }

    /// Strip markdown formatting from text for plain terminal display
    fn strip_markdown(text: &str) -> String {
        let mut result = String::new();
        let mut in_code_block = false;
        let mut skip_line = false;

        for line in text.lines() {
            // Toggle code block state
            if line.trim().starts_with("```") {
                in_code_block = !in_code_block;
                skip_line = true;
            }

            if skip_line {
                skip_line = false;
                continue;
            }

            // Clean the line
            let mut cleaned = line.to_string();

            // Remove headers (## Header -> Header)
            if cleaned.trim_start().starts_with('#') {
                cleaned = cleaned.trim_start().trim_start_matches('#').trim().to_string();
            }

            // Remove bold/italic markers
            cleaned = cleaned.replace("**", "").replace("*", "");

            // Remove inline code backticks (but not the content)
            cleaned = cleaned.replace('`', "");

            // Remove list markers (- item -> item, but keep indentation)
            if let Some(stripped) = cleaned.trim_start().strip_prefix("- ") {
                let indent = cleaned.len() - cleaned.trim_start().len();
                cleaned = format!("{}{}", " ".repeat(indent), stripped);
            }

            result.push_str(&cleaned);
            result.push('\n');
        }

        result.trim_end().to_string()
    }

    /// Handle onboarding wizard events
    async fn handle_onboarding_event(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(wizard) = self.onboarding.as_mut() {
            match key.code {
                KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT => {
                    wizard.handle_char(c);
                }
                KeyCode::Backspace => {
                    wizard.handle_backspace();
                }
                KeyCode::Up => {
                    wizard.handle_up();
                }
                KeyCode::Down => {
                    let max_options = match wizard.step {
                        crate::panels::onboarding::OnboardingStep::AskAI => 3,
                        crate::panels::onboarding::OnboardingStep::AskAIProvider => 3,
                        _ => 1,
                    };
                    wizard.handle_down(max_options);
                }
                KeyCode::Enter => {
                    wizard.handle_enter();

                    // Check if onboarding is complete
                    if wizard.step == crate::panels::onboarding::OnboardingStep::Complete {
                        // Save settings to config
                        if let Some(wizard) = self.onboarding.take() {
                            self.complete_onboarding(wizard).await?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Complete onboarding and save settings
    async fn complete_onboarding(&mut self, wizard: crate::panels::onboarding::OnboardingWizard) -> Result<()> {
        // Update config with onboarding results
        if !wizard.user_name.is_empty() {
            self.config.general.user_name = Some(wizard.user_name.clone());
        }

        if let Some(ai_enabled) = wizard.ai_enabled {
            self.config.ai.enabled = ai_enabled;

            if ai_enabled {
                // Configure AI provider based on user selection
                if wizard.ai_provider.as_deref() == Some("claude-code") {
                    // Claude Code CLI for Max subscribers
                    self.config.ai.provider = "claude-cli".to_string();
                    self.config.ai.model = Some("claude-sonnet-4".to_string());
                    // No API key needed - uses Claude Code authentication
                } else if wizard.ai_provider.as_deref() == Some("own") {
                    // User has their own API key - default to local LLM
                    self.config.ai.provider = "local".to_string();
                    self.config.ai.endpoint = Some("http://localhost:11434".to_string());
                    self.config.ai.model = Some("llama3.2".to_string());
                } else if wizard.ai_provider.as_deref() == Some("managed") {
                    // Arc Academy managed service
                    self.config.ai.provider = "managed".to_string();
                }
            }
        }

        // Mark setup as complete
        self.config.general.setup_complete = true;

        // Save config
        self.config.save()?;

        // Reinitialize AI provider with new settings
        if self.config.ai.enabled {
            match Self::create_ai_provider(&self.config.ai) {
                Ok(provider) => {
                    self.ai_provider = Some(provider);
                }
                Err(e) => {
                    // Log error but don't fail onboarding
                    self.last_output = format!("{}AI provider initialization failed: {}\n", icons::warning().content, e);
                }
            }
        }

        // Remove onboarding
        self.onboarding = None;

        // Show greeting in output
        let name = self.config.general.user_name.as_deref().unwrap_or("there");
        let mut welcome_msg = format!(
            "{}Welcome, {}!\n\n\
             You're all set to start learning shell commands!\n\n",
            icons::celebration().content, name
        );

        // Add provider-specific setup notes if AI is enabled
        if self.config.ai.enabled {
            match self.config.ai.provider.as_str() {
                "claude-cli" => {
                    welcome_msg.push_str(
                        &format!("{}Using Claude Code CLI - your Max subscription is ready!\n\
                         Press Ctrl+A to ask Claude for help.\n\n", icons::ai().content)
                    );
                }
                "anthropic" | "openai" => {
                    welcome_msg.push_str(
                        &format!("{}To use AI features, set your API key:\n\
                         export ARCT_AI_API_KEY=\"your-api-key-here\"\n\n", icons::note().content)
                    );
                }
                "local" => {
                    welcome_msg.push_str(
                        &format!("{}Using local LLM - make sure your server is running!\n\n", icons::info().content)
                    );
                }
                _ => {}
            }
        }

        welcome_msg.push_str("Press ? for help, or just start typing commands.\n");
        self.last_output = welcome_msg;

        Ok(())
    }

    /// Handle settings panel events
    async fn handle_settings_event(&mut self, key: KeyEvent) -> Result<()> {
        // Determine what action to take
        let (action, selected_field) = {
            let panel = match self.settings_panel.as_ref() {
                Some(p) => p,
                None => return Ok(()),
            };

            let action = if panel.editing {
                // In edit mode
                match key.code {
                    KeyCode::Char(c) if key.modifiers == KeyModifiers::NONE || key.modifiers == KeyModifiers::SHIFT => {
                        SettingsAction::PushChar(c)
                    }
                    KeyCode::Backspace => SettingsAction::PopChar,
                    KeyCode::Enter => SettingsAction::SaveEdit,
                    KeyCode::Esc => SettingsAction::CancelEdit,
                    _ => SettingsAction::None,
                }
            } else {
                // In navigation mode
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                        SettingsAction::PreviousField
                    }
                    KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                        SettingsAction::NextField
                    }
                    KeyCode::Enter => SettingsAction::StartEdit,
                    KeyCode::Esc | KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                        SettingsAction::Close
                    }
                    _ => SettingsAction::None,
                }
            };

            (action, panel.selected_field)
        };

        // Execute the action
        match action {
            SettingsAction::PushChar(c) => {
                if let Some(panel) = self.settings_panel.as_mut() {
                    panel.push_char(c);
                }
            }
            SettingsAction::PopChar => {
                if let Some(panel) = self.settings_panel.as_mut() {
                    panel.pop_char();
                }
            }
            SettingsAction::SaveEdit => {
                if let Some(panel) = self.settings_panel.as_mut() {
                    panel.save_edit(&mut self.config)?;

                    // If theme was changed, reload it
                    if selected_field == crate::panels::settings::SettingField::Theme {
                        self.theme = Theme::from_name(&self.config.theme.default_theme);
                    }

                    // If AI was toggled, reload provider
                    if selected_field == crate::panels::settings::SettingField::AIEnabled {
                        if self.config.ai.enabled {
                            // Try to initialize AI provider
                            match Self::create_ai_provider(&self.config.ai) {
                                Ok(provider) => {
                                    self.ai_provider = Some(provider);
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to initialize AI provider: {}", e);
                                    self.ai_provider = None;
                                }
                            }
                        } else {
                            self.ai_provider = None;
                            self.ai_mode = false;
                        }
                    }
                }
            }
            SettingsAction::CancelEdit => {
                if let Some(panel) = self.settings_panel.as_mut() {
                    panel.cancel_editing();
                }
            }
            SettingsAction::PreviousField => {
                if let Some(panel) = self.settings_panel.as_mut() {
                    panel.previous_field();
                }
            }
            SettingsAction::NextField => {
                if let Some(panel) = self.settings_panel.as_mut() {
                    panel.next_field();
                }
            }
            SettingsAction::StartEdit => {
                if let Some(panel) = self.settings_panel.as_mut() {
                    panel.start_editing(&self.config);
                }
            }
            SettingsAction::Close => {
                self.settings_panel = None;
            }
            SettingsAction::None => {}
        }

        Ok(())
    }

    /// Toggle achievements panel
    pub fn toggle_achievements_panel(&mut self) {
        if self.achievements_panel.is_some() {
            self.achievements_panel = None;
        } else {
            self.achievements_panel = Some(crate::panels::achievements::AchievementsPanel::new());
        }
    }

    /// Toggle progress panel
    pub fn toggle_progress_panel(&mut self) {
        if self.progress_panel.is_some() {
            self.progress_panel = None;
        } else {
            self.progress_panel = Some(crate::panels::progress::ProgressPanel::new());
        }
    }

    /// Toggle challenges panel
    pub fn toggle_challenges_panel(&mut self) {
        if self.challenges_panel.is_some() {
            self.challenges_panel = None;
        } else {
            self.challenges_panel = Some(crate::panels::challenges::ChallengesPanel::new());
        }
    }

    /// Check for newly unlocked achievements and queue notifications
    pub fn check_and_unlock_achievements(&mut self) {
        let newly_unlocked = self.user_stats.check_achievements();

        // Add to pending queue
        for achievement in newly_unlocked {
            self.pending_achievements.push(achievement);
        }

        // Show first notification if we're not already showing one
        if self.showing_notification.is_none() && !self.pending_achievements.is_empty() {
            self.show_next_achievement_notification();
        }
    }

    /// Show the next achievement notification from the queue
    fn show_next_achievement_notification(&mut self) {
        if let Some(achievement) = self.pending_achievements.first() {
            self.showing_notification = Some(crate::panels::notification::NotificationPanel::new(
                achievement.clone(),
            ));
        }
    }

    /// Dismiss the current achievement notification and show next if any
    fn dismiss_notification(&mut self) {
        self.showing_notification = None;

        // Remove the first achievement from queue if it was shown
        if !self.pending_achievements.is_empty() {
            self.pending_achievements.remove(0);
        }

        // Show next notification if there are more
        if !self.pending_achievements.is_empty() {
            self.show_next_achievement_notification();
        }
    }

    /// Record command execution for stats tracking
    pub fn record_command_for_stats(&mut self, command: &str) {
        // Extract just the command name (first word)
        let command_name = command.split_whitespace().next().unwrap_or("");
        if !command_name.is_empty() {
            self.user_stats.record_command_use(command_name.to_string());
        }
    }

    /// Record lesson completion and check for achievements
    pub fn record_lesson_completion(&mut self, lesson_id: String, difficulty: arct_core::Difficulty) {
        // Record in stats
        self.user_stats.record_lesson_completion(lesson_id.clone(), difficulty);

        // Add to completed lessons set
        self.completed_lessons.insert(lesson_id);

        // Check for newly unlocked achievements
        self.check_and_unlock_achievements();
    }
}

/// Actions that can be performed in the settings panel
enum SettingsAction {
    PushChar(char),
    PopChar,
    SaveEdit,
    CancelEdit,
    PreviousField,
    NextField,
    StartEdit,
    Close,
    None,
}
