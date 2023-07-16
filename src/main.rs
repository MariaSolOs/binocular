use anyhow::{Context, Result};

use binocular::{app::App, tui::Tui};

fn main() -> Result<()> {
    // Initialize the application.
    let mut tui = Tui::setup().context("failed to setup terminal")?;
    let mut app = App::new();

    // Application loop.
    app.run(&mut tui)?;

    // Cleanup.
    tui.shutdown().context("failed to shutdown terminal")
}
