/// Application.
mod app;
pub use app::App;

/// List item representing a `ripgrep` result.
mod rg_item;

/// Terminal user interface.
mod tui;
pub use tui::Tui;
