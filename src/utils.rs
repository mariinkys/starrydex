use std::path::Path;

use rustemon::model::pokemon::Pokemon;

use crate::app::CustomPokemon;

pub async fn load_all_pokemon() -> Vec<CustomPokemon> {
    let client = rustemon::client::RustemonClient::default();
    let all_entries = rustemon::pokemon::pokemon::get_all_entries(&client)
        .await
        .unwrap_or_default();

    let mut result = Vec::<CustomPokemon>::new();

    for entry in all_entries {
        let image_filename = format!("{}_front.png", entry.name);
        let image_path = format!("resources/sprites/{}/{}", entry.name, image_filename);

        result.push(CustomPokemon {
            pokemon: Pokemon {
                name: entry.name,
                ..Default::default()
            },
            sprite_path: Some(image_path),
        })
    }

    result
}

pub async fn load_pokemon(pokemon_name: String) -> CustomPokemon {
    let client = rustemon::client::RustemonClient::default();
    let pokemon = rustemon::pokemon::pokemon::get_by_name(pokemon_name.as_str(), &client)
        .await
        .unwrap_or_default();

    let image_path = if let Some(front_default_sprite) = &pokemon.sprites.front_default {
        // Only download the image if front_default sprite is available
        Some(
            download_image(front_default_sprite.to_string(), pokemon.name.to_string())
                .await
                .unwrap_or_else(|_| String::new()),
        )
    } else {
        None
    };

    CustomPokemon {
        pokemon: pokemon,
        sprite_path: image_path, // Set default if image_path is None
    }
}

pub async fn download_all_pokemon_sprites() {
    let client = rustemon::client::RustemonClient::default();
    let all_entries = rustemon::pokemon::pokemon::get_all_entries(&client)
        .await
        .unwrap_or_default();

    for entry in all_entries {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(&entry.name, &client)
            .await
            .unwrap_or_default();

        let _ = download_image(
            pokemon.sprites.front_default.unwrap_or_default(),
            pokemon.name.to_string(),
        )
        .await;
    }
}

pub async fn fix_all_sprites() -> bool {
    let path = Path::new("resources/sprites");
    let remove_operation = std::fs::remove_dir_all(path);
    match remove_operation {
        Ok(_) => {
            download_all_pokemon_sprites().await;
            true
        }
        Err(_) => false,
    }
}

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

pub async fn download_image(
    image_url: String,
    pokemon_name: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let image_filename = format!("{}_front.png", pokemon_name);
    let image_path = format!("resources/sprites/{}/{}", pokemon_name, image_filename);

    // file already downloaded?
    if tokio::fs::metadata(&image_path).await.is_ok() {
        return Ok(image_path);
    }

    let response = reqwest::get(&image_url).await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;

        let path = std::path::PathBuf::from(&image_path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        save_image(&image_path, &bytes).await?;
        Ok(image_path)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download image. Status: {}", response.status()),
        )))
    }
}

async fn save_image(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = tokio::fs::File::create(path).await?;
    tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
    Ok(())
}
