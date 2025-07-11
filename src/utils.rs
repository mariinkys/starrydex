// SPDX-License-Identifier: GPL-3.0-only

use crate::entities::starry_pokemon::StarryPokemonStats;

/// Transforms a kebab-case string into a space-separated string where each word starts with an uppercase letter.
pub fn capitalize_string(input: &str) -> String {
    let words: Vec<&str> = input.split('-').collect();

    let capitalized_words: Vec<String> = words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            if let Some(first_char) = chars.next() {
                first_char.to_uppercase().collect::<String>() + chars.as_str()
            } else {
                String::new()
            }
        })
        .collect();

    capitalized_words.join(" ")
}

/// Parses the rustemon pokemon stats to the StarryDex ones
pub fn parse_pokemon_stats(stats: &[rustemon::model::pokemon::PokemonStat]) -> StarryPokemonStats {
    let mut starry_stats = StarryPokemonStats {
        hp: 0,
        attack: 0,
        defense: 0,
        sp_attack: 0,
        sp_defense: 0,
        speed: 0,
    };

    for stat in stats {
        match stat.stat.name.as_str() {
            "hp" => starry_stats.hp = stat.base_stat,
            "attack" => starry_stats.attack = stat.base_stat,
            "defense" => starry_stats.defense = stat.base_stat,
            "special-attack" => starry_stats.sp_attack = stat.base_stat,
            "special-defense" => starry_stats.sp_defense = stat.base_stat,
            "speed" => starry_stats.speed = stat.base_stat,
            _ => {} // Ignore any unknown stats
        }
    }

    starry_stats
}

/// Helper to scale some data from PokeApi such as weight...
/// scales a number down by dividing it by 10, converting it to a floating-point
pub fn scale_numbers(num: i64) -> f64 {
    (num as f64) / 10.0
}

/// Attempts to remove the contents of the provided directory path
pub fn remove_dir_contents<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}
