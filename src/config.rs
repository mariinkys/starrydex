// SPDX-License-Identifier: GPL-3.0-only

use cosmic::{
    cosmic_config::{self, CosmicConfigEntry, cosmic_config_derive::CosmicConfigEntry},
    theme,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, CosmicConfigEntry, Eq, PartialEq)]
#[version = 1]
pub struct Config {
    pub app_theme: AppTheme,
    pub first_run_completed: bool,
    pub pokemon_per_row: usize,
    pub items_per_page: usize,
    pub type_filtering_mode: TypeFilteringMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_theme: Default::default(),
            first_run_completed: false,
            pokemon_per_row: 3,
            type_filtering_mode: Default::default(),
            items_per_page: 30,
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
