use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// Simple owned data structure, for displaying the Pokémon in the list page (main page)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonInfo {
    pub id: i64,
    pub name: String,
    pub sprite_path: Option<String>,
}
