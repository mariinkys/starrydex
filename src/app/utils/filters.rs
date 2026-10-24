// SPDX-License-Identifier: GPL-3.0

use std::collections::BTreeSet;

use starrydex_model::{Generation, PokemonType};

/// Rarity bucket of a species.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Regular,
    Baby,
    Legendary,
    Mythical,
}

impl Category {
    pub const ALL: &'static [Self] = &[Self::Regular, Self::Baby, Self::Legendary, Self::Mythical];
}

/// Different filters you can apply to the Pokémon list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filters {
    pub types: BTreeSet<PokemonType>,
    pub generations: BTreeSet<Generation>,
    pub categories: BTreeSet<Category>,
    pub stats_enabled: bool,
    pub min_total: u16,
}

impl Default for Filters {
    fn default() -> Self {
        Self {
            types: BTreeSet::new(),
            generations: BTreeSet::new(),
            categories: BTreeSet::new(),
            stats_enabled: false,
            min_total: 400,
        }
    }
}

impl Filters {
    pub fn any_applied(&self) -> bool {
        !self.types.is_empty()
            || !self.generations.is_empty()
            || !self.categories.is_empty()
            || self.stats_enabled
    }

    /// Number of active filter groups (shown as a badge on the filter button).
    pub fn active_count(&self) -> usize {
        usize::from(!self.types.is_empty())
            + usize::from(!self.generations.is_empty())
            + usize::from(!self.categories.is_empty())
            + usize::from(self.stats_enabled)
    }
}

/// Order of the Pokémon list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortBy {
    #[default]
    Number,
    Name,
    Total,
    Hp,
    Attack,
    Defense,
    SpAttack,
    SpDefense,
    Speed,
    Height,
    Weight,
}

impl SortBy {
    pub const ALL: &'static [Self] = &[
        Self::Number,
        Self::Name,
        Self::Total,
        Self::Hp,
        Self::Attack,
        Self::Defense,
        Self::SpAttack,
        Self::SpDefense,
        Self::Speed,
        Self::Height,
        Self::Weight,
    ];
}
