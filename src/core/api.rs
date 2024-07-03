use std::{fs, path::Path, sync::Arc};

use rustemon::{
    client::{CACacheManager, RustemonClient, RustemonClientBuilder},
    model::pokemon::Pokemon,
};

use crate::{app::CustomPokemon, utils::download_image};

#[derive(Debug)]
pub struct Api {
    app_id: String,
    client: Arc<RustemonClient>,
}

impl Clone for Api {
    fn clone(&self) -> Self {
        Api {
            app_id: self.app_id.clone(),
            client: Arc::clone(&self.client),
        }
    }
}

impl Api {
    pub fn new(app_id: &str) -> Api {
        let cache_path = dirs::data_dir().unwrap().join(app_id).join("api_cache");

        if !cache_path.exists() {
            fs::create_dir_all(&cache_path).expect("Failed to create the cache path");
        }

        Api {
            client: Arc::new(
                RustemonClientBuilder::default()
                    .with_manager(CACacheManager { path: cache_path })
                    .try_build()
                    .unwrap(),
            ),
            app_id: app_id.to_string(),
        }
    }

    //
    // API
    //

    pub async fn load_all_pokemon(&self) -> Vec<CustomPokemon> {
        let all_entries = rustemon::pokemon::pokemon::get_all_entries(&self.client)
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

    pub async fn load_pokemon(&self, pokemon_name: String) -> CustomPokemon {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(pokemon_name.as_str(), &self.client)
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

    pub async fn download_all_pokemon_sprites(&self) {
        let all_entries = rustemon::pokemon::pokemon::get_all_entries(&self.client)
            .await
            .unwrap_or_default();

        for entry in all_entries {
            let pokemon = rustemon::pokemon::pokemon::get_by_name(&entry.name, &self.client)
                .await
                .unwrap_or_default();

            let _ = download_image(
                pokemon.sprites.front_default.unwrap_or_default(),
                pokemon.name.to_string(),
            )
            .await;
        }
    }

    //
    // HELPERS
    //

    pub async fn fix_all_sprites(&self) -> bool {
        let path = Path::new("resources/sprites");
        let remove_operation = std::fs::remove_dir_all(path);
        match remove_operation {
            Ok(_) => {
                self.download_all_pokemon_sprites().await;
                true
            }
            Err(_) => false,
        }
    }
}
