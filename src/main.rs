use anyhow::{Context, Result};

use binocular::{App, Tui};

fn main() -> Result<()> {
    // Initialize the application.
    let mut tui = Tui::setup().context("Failed to setup terminal")?;
    let mut app = App::new();

    // Application loop.
    let res = app.run(&mut tui).context("Failed to run the application");

    // Cleanup.
    tui.shutdown();

    res
}
