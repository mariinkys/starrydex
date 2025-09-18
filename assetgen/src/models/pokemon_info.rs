use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PokemonInfo {
    pub id: i64,
    pub name: String,
    pub sprite_path: Option<String>,
}
