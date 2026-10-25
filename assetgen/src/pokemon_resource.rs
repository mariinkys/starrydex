//! Lenient model of PokéAPI's `pokemon` resource.

use rustemon::model::{
    pokemon::{
        PokemonAbility, PokemonForm, PokemonHeldItem, PokemonMove, PokemonSpecies, PokemonStat,
        PokemonType, PokemonTypePast,
    },
    resource::{NamedApiResource, VersionGameIndex},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pokemon {
    pub id: i64,
    pub name: String,
    pub base_experience: Option<i64>,
    pub height: i64,
    pub is_default: bool,
    pub order: i64,
    pub weight: i64,
    #[serde(default)]
    pub abilities: Vec<PokemonAbility>,
    #[serde(default)]
    pub forms: Vec<NamedApiResource<PokemonForm>>,
    #[serde(default)]
    pub game_indices: Vec<VersionGameIndex>,
    #[serde(default)]
    pub held_items: Vec<PokemonHeldItem>,
    #[serde(default)]
    pub moves: Vec<PokemonMove>,
    #[serde(default)]
    pub past_types: Vec<PokemonTypePast>,
    #[serde(default)]
    pub sprites: Sprites,
    #[serde(default)]
    pub cries: Cries,
    pub species: NamedApiResource<PokemonSpecies>,
    #[serde(default)]
    pub stats: Vec<PokemonStat>,
    #[serde(default)]
    pub types: Vec<PokemonType>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Sprites {
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
    pub front_female: Option<String>,
    pub front_shiny_female: Option<String>,
    pub back_default: Option<String>,
    pub back_shiny: Option<String>,
    pub back_female: Option<String>,
    pub back_shiny_female: Option<String>,
    pub other: OtherSprites,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct OtherSprites {
    pub home: LargeSprites,
    #[serde(rename = "official-artwork")]
    pub official_artwork: LargeSprites,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct LargeSprites {
    pub front_default: Option<String>,
    pub front_shiny: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Cries {
    pub latest: Option<String>,
    pub legacy: Option<String>,
}
