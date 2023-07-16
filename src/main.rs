use anyhow::{Context, Result};
use binocular::tui::Tui;

fn main() -> Result<()> {
    let mut tui = Tui::setup().context("failed to setup terminal")?;


    tui.shutdown().context("failed to shutdown terminal")
}
