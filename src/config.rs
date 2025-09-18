// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

const CONFIG_VERSION: u64 = 2;

/// Contains the configurations fields of the application
#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
pub struct StarryConfig {
    pub app_theme: AppTheme,
    pub pokemon_per_row: usize,
    pub items_per_page: usize,
    pub type_filtering_mode: TypeFilteringMode,
}

impl Default for StarryConfig {
    fn default() -> Self {
        Self {
            app_theme: Default::default(),
            pokemon_per_row: 3,
            type_filtering_mode: Default::default(),
            items_per_page: 30,
        }
    }
}

impl StarryConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(crate::core::APP_ID, CONFIG_VERSION).ok()
    }

    pub fn config() -> StarryConfig {
        match Self::config_handler() {
            Some(config_handler) => {
                StarryConfig::get_entry(&config_handler).unwrap_or_else(|(error, config)| {
                    eprintln!("Error whilst loading config: {error:#?}");
                    config
                })
            }
            None => StarryConfig::default(),
        }
    }
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum TypeFilteringMode {
    Inclusive,
    #[default]
    Exclusive,
}
