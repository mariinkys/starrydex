// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, CheckBytes, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct StarryPokemon {
    pub pokemon: StarryPokemonData,
    pub sprite_path: Option<String>,
    pub encounter_info: Option<Vec<StarryPokemonEncounterInfo>>,
}

impl Debug for StarryPokemon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StarryPokemon")
            .field("pokemon", &self.pokemon.id)
            .finish()
    }
}

impl StarryPokemon {
    /// Returns the total value of all the stats of the Pokémon
    pub fn get_total_stats(&self) -> i64 {
        self.pokemon.stats.hp
            + self.pokemon.stats.attack
            + self.pokemon.stats.defense
            + self.pokemon.stats.sp_attack
            + self.pokemon.stats.sp_defense
            + self.pokemon.stats.speed
    }
}

/// Core Pokémon data
#[derive(Archive, CheckBytes, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonData {
    pub id: i64,
    pub name: String,
    pub weight: i64,
    pub height: i64,
    pub types: Vec<String>,
    pub abilities: Vec<String>,
    pub stats: StarryPokemonStats,
}

impl Debug for StarryPokemonData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StarryPokemonData")
            .field("id", &self.id)
            .finish()
    }
}

/// Pokémon statistics
#[derive(Archive, CheckBytes, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub sp_attack: i64,
    pub sp_defense: i64,
    pub speed: i64,
}

impl Debug for StarryPokemonStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StarryPokemonStats").finish()
    }
}

/// Pokémon encounter information
#[derive(Archive, CheckBytes, Serialize, Deserialize, Clone)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonEncounterInfo {
    pub city: String,
    pub games_method: Vec<String>,
}

impl Debug for StarryPokemonEncounterInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StarryPokemonEncounterInfo").finish()
    }
}

// Simple owned data structure, for list Page
#[derive(Clone)]
pub struct PokemonInfo {
    pub id: i64,
    pub name: String,
    pub sprite_path: Option<String>,
}

impl Debug for PokemonInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PokemonInfo").field("id", &self.id).finish()
    }
}
