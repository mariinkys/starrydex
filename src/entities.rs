// SPDX-License-Identifier: GPL-3.0-only

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, CheckBytes, Serialize, Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct StarryPokemon {
    pub pokemon: StarryPokemonData,
    pub sprite_path: Option<String>,
    pub encounter_info: Option<Vec<StarryPokemonEncounterInfo>>,
}

/// Core Pokémon data
#[derive(Archive, CheckBytes, Serialize, Deserialize, Debug)]
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

/// Pokémon statistics
#[derive(Archive, CheckBytes, Serialize, Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub sp_attack: i64,
    pub sp_defense: i64,
    pub speed: i64,
}

/// Pokémon encounter information
#[derive(Archive, CheckBytes, Serialize, Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonEncounterInfo {
    pub city: String,
    pub games_method: Vec<String>,
}

// Simple owned data structure, for list Page
#[derive(Debug, Clone)]
pub struct PokemonInfo {
    pub id: i64,
    pub name: String,
    pub sprite_path: Option<String>,
}
