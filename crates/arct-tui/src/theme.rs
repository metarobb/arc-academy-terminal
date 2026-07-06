//! Theme and color scheme management

use ratatui::style::{Color, Style, Modifier};

/// Theme for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,

    // Background colors
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,

    // Foreground colors
    pub fg_primary: Color,
    pub fg_secondary: Color,
    pub fg_dim: Color,

    // Accent colors
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // UI elements
    pub border: Color,
    pub border_focused: Color,
    pub selection: Color,
}

impl Theme {
    /// Arc Academy Orange theme (default)
    pub fn arc_academy_orange() -> Self {
        Self {
            name: "Arc Academy Orange".to_string(),

            // Dark background with orange accents
            bg_primary: Color::Rgb(18, 18, 20),      // #121214 - Very dark
            bg_secondary: Color::Rgb(26, 26, 28),    // #1a1a1c - Dark
            bg_tertiary: Color::Rgb(32, 32, 36),     // #202024 - Lighter dark

            fg_primary: Color::Rgb(230, 230, 235),   // #e6e6eb - Bright text
            fg_secondary: Color::Rgb(180, 180, 190), // #b4b4be - Secondary text
            fg_dim: Color::Rgb(120, 120, 130),       // #787882 - Dim text

            accent: Color::Rgb(255, 107, 53),        // #FF6B35 - Arc Orange
            success: Color::Rgb(80, 200, 120),       // #50C878 - Emerald green
            warning: Color::Rgb(255, 179, 71),       // #FFB347 - Warm orange
            error: Color::Rgb(255, 82, 82),          // #FF5252 - Red
            info: Color::Rgb(100, 181, 246),         // #64B5F6 - Blue

            border: Color::Rgb(60, 60, 65),          // #3c3c41
            border_focused: Color::Rgb(255, 107, 53), // #FF6B35 - Arc Orange
            selection: Color::Rgb(80, 50, 35),       // #503223 - Dark orange
        }
    }

    /// Arc Academy Green theme
    pub fn arc_academy_green() -> Self {
        Self {
            name: "Arc Academy Green".to_string(),

            // Dark background with green accents
            bg_primary: Color::Rgb(16, 20, 18),      // #101412 - Very dark green tint
            bg_secondary: Color::Rgb(22, 27, 24),    // #161b18 - Dark green tint
            bg_tertiary: Color::Rgb(28, 34, 30),     // #1c221e - Lighter dark green

            fg_primary: Color::Rgb(230, 240, 235),   // #e6f0eb - Bright text with green tint
            fg_secondary: Color::Rgb(180, 200, 190), // #b4c8be - Secondary text
            fg_dim: Color::Rgb(120, 140, 130),       // #788c82 - Dim text

            accent: Color::Rgb(76, 175, 80),         // #4CAF50 - Arc Green
            success: Color::Rgb(102, 187, 106),      // #66BB6A - Success green
            warning: Color::Rgb(255, 193, 7),        // #FFC107 - Amber
            error: Color::Rgb(244, 67, 54),          // #F44336 - Red
            info: Color::Rgb(41, 182, 246),          // #29B6F6 - Cyan

            border: Color::Rgb(50, 65, 55),          // #324137
            border_focused: Color::Rgb(76, 175, 80), // #4CAF50 - Arc Green
            selection: Color::Rgb(30, 50, 35),       // #1e3223 - Dark green
        }
    }

    /// Arc Dark theme (Catppuccin inspired - fallback)
    pub fn arc_dark() -> Self {
        Self {
            name: "Arc Dark".to_string(),

            // Catppuccin Mocha inspired
            bg_primary: Color::Rgb(30, 30, 46),      // #1e1e2e
            bg_secondary: Color::Rgb(24, 24, 37),    // #181825
            bg_tertiary: Color::Rgb(17, 17, 27),     // #11111b

            fg_primary: Color::Rgb(205, 214, 244),   // #cdd6f4
            fg_secondary: Color::Rgb(186, 194, 222), // #bac2de
            fg_dim: Color::Rgb(108, 112, 134),       // #6c7086

            accent: Color::Rgb(137, 180, 250),       // #89b4fa (blue)
            success: Color::Rgb(166, 227, 161),      // #a6e3a1 (green)
            warning: Color::Rgb(249, 226, 175),      // #f9e2af (yellow)
            error: Color::Rgb(243, 139, 168),        // #f38ba8 (red)
            info: Color::Rgb(148, 226, 213),         // #94e2d5 (teal)

            border: Color::Rgb(108, 112, 134),       // #6c7086
            border_focused: Color::Rgb(137, 180, 250), // #89b4fa
            selection: Color::Rgb(88, 91, 112),      // #585b70
        }
    }

    /// Arc Light theme
    pub fn arc_light() -> Self {
        Self {
            name: "Arc Light".to_string(),

            bg_primary: Color::Rgb(239, 241, 245),   // #eff1f5
            bg_secondary: Color::Rgb(230, 233, 239), // #e6e9ef
            bg_tertiary: Color::Rgb(220, 224, 232),  // #dce0e8

            fg_primary: Color::Rgb(76, 79, 105),     // #4c4f69
            fg_secondary: Color::Rgb(92, 95, 119),   // #5c5f77
            fg_dim: Color::Rgb(156, 160, 176),       // #9ca0b0

            accent: Color::Rgb(30, 102, 245),        // #1e66f5
            success: Color::Rgb(64, 160, 43),        // #40a02b
            warning: Color::Rgb(223, 142, 29),       // #df8e1d
            error: Color::Rgb(210, 15, 57),          // #d20f39
            info: Color::Rgb(4, 165, 229),           // #04a5e5

            border: Color::Rgb(156, 160, 176),       // #9ca0b0
            border_focused: Color::Rgb(30, 102, 245), // #1e66f5
            selection: Color::Rgb(204, 208, 218),    // #ccd0da
        }
    }

    /// "Night" theme — deep blue/purple background with cyan/blue accents
    /// (Tokyo-night-style palette)
    pub fn night() -> Self {
        Self {
            name: "Night".to_string(),

            bg_primary: Color::Rgb(26, 27, 38),      // #1a1b26 - Deep night blue
            bg_secondary: Color::Rgb(22, 22, 30),    // #16161e - Darker
            bg_tertiary: Color::Rgb(41, 42, 60),     // #292a3c - Raised surface

            fg_primary: Color::Rgb(192, 202, 245),   // #c0caf5 - Pale periwinkle
            fg_secondary: Color::Rgb(169, 177, 214), // #a9b1d6 - Muted lavender
            fg_dim: Color::Rgb(86, 95, 137),         // #565f89 - Storm gray

            accent: Color::Rgb(122, 162, 247),       // #7aa2f7 - Night blue
            success: Color::Rgb(158, 206, 106),      // #9ece6a - Spring green
            warning: Color::Rgb(224, 175, 104),      // #e0af68 - Amber
            error: Color::Rgb(247, 118, 142),        // #f7768e - Rose
            info: Color::Rgb(125, 207, 255),         // #7dcfff - Sky cyan

            border: Color::Rgb(59, 66, 97),          // #3b4261
            border_focused: Color::Rgb(187, 154, 247), // #bb9af7 - Purple glow
            selection: Color::Rgb(52, 59, 88),       // #343b58 - Indigo wash
        }
    }

    /// "Mocha" theme — soft pastels on a warm dark base
    /// (Catppuccin-mocha-style palette with mauve/peach accents)
    pub fn mocha() -> Self {
        Self {
            name: "Mocha".to_string(),

            bg_primary: Color::Rgb(30, 30, 46),      // #1e1e2e - Base
            bg_secondary: Color::Rgb(24, 24, 37),    // #181825 - Mantle
            bg_tertiary: Color::Rgb(49, 50, 68),     // #313244 - Surface

            fg_primary: Color::Rgb(205, 214, 244),   // #cdd6f4 - Text
            fg_secondary: Color::Rgb(166, 173, 200), // #a6adc8 - Subtext
            fg_dim: Color::Rgb(127, 132, 156),       // #7f849c - Overlay

            accent: Color::Rgb(203, 166, 247),       // #cba6f7 - Mauve
            success: Color::Rgb(166, 227, 161),      // #a6e3a1 - Green
            warning: Color::Rgb(250, 179, 135),      // #fab387 - Peach
            error: Color::Rgb(243, 139, 168),        // #f38ba8 - Pink
            info: Color::Rgb(137, 220, 235),         // #89dceb - Sky

            border: Color::Rgb(69, 71, 90),          // #45475a
            border_focused: Color::Rgb(203, 166, 247), // #cba6f7 - Mauve
            selection: Color::Rgb(88, 91, 112),      // #585b70
        }
    }

    /// Every registered theme, in Ctrl+T cycle order.
    ///
    /// This is the single registry: `cycle_next`, `from_name`, the command
    /// palette, the settings hint, and the `--theme` CLI flag all derive
    /// their theme lists from here.
    pub fn all() -> Vec<Theme> {
        vec![
            Self::arc_academy_orange(),
            Self::arc_academy_green(),
            Self::arc_dark(),
            Self::arc_light(),
            Self::night(),
            Self::mocha(),
        ]
    }

    /// Names of every registered theme, in cycle order
    pub fn all_names() -> Vec<String> {
        Self::all().into_iter().map(|t| t.name).collect()
    }

    /// Get a style for normal text
    pub fn style_normal(&self) -> Style {
        Style::default().fg(self.fg_primary).bg(self.bg_primary)
    }

    /// Get a style for secondary text
    pub fn style_secondary(&self) -> Style {
        Style::default().fg(self.fg_secondary).bg(self.bg_primary)
    }

    /// Get a style for dim text
    pub fn style_dim(&self) -> Style {
        Style::default().fg(self.fg_dim).bg(self.bg_primary)
    }

    /// Get a style for headers
    pub fn style_header(&self) -> Style {
        Style::default()
            .fg(self.fg_primary)
            .bg(self.bg_secondary)
            .add_modifier(Modifier::BOLD)
    }

    /// Get a style for focused borders
    pub fn style_border_focused(&self) -> Style {
        Style::default().fg(self.border_focused)
    }

    /// Get a style for unfocused borders
    pub fn style_border(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// Get a style for accent text
    pub fn style_accent(&self) -> Style {
        Style::default().fg(self.accent).add_modifier(Modifier::BOLD)
    }

    /// Get a style for success messages
    pub fn style_success(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Get a style for warnings
    pub fn style_warning(&self) -> Style {
        Style::default().fg(self.warning).add_modifier(Modifier::BOLD)
    }

    /// Get a style for errors
    pub fn style_error(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    /// Get a style for info messages
    pub fn style_info(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Get a style for selections
    pub fn style_selection(&self) -> Style {
        Style::default()
            .bg(self.selection)
            .fg(self.fg_primary)
    }

    /// Get a style for panel/block backgrounds
    /// This ensures panels have the correct background color (important for light themes)
    pub fn style_block(&self) -> Style {
        Style::default().bg(self.bg_primary).fg(self.fg_primary)
    }

    /// Panel title style: bold accent when the panel is focused, dim when not.
    /// Use together with `style_border_focused`/`style_border` so the active
    /// panel is unmistakable.
    pub fn style_title(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(self.fg_dim)
        }
    }

    /// Style for the footer hint bar background strip
    pub fn style_footer(&self) -> Style {
        Style::default().bg(self.bg_tertiary).fg(self.fg_dim)
    }

    /// Style for key "chips" in the footer hint bar
    pub fn style_footer_key(&self) -> Style {
        Style::default()
            .bg(self.bg_tertiary)
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::arc_academy_orange()
    }
}

impl Theme {
    /// Cycle to the next theme (Ctrl+T) — wraps around the full registry
    pub fn cycle_next(&self) -> Self {
        let themes = Self::all();
        let idx = themes.iter().position(|t| t.name == self.name);
        match idx {
            Some(i) => themes[(i + 1) % themes.len()].clone(),
            None => Self::default(),
        }
    }

    /// Get theme by name (exact match against the registry)
    pub fn from_name(name: &str) -> Self {
        match Self::all().into_iter().find(|t| t.name == name) {
            Some(theme) => theme,
            None => {
                tracing::warn!("Unknown theme '{}', using default", name);
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_registry_has_six_unique_themes() {
        let names = Theme::all_names();
        assert_eq!(names.len(), 6);
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len(), "theme names must be unique");
        for expected in [
            "Arc Academy Orange",
            "Arc Academy Green",
            "Arc Dark",
            "Arc Light",
            "Night",
            "Mocha",
        ] {
            assert!(
                names.iter().any(|n| n == expected),
                "registry is missing theme '{}'",
                expected
            );
        }
    }

    #[test]
    fn test_from_name_roundtrip_for_every_registered_theme() {
        for name in Theme::all_names() {
            assert_eq!(Theme::from_name(&name).name, name);
        }
    }

    #[test]
    fn test_from_name_unknown_falls_back_to_default() {
        assert_eq!(Theme::from_name("Solarized").name, Theme::default().name);
    }

    #[test]
    fn test_cycle_visits_every_theme_and_wraps() {
        let mut theme = Theme::default();
        let mut visited = Vec::new();
        for _ in 0..Theme::all().len() {
            visited.push(theme.name.clone());
            theme = theme.cycle_next();
        }
        // After a full cycle we're back at the start
        assert_eq!(theme.name, Theme::default().name);
        // And every registered theme was visited exactly once
        let unique: HashSet<_> = visited.iter().collect();
        assert_eq!(unique.len(), Theme::all().len());
    }
}
