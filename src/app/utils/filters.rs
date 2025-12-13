// SPDX-License-Identifier: GPL-3.0

use std::collections::HashSet;

use crate::app::entities::{StarryPokemonGeneration, StarryPokemonType};

/// Different filters you can apply to the Pokémon List
pub struct Filters {
    pub selected_types: HashSet<StarryPokemonType>,
    pub selected_generations: HashSet<StarryPokemonGeneration>,
    pub total_stats: (bool, i64),
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            selected_types: HashSet::new(),
            total_stats: (false, 50),
            selected_generations: HashSet::new(),
        }
    }
}

impl Filters {
    pub fn any_applied(&self) -> bool {
        if !self.selected_types.is_empty()
            || !self.selected_generations.is_empty()
            || self.total_stats.0
        {
            return true;
        }

        false
    }
}
