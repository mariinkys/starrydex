use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StarryPokemon {
    pub pokemon: StarryPokemonData,
    pub specie: Option<StarryPokemonSpecie>,
    pub sprite_path: Option<String>,
    pub encounter_info: Option<Vec<StarryPokemonEncounterInfo>>,
}

#[derive(Serialize, Deserialize)]
pub struct StarryPokemonData {
    pub id: i64,
    pub name: String,
    pub weight: i64,
    pub height: i64,
    pub types: Vec<String>,
    pub abilities: Vec<String>,
    pub stats: StarryPokemonStats,
}

#[derive(Serialize, Deserialize)]
pub struct StarryPokemonStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub sp_attack: i64,
    pub sp_defense: i64,
    pub speed: i64,
}

#[derive(Serialize, Deserialize)]
pub struct StarryPokemonEncounterInfo {
    pub city: String,
    pub games_method: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct StarryPokemonSpecie {
    pub evolution_chain_url: Option<String>,
    pub flavor_text: Option<String>,
    pub generation: StarryPokemonGeneration,
    pub evolution_data: Vec<StarryEvolutionData>,
}

#[derive(Serialize, Deserialize)]
pub enum StarryPokemonGeneration {
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

#[derive(Serialize, Deserialize)]
pub struct StarryEvolutionData {
    pub id: i64,
    pub name: String,
    pub sprite_path: Option<String>,
    pub needs_to_evolve: Option<String>,
}
