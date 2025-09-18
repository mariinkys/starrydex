use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PokemonType {
    pub display_name: String,
    pub name: String,
}
