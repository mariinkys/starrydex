// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

use crate::fl;

/// Main Pokemon structure with all the info we want to display about it
#[derive(Archive, CheckBytes, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct StarryPokemon {
    pub pokemon: StarryPokemonData,
    pub specie: Option<StarryPokemonSpecie>,
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

/// Pokémon specie
#[derive(Archive, CheckBytes, Serialize, Deserialize)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonSpecie {
    pub evolution_chain_url: Option<String>,
    pub flavor_text: Option<String>,
    pub generation: StarryPokemonGeneration,
    pub evolution_data: Vec<StarryEvolutionData>,
}


/// Pokémon generation
#[derive(Archive, Serialize, Deserialize, Default)]
#[rkyv(derive(Debug))]
pub enum StarryPokemonGeneration {
    #[default]
    Unknown,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl std::fmt::Display for StarryPokemonGeneration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            StarryPokemonGeneration::Unknown => write!(f, "{}", fl!("unknown")),
            StarryPokemonGeneration::One => write!(f, "{}", fl!("gen-i")),
            StarryPokemonGeneration::Two => write!(f, "{}", fl!("gen-ii")),
            StarryPokemonGeneration::Three => write!(f, "{}", fl!("gen-iii")),
            StarryPokemonGeneration::Four => write!(f, "{}", fl!("gen-iv")),
            StarryPokemonGeneration::Five => write!(f, "{}", fl!("gen-v")),
            StarryPokemonGeneration::Six => write!(f, "{}", fl!("gen-vi")),
            StarryPokemonGeneration::Seven => write!(f, "{}", fl!("gen-vii")),
            StarryPokemonGeneration::Eight => write!(f, "{}", fl!("gen-viii")),
            StarryPokemonGeneration::Nine => write!(f, "{}", fl!("gen-ix")),
        }
    }
}

impl StarryPokemonGeneration {
    /// Parses a generation name to the StarryPokemonGeneration enum
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "generation-i" => StarryPokemonGeneration::One,
            "generation-ii" => StarryPokemonGeneration::Two,
            "generation-iii" => StarryPokemonGeneration::Three,
            "generation-iv" => StarryPokemonGeneration::Four,
            "generation-v" => StarryPokemonGeneration::Five,
            "generation-vi" => StarryPokemonGeneration::Six,
            "generation-vii" => StarryPokemonGeneration::Seven,
            "generation-viii" => StarryPokemonGeneration::Eight,
            "generation-ix" => StarryPokemonGeneration::Nine,
            _ => StarryPokemonGeneration::Unknown,
        }
    }
}

/// Pokémon evolution data
#[derive(Archive, CheckBytes, Serialize, Deserialize, Debug)]
#[rkyv(derive(Debug))]
pub struct StarryEvolutionData {
    pub id: i64,
    pub name: String,
    pub sprite_path: Option<String>,
    pub needs_to_evolve: Option<String>,
}
