// SPDX-License-Identifier: GPL-3.0-only

use std::fmt::Debug;

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

use crate::fl;

/// Main Pokemon structure with all the info we want to display about it
#[derive(Archive, CheckBytes, Serialize, Deserialize, serde::Serialize, serde::Deserialize)]
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
#[derive(Archive, CheckBytes, Serialize, Deserialize, serde::Serialize, serde::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonData {
    pub id: i64,
    pub name: String,
    pub weight: i64,
    pub height: i64,
    pub types: Vec<StarryPokemonType>,
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

/// Possible Pokémon Types
#[derive(
    Archive,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug))]
pub enum StarryPokemonType {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl std::fmt::Display for StarryPokemonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            StarryPokemonType::Normal => write!(f, "{}", fl!("normal")),
            StarryPokemonType::Fire => write!(f, "{}", fl!("fire")),
            StarryPokemonType::Water => write!(f, "{}", fl!("water")),
            StarryPokemonType::Electric => write!(f, "{}", fl!("electric")),
            StarryPokemonType::Grass => write!(f, "{}", fl!("grass")),
            StarryPokemonType::Ice => write!(f, "{}", fl!("ice")),
            StarryPokemonType::Fighting => write!(f, "{}", fl!("fighting")),
            StarryPokemonType::Poison => write!(f, "{}", fl!("poison")),
            StarryPokemonType::Ground => write!(f, "{}", fl!("ground")),
            StarryPokemonType::Flying => write!(f, "{}", fl!("flying")),
            StarryPokemonType::Psychic => write!(f, "{}", fl!("psychic")),
            StarryPokemonType::Bug => write!(f, "{}", fl!("bug")),
            StarryPokemonType::Rock => write!(f, "{}", fl!("rock")),
            StarryPokemonType::Ghost => write!(f, "{}", fl!("ghost")),
            StarryPokemonType::Dragon => write!(f, "{}", fl!("dragon")),
            StarryPokemonType::Dark => write!(f, "{}", fl!("dark")),
            StarryPokemonType::Steel => write!(f, "{}", fl!("steel")),
            StarryPokemonType::Fairy => write!(f, "{}", fl!("fairy")),
        }
    }
}

impl StarryPokemonType {
    /// List of all Pokémon Types
    pub const ALL: &'static [Self] = &[
        Self::Normal,
        Self::Fire,
        Self::Water,
        Self::Electric,
        Self::Grass,
        Self::Ice,
        Self::Fighting,
        Self::Poison,
        Self::Ground,
        Self::Flying,
        Self::Psychic,
        Self::Bug,
        Self::Rock,
        Self::Ghost,
        Self::Dragon,
        Self::Dark,
        Self::Steel,
        Self::Fairy,
    ];

    pub fn icon_name(&self) -> String {
        let name = match &self {
            StarryPokemonType::Normal => "type-normal",
            StarryPokemonType::Fire => "type-fire",
            StarryPokemonType::Water => "type-water",
            StarryPokemonType::Electric => "type-electric",
            StarryPokemonType::Grass => "type-grass",
            StarryPokemonType::Ice => "type-ice",
            StarryPokemonType::Fighting => "type-fighting",
            StarryPokemonType::Poison => "type-poison",
            StarryPokemonType::Ground => "type-ground",
            StarryPokemonType::Flying => "type-flying",
            StarryPokemonType::Psychic => "type-psychic",
            StarryPokemonType::Bug => "type-bug",
            StarryPokemonType::Rock => "type-rock",
            StarryPokemonType::Ghost => "type-ghost",
            StarryPokemonType::Dragon => "type-dragon",
            StarryPokemonType::Dark => "type-dark",
            StarryPokemonType::Steel => "type-steel",
            StarryPokemonType::Fairy => "type-fairy",
        };

        String::from(name)
    }
}

/// Pokémon statistics
#[derive(Archive, CheckBytes, Serialize, Deserialize, serde::Serialize, serde::Deserialize)]
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
#[derive(
    Archive, CheckBytes, Serialize, Deserialize, Clone, serde::Serialize, serde::Deserialize,
)]
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
#[derive(Archive, CheckBytes, Serialize, Deserialize, serde::Serialize, serde::Deserialize)]
#[rkyv(derive(Debug))]
pub struct StarryPokemonSpecie {
    pub evolution_chain_url: Option<String>,
    pub flavor_text: Option<String>,
    pub generation: StarryPokemonGeneration,
    pub evolution_data: Vec<StarryEvolutionData>,
}

/// Pokémon generation
#[derive(
    Archive,
    Serialize,
    Deserialize,
    Default,
    PartialEq,
    Eq,
    Hash,
    Debug,
    Clone,
    serde::Serialize,
    serde::Deserialize,
)]
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
    /// List of all Pokémon Generations
    pub const ALL: &'static [Self] = &[
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Unknown,
    ];
}

/// Pokémon evolution data
#[derive(
    Archive, CheckBytes, Serialize, Deserialize, Debug, serde::Serialize, serde::Deserialize,
)]
#[rkyv(derive(Debug))]
pub struct StarryEvolutionData {
    pub id: i64,
    pub name: String,
    pub sprite_path: Option<String>,
    pub needs_to_evolve: Option<String>,
}
