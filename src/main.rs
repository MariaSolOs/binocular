use anyhow::{Context, Result};
use std::panic;

use binocular::{pickers::GrepPicker, App, Tui};

#[tokio::main]
async fn main() -> Result<()> {
    // Make sure we cleanup when panicking.
    let original_panic = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        Tui::shutdown();
        original_panic(panic_info);
    }));

    // Initialize the application.
    let mut tui = Tui::setup().context("Failed to setup terminal")?;
    let mut app = App::new(GrepPicker);

    // Application loop.
    let res = app
        .run(&mut tui)
        .await
        .context("Failed to run the application");

    // Cleanup.
    Tui::shutdown();

    res
}
