use std::{fs, path::Path, sync::Arc};

use futures::StreamExt;
use rustemon::{
    client::{CACacheManager, RustemonClient, RustemonClientBuilder},
    model::pokemon::Pokemon,
};
use tokio::sync::Semaphore;

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
                sprite_path: if Path::new(&image_path).exists() {
                    Some(image_path)
                } else {
                    None
                },
            })
        }

        result
    }

    pub async fn load_pokemon(&self, pokemon_name: String) -> CustomPokemon {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(pokemon_name.as_str(), &self.client)
            .await
            .unwrap_or_default();

        let image_path = if let Some(front_default_sprite) = &pokemon.sprites.front_default {
            // Create a reqwest client
            let client = reqwest::Client::new();

            // Only download the image if front_default sprite is available
            match download_image(
                &client,
                front_default_sprite.to_string(),
                pokemon.name.to_string(),
            )
            .await
            {
                Ok(()) => Some(format!(
                    "resources/sprites/{}/{}_front.png",
                    pokemon.name, pokemon.name
                )),
                Err(_) => None,
            }
        } else {
            None
        };

        CustomPokemon {
            pokemon: pokemon,
            sprite_path: image_path,
        }
    }

    pub async fn download_all_pokemon_sprites(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let all_entries = rustemon::pokemon::pokemon::get_all_entries(&self.client)
            .await
            .unwrap_or_default();

        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .build()?;

        let semaphore = Arc::new(Semaphore::new(20));

        let results = futures::stream::iter(all_entries)
            .map(|entry| {
                let client = client.clone();
                let semaphore = Arc::clone(&semaphore);
                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let pokemon =
                        rustemon::pokemon::pokemon::get_by_name(&entry.name, &self.client)
                            .await
                            .unwrap_or_default();
                    if let Some(sprite_url) = pokemon.sprites.front_default {
                        download_image(&client, sprite_url, pokemon.name.to_string()).await
                    } else {
                        Ok(())
                    }
                }
            })
            .buffer_unordered(20) // Adjust the number of concurrent tasks
            .collect::<Vec<_>>()
            .await;

        for result in results {
            if let Err(e) = result {
                eprintln!("Error downloading sprite: {}", e);
            }
        }

        Ok(())
    }

    //
    // HELPERS
    //

    pub async fn fix_all_sprites(&self) -> bool {
        let path = Path::new("resources/sprites");

        match tokio::fs::remove_dir_all(path).await {
            Ok(_) => match self.download_all_pokemon_sprites().await {
                Ok(_) => true,
                Err(e) => {
                    eprintln!("Error downloading sprites: {}", e);
                    false
                }
            },
            Err(e) => {
                eprintln!("Error removing directory: {}", e);
                false
            }
        }
    }
}
