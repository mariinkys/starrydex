// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{self, Config, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

const APP_ID: &str = "dev.mariinkys.StarryDex";
const CONFIG_VERSION: u64 = 3;

/// Contains the configurations fields of the application
#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
pub struct StarryConfig {
    pub app_theme: AppTheme,
    pub view_mode: ViewMode,
    pub pokemon_per_page: usize,
    pub type_filtering_mode: TypeFilteringMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum ViewMode {
    Manual { pokemon_per_row: usize },
    Responsive,
}

impl Default for StarryConfig {
    fn default() -> Self {
        Self {
            app_theme: Default::default(),
            view_mode: ViewMode::Responsive,
            type_filtering_mode: Default::default(),
            pokemon_per_page: 30,
        }
    }
}

impl StarryConfig {
    pub fn config_handler() -> Option<Config> {
        Config::new(APP_ID, CONFIG_VERSION).ok()
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

/// Represents the different inputs that can happen in the config [`ContextPage`]
#[derive(Debug, Clone)]
pub enum ConfigInput {
    /// Update the application theme
    UpdateTheme(usize),
    /// Update the current view mode
    UpdateViewMode(usize),
    /// Update the pokemon per row setting
    UpdatePokemonPerRow(u16),
    /// Update the pokemon per page setting
    UpdatePokemonPerPage(u16),
    /// Update the type filtering mode setting
    UpdateTypeFilterMode(usize),
    /// Ask to delete and recreate the app cache
    DeleteCache,
}
