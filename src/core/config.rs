use std::path::PathBuf;

use crate::app::StarryDex;
use cosmic::{
    cosmic_config::{self, cosmic_config_derive::CosmicConfigEntry, Config, CosmicConfigEntry},
    theme, Application,
};
use serde::{Deserialize, Serialize};

pub const CONFIG_VERSION: u64 = 1;

#[derive(Clone, Default, Debug, Eq, PartialEq, Deserialize, Serialize, CosmicConfigEntry)]
pub struct StarryDexConfig {
    pub app_theme: AppTheme,
    pub repositories: Vec<Repository>,
}

impl StarryDexConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(StarryDex::APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> StarryDexConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                StarryDexConfig::get_entry(&config_handler).unwrap_or_else(|(errs, config)| {
                    log::info!("errors loading config: {:?}", errs);
                    config
                })
            }
            None => StarryDexConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum AppTheme {
    Dark,
    Light,
    #[default]
    System,
}

impl AppTheme {
    pub fn theme(&self) -> theme::Theme {
        match self {
            Self::Dark => theme::Theme::dark(),
            Self::Light => theme::Theme::light(),
            Self::System => theme::system_preference(),
        }
    }
}
