// SPDX-License-Identifier: GPL-3.0-only

use std::fs;

use crate::app::StarryPokemonStats;

const APP_ID: &str = "dev.mariinkys.StarryDex";

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

pub fn scale_numbers(num: i64) -> f64 {
    (num as f64) / 10.0
}

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

pub async fn download_image(
    client: &reqwest::Client,
    image_url: String,
    pokemon_name: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let resources_path = dirs::data_dir()
        .unwrap()
        .join(APP_ID)
        .join("resources")
        .join("sprites");

    if !resources_path.exists() {
        fs::create_dir_all(&resources_path).expect("Failed to create the resources path");
    }

    let image_filename = format!("{}_front.png", pokemon_name);
    let image_path = resources_path.join(&pokemon_name).join(&image_filename);

    // Check if file already exists
    if tokio::fs::metadata(&image_path).await.is_ok() {
        return Ok(());
    }

    let response = client.get(&image_url).send().await?;
    if response.status().is_success() {
        let bytes = response.bytes().await?;
        let path = std::path::PathBuf::from(&image_path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&image_path, &bytes).await?;
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download image. Status: {}", response.status()),
        )))
    }
}

pub fn remove_dir_contents<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}
