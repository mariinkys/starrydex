use crate::fl;

/// Allows us to show translated names for pokemon types while keeping the app working (we depend on concats with the original english name)
/// Represents a Pokemon type
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct PokemonType {
    pub display_name: String,
    pub name: String,
}

impl PokemonType {
    pub fn get_all() -> Vec<PokemonType> {
        vec![
            PokemonType {
                display_name: fl!("normal"),
                name: String::from("normal"),
            },
            PokemonType {
                display_name: fl!("fire"),
                name: String::from("fire"),
            },
            PokemonType {
                display_name: fl!("water"),
                name: String::from("water"),
            },
            PokemonType {
                display_name: fl!("electric"),
                name: String::from("electric"),
            },
            PokemonType {
                display_name: fl!("grass"),
                name: String::from("grass"),
            },
            PokemonType {
                display_name: fl!("ice"),
                name: String::from("ice"),
            },
            PokemonType {
                display_name: fl!("fighting"),
                name: String::from("fighting"),
            },
            PokemonType {
                display_name: fl!("poison"),
                name: String::from("poison"),
            },
            PokemonType {
                display_name: fl!("ground"),
                name: String::from("ground"),
            },
            PokemonType {
                display_name: fl!("flying"),
                name: String::from("flying"),
            },
            PokemonType {
                display_name: fl!("psychic"),
                name: String::from("psychic"),
            },
            PokemonType {
                display_name: fl!("bug"),
                name: String::from("bug"),
            },
            PokemonType {
                display_name: fl!("rock"),
                name: String::from("rock"),
            },
            PokemonType {
                display_name: fl!("ghost"),
                name: String::from("ghost"),
            },
            PokemonType {
                display_name: fl!("dragon"),
                name: String::from("dragon"),
            },
            PokemonType {
                display_name: fl!("dark"),
                name: String::from("dark"),
            },
            PokemonType {
                display_name: fl!("steel"),
                name: String::from("steel"),
            },
            PokemonType {
                display_name: fl!("fairy"),
                name: String::from("fairy"),
            },
        ]
    }
}
