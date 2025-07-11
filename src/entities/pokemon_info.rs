use std::fmt::Debug;

/// Simple owned data structure, for displaying the Pokémon in the list page (main page)
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
