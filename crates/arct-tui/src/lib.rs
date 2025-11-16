//! Arc Academy Terminal - TUI Interface
//!
//! Beautiful terminal UI using ratatui

pub mod app;
pub mod ui;
pub mod events;
pub mod panels;
pub mod theme;
pub mod shell;
pub mod ansi;
pub mod persistence;
pub mod autocomplete;
pub mod analytics;
pub mod icons;

pub use app::App;
pub use theme::Theme;

use anyhow::Result;

/// Run the TUI application
pub async fn run() -> Result<()> {
    let mut app = App::new()?;
    app.run().await
}
