use anyhow::{anyhow, Context, Result};
use ratatui::style::Color;
use serde::Deserialize;
use std::{fs, io};

/// `binocular`'s configuration folder name.
const CONFIG_DIR: &str = "binocular";

/// `binocular`'s configuration file name.
const CONFIG_FILE: &str = "config.json";

#[derive(Deserialize, PartialEq)]
#[derive(Default, Deserialize)]
#[serde(default)]
pub struct ConfigColors {
    base: Option<Color>,
    filepath: Option<Color>,
    selection: Option<Color>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Config {
    colors: ConfigColors,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Create the configuration directory if needed.
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Failed to find a configuration directory"))?;
        let config_dir = config_dir.join(CONFIG_DIR);
        fs::create_dir_all(&config_dir).context("Failed to create configuration directory")?;

        // Load the user's configuration file (if it exists).
        let config = match fs::read_to_string(config_dir.join(CONFIG_FILE)) {
            Ok(user_config) => {
                serde_json::from_str(&user_config).context("Failed to parse configuration file")?
            }
            // If the configuration file doesn't exist, just use the defaults.
            Err(err) if err.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(err) => return Err(err).context("Failed to read configuration file"),
        };

        Ok(config)
    }

    /// Returns the base UI color. Used for borders, titles, and other general UI elements.
    /// Defaults to [Color::LightCyan].
    pub(crate) fn base_color(&self) -> Color {
        self.colors.base.unwrap_or(Color::LightCyan)
    }

    /// Returns the filepath color. Used for the filepath in the results list.
    /// Defaults to [Color::LightBlue].
    pub(crate) fn filepath_color(&self) -> Color {
        self.colors.filepath.unwrap_or(Color::LightBlue)
    }

    /// Returns the selection color. Used for the currently selected item in the results list.
    /// Defaults to [Color::Yellow].
    pub(crate) fn selection_color(&self) -> Color {
        self.colors.selection.unwrap_or(Color::Yellow)
    }
}
