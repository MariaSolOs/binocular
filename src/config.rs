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

        // Load the user's configuration file and merge it with the defaults.
        let config = match fs::read_to_string(config_dir.join(CONFIG_FILE)) {
            Ok(user_config) => {
                serde_json::from_str(&user_config).context("Failed to parse configuration file")?
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(err) => return Err(err).context("Failed to read configuration file"),
        };

        Ok(config)
    }

    pub(crate) fn base_color(&self) -> Color {
        self.colors.base.unwrap_or(Color::LightCyan)
    }
}
