use anywho::anywho;
use serde::{Deserialize, Serialize};

use crate::utils::{capitalize_string, clean_flavor_text};

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
    pub types: Vec<StarryPokemonType>,
    pub abilities: Vec<String>,
    pub stats: StarryPokemonStats,
    pub moves: Vec<StarryMoves>,
}

impl StarryPokemonData {
    pub fn format_ability(a: &rustemon::model::pokemon::PokemonAbility) -> String {
        let name = a
            .ability
            .as_ref()
            .map(|ab| ab.name.as_str())
            .unwrap_or("Unknown");

        if a.is_hidden {
            format!("{} (HIDDEN)", name)
        } else {
            name.to_string()
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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

impl TryFrom<&str> for StarryPokemonType {
    type Error = anywho::Error;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        match name.to_lowercase().as_str() {
            "normal" => Ok(Self::Normal),
            "fire" => Ok(Self::Fire),
            "water" => Ok(Self::Water),
            "electric" => Ok(Self::Electric),
            "grass" => Ok(Self::Grass),
            "ice" => Ok(Self::Ice),
            "fighting" => Ok(Self::Fighting),
            "poison" => Ok(Self::Poison),
            "ground" => Ok(Self::Ground),
            "flying" => Ok(Self::Flying),
            "psychic" => Ok(Self::Psychic),
            "bug" => Ok(Self::Bug),
            "rock" => Ok(Self::Rock),
            "ghost" => Ok(Self::Ghost),
            "dragon" => Ok(Self::Dragon),
            "dark" => Ok(Self::Dark),
            "steel" => Ok(Self::Steel),
            "fairy" => Ok(Self::Fairy),
            other => Err(anywho!("Unknown pokemon type: {other}")),
        }
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct StarryPokemonStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub sp_attack: i64,
    pub sp_defense: i64,
    pub speed: i64,
}

impl StarryPokemonStats {
    pub fn from_stats(stats: &[rustemon::model::pokemon::PokemonStat]) -> Self {
        let mut s = Self::default();
        for stat in stats {
            match stat.stat.name.as_str() {
                "hp" => s.hp = stat.base_stat,
                "attack" => s.attack = stat.base_stat,
                "defense" => s.defense = stat.base_stat,
                "special-attack" => s.sp_attack = stat.base_stat,
                "special-defense" => s.sp_defense = stat.base_stat,
                "speed" => s.speed = stat.base_stat,
                _ => {}
            }
        }
        s
    }
}

#[derive(Serialize, Deserialize)]
pub struct StarryPokemonEncounterInfo {
    pub city: String,
    pub games_method: Vec<String>,
}

impl From<&rustemon::model::pokemon::LocationAreaEncounter> for StarryPokemonEncounterInfo {
    fn from(ef: &rustemon::model::pokemon::LocationAreaEncounter) -> Self {
        StarryPokemonEncounterInfo {
            city: capitalize_string(&ef.location_area.name),
            games_method: ef
                .version_details
                .iter()
                .map(|vd| {
                    let unique_methods: std::collections::HashSet<String> = vd
                        .encounter_details
                        .iter()
                        .map(|ed| capitalize_string(&ed.method.name))
                        .collect();
                    format!(
                        "{}: {}",
                        capitalize_string(&vd.version.name),
                        unique_methods.into_iter().collect::<Vec<_>>().join(", ")
                    )
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StarryPokemonSpecie {
    pub evolution_chain_url: Option<String>,
    pub flavor_text: Option<String>,
    pub generation: StarryPokemonGeneration,
    pub evolution_data: Vec<StarryEvolutionData>,
}

impl StarryPokemonSpecie {
    pub fn try_from_specie(
        specie_info: rustemon::model::pokemon::PokemonSpecies,
        evolution_info: Result<rustemon::model::evolution::EvolutionChain, anywho::Error>,
        resources_path: &std::path::Path,
    ) -> Result<Self, anywho::Error> {
        Ok(StarryPokemonSpecie {
            evolution_chain_url: specie_info.evolution_chain.as_ref().map(|x| x.url.clone()),
            flavor_text: specie_info
                .flavor_text_entries
                .iter()
                .find(|x| x.language.name == "en")
                .map(|x| clean_flavor_text(&x.flavor_text)),
            generation: StarryPokemonGeneration::try_from(specie_info.generation.name.as_str())?,
            evolution_data: match evolution_info {
                Ok(chain) => StarryEvolutionData::from_chain_link(&chain.chain, resources_path),
                Err(_) => Vec::new(),
            },
        })
    }
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

impl TryFrom<&str> for StarryPokemonGeneration {
    type Error = anywho::Error;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        match name.to_lowercase().as_str() {
            "generation-i" => Ok(Self::One),
            "generation-ii" => Ok(Self::Two),
            "generation-iii" => Ok(Self::Three),
            "generation-iv" => Ok(Self::Four),
            "generation-v" => Ok(Self::Five),
            "generation-vi" => Ok(Self::Six),
            "generation-vii" => Ok(Self::Seven),
            "generation-viii" => Ok(Self::Eight),
            "generation-ix" => Ok(Self::Nine),
            other => Err(anywho!("Unknown generation: {other}")),
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

impl StarryEvolutionData {
    /// Recursively extracts evolution chain data from a rustemon ChainLink
    pub fn from_chain_link(
        chain_link: &rustemon::model::evolution::ChainLink,
        resources_path: &std::path::Path,
    ) -> Vec<StarryEvolutionData> {
        let mut evolution_data = Vec::new();

        let sprite_path = resources_path
            .join(&chain_link.species.name)
            .join(format!("{}_front.png", chain_link.species.name))
            .to_str()
            .map(String::from);

        evolution_data.push(StarryEvolutionData {
            id: chain_link
                .species
                .url
                .trim_end_matches('/')
                .split('/')
                .next_back()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            name: capitalize_string(&chain_link.species.name),
            sprite_path,
            needs_to_evolve: None,
        });

        for evolution in &chain_link.evolves_to {
            let mut evolved = StarryEvolutionData::from_chain_link(evolution, resources_path);
            if let Some(first) = evolved.first_mut() {
                first.needs_to_evolve =
                    StarryEvolutionData::evolution_requirement(&evolution.evolution_details);
            }
            evolution_data.extend(evolved);
        }

        evolution_data
    }

    /// Extracts a human-readable evolution requirement from evolution details
    fn evolution_requirement(
        details: &[rustemon::model::evolution::EvolutionDetail],
    ) -> Option<String> {
        let detail = details.first()?;

        if let Some(lvl) = detail.min_level {
            return Some(format!("Level {lvl}"));
        }
        if let Some(ref i) = detail.item {
            return Some(capitalize_string(&i.name));
        }
        if let Some(ref h) = detail.held_item {
            return Some(format!("Holding {}", capitalize_string(&h.name)));
        }
        if let Some(hap) = detail.min_happiness {
            return Some(format!("Happiness {hap}"));
        }
        if !detail.time_of_day.is_empty() {
            return Some(format!("During {}", capitalize_string(&detail.time_of_day)));
        }
        if let Some(ref l) = detail.location {
            return Some(format!("At {}", capitalize_string(&l.name)));
        }
        if let Some(ref m) = detail.known_move {
            return Some(format!("Knowing {}", capitalize_string(&m.name)));
        }

        match detail.relative_physical_stats {
            Some(1) => Some("Attack > Defense".to_string()),
            Some(-1) => Some("Defense > Attack".to_string()),
            Some(0) => Some("Attack = Defense".to_string()),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StarryMoves {
    pub name: String,
    pub movement_type: StarryPokemonType,
    pub move_details: Vec<StarryMoveDetails>,
}

#[derive(Serialize, Deserialize)]
pub struct StarryMoveDetails {
    pub game: String,
    pub learned_at: Option<i64>,
    pub learn_method: Vec<StarryMoveLearnMethod>,
}

impl From<&rustemon::model::pokemon::PokemonMoveVersion> for StarryMoveDetails {
    fn from(vgd: &rustemon::model::pokemon::PokemonMoveVersion) -> Self {
        StarryMoveDetails {
            game: vgd.version_group.name.clone(),
            learned_at: (vgd.level_learned_at != 0).then_some(vgd.level_learned_at),
            learn_method: vec![StarryMoveLearnMethod::from(
                vgd.move_learn_method.name.as_str(),
            )],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum StarryMoveLearnMethod {
    LevelUp,
    Tutor,
    TM,
    Egg,
    Unknown,
}

impl From<&str> for StarryMoveLearnMethod {
    fn from(s: &str) -> Self {
        match s {
            "level-up" => StarryMoveLearnMethod::LevelUp,
            "tutor" => StarryMoveLearnMethod::Tutor,
            "machine" => StarryMoveLearnMethod::TM,
            "egg" => StarryMoveLearnMethod::Egg,
            _ => StarryMoveLearnMethod::Unknown,
        }
    }
}
