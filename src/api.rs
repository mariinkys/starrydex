use futures::StreamExt;
use rustemon::client::{
    CacheMode, CacheOptions, MokaManager, RustemonClient, RustemonClientBuilder,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;

use crate::{
    app::{StarryPokemon, StarryPokemonData, StarryPokemonEncounterInfo},
    utils::{capitalize_string, download_image, parse_pokemon_stats},
};

const APP_ID: &str = "dev.mariinkys.StarryDex";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PokemonCache {
    pokemon: BTreeMap<i64, StarryPokemon>,
}

#[derive(Debug)]
pub struct Api {
    app_id: String,
    client: Arc<RustemonClient>,
    cache: Arc<RwLock<Option<PokemonCache>>>,
}

impl Clone for Api {
    fn clone(&self) -> Self {
        Api {
            app_id: self.app_id.clone(),
            client: Arc::clone(&self.client),
            cache: Arc::clone(&self.cache),
        }
    }
}

impl Api {
    pub fn new(app_id: &str) -> Api {
        Api {
            client: Arc::new(
                RustemonClientBuilder::default()
                    .with_manager(MokaManager::default())
                    .with_mode(CacheMode::NoStore)
                    .with_options(CacheOptions {
                        shared: true,
                        cache_heuristic: 0.1,
                        immutable_min_time_to_live: Duration::from_secs(3600),
                        ignore_cargo_cult: true,
                    })
                    .try_build()
                    .unwrap(),
            ),
            cache: Arc::new(RwLock::new(None)),
            app_id: app_id.to_string(),
        }
    }

    /// Attempts to load the data from the cache
    async fn load_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let cache_file = dirs::data_dir()
            .unwrap()
            .join(&self.app_id)
            .join("pokemon_cache.json");

        if cache_file.exists() {
            let cache_data = tokio::fs::read_to_string(cache_file).await?;
            let cache: PokemonCache = serde_json::from_str(&cache_data)?;
            let mut write_guard = self.cache.write().await;
            *write_guard = Some(cache);
        }

        Ok(())
    }

    /// Attempts to save the data to the cache
    async fn save_cache(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cache_file = dirs::data_dir()
            .unwrap()
            .join(&self.app_id)
            .join("pokemon_cache.json");

        println!("Attempting to save cache to: {:?}", cache_file);

        // Retry logic for acquiring the lock
        let cache_data = self.get_cache_data().await?;

        // Perform serialization outside the lock
        let serialized_data =
            tokio::task::spawn_blocking(move || serde_json::to_string(&cache_data)).await??;

        tokio::fs::write(&cache_file, serialized_data).await?;

        println!("Cache successfully saved to: {:?}", cache_file);
        Ok(())
    }

    /// Attempts to get the data from the cache
    async fn get_cache_data(
        &self,
    ) -> Result<PokemonCache, Box<dyn std::error::Error + Send + Sync>> {
        const MAX_RETRIES: u32 = 3;
        const RETRY_DELAY: Duration = Duration::from_millis(100);

        for attempt in 1..=MAX_RETRIES {
            match timeout(Duration::from_secs(1), self.cache.read()).await {
                Ok(guard) => {
                    if let Some(cache) = &*guard {
                        return Ok(cache.clone());
                    } else {
                        return Err("No cache data available".into());
                    }
                }
                Err(_) => {
                    println!(
                        "Timeout while acquiring cache lock, attempt {}/{}",
                        attempt, MAX_RETRIES
                    );
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(RETRY_DELAY).await;
                    }
                }
            }
        }
        Err("Failed to acquire cache lock after multiple attempts".into())
    }

    //
    // API
    //

    /// Retrieve all Pokémon Data from Cache, if the cache does not exist, create the cache
    pub async fn load_all_pokemon(&self) -> BTreeMap<i64, StarryPokemon> {
        println!("Loading Cache");
        self.load_cache()
            .await
            .unwrap_or_else(|e| eprintln!("Failed to load cache: {}", e));

        println!("Reading Cache");
        let read_guard = self.cache.read().await;

        println!("Getting Cache");
        if let Some(cache_data) = &*read_guard {
            println!("Cache Found, returning list");
            return cache_data.pokemon.clone();
        }
        drop(read_guard); // Release the read lock

        println!("No Cache, Downloading Sprites");
        if let Err(e) = self.download_all_pokemon_sprites().await {
            eprintln!("Error downloading sprites: {}", e);
        }

        println!("Fetching Pokemon");
        let pokemon = self.fetch_all_pokemon().await;

        println!("Updating Cache");
        let mut write_guard = self.cache.write().await;
        *write_guard = Some(PokemonCache {
            pokemon: pokemon.clone(),
        });
        drop(write_guard); // Release the write lock

        println!("Save Cache");
        self.save_cache()
            .await
            .unwrap_or_else(|e| eprintln!("Failed to save cache: {}", e));

        println!("Return Pokémon List");
        pokemon
    }

    /// Fetches all Pokémon Data from the PokéApi
    async fn fetch_all_pokemon(&self) -> BTreeMap<i64, StarryPokemon> {
        let all_entries = rustemon::pokemon::pokemon::get_all_entries(&self.client)
            .await
            .unwrap_or_default();

        let semaphore = Arc::new(Semaphore::new(30));

        let pokemon_stream = futures::stream::iter(all_entries)
            .map(|entry| {
                let client = self.client.clone();
                let sem = Arc::clone(&semaphore);
                async move {
                    let _permit = sem.acquire().await.unwrap();
                    Self::fetch_pokemon_details(&entry.name, &client).await
                }
            })
            .buffer_unordered(30);

        pokemon_stream
            .collect::<Vec<StarryPokemon>>()
            .await
            .into_iter()
            .map(|pokemon| (pokemon.pokemon.id, pokemon))
            .collect()
    }

    /// Retrieve a single Pokémon Data from PokéApi
    async fn fetch_pokemon_details(
        name: &str,
        client: &rustemon::client::RustemonClient,
    ) -> StarryPokemon {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(name, client)
            .await
            .unwrap_or_default();

        let encounter_info = rustemon::pokemon::pokemon::encounters::get_by_id(pokemon.id, client)
            .await
            .unwrap_or_default();

        let resources_path = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join("resources")
            .join("sprites");

        let image_path = if let Some(_front_default_sprite) = &pokemon.sprites.front_default {
            let image_filename = format!("{}_front.png", pokemon.name);
            let full_image_path = resources_path.join(&pokemon.name).join(&image_filename);
            full_image_path.to_str().map(String::from)
        } else {
            None
        };

        // Parse Rustemon data to the StarryDex format
        let starry_pokemon_data = StarryPokemonData {
            id: pokemon.id,
            name: pokemon.name,
            weight: pokemon.weight,
            height: pokemon.height,
            types: pokemon
                .types
                .iter()
                .map(|types| types.type_.name.to_string())
                .collect(),
            abilities: pokemon
                .abilities
                .iter()
                .map(|a| {
                    if a.is_hidden {
                        format!("{} (HIDDEN)", a.ability.name)
                    } else {
                        a.ability.name.clone()
                    }
                })
                .collect(),
            stats: parse_pokemon_stats(&pokemon.stats),
        };

        // Parse Rustemon encounter info data to the StarryDex format
        let starry_encounter_info: Vec<StarryPokemonEncounterInfo> = encounter_info
            .iter()
            .map(|ef| StarryPokemonEncounterInfo {
                city: capitalize_string(&ef.location_area.name),
                games_method: ef
                    .version_details
                    .iter()
                    .map(|vd| {
                        // Remove repeated methods
                        let unique_methods: std::collections::HashSet<String> = vd
                            .encounter_details
                            .iter()
                            .map(|ed| capitalize_string(&ed.method.name))
                            .collect();

                        format!(
                            "{}: {}",
                            capitalize_string(&vd.version.name),
                            unique_methods
                                .into_iter()
                                .collect::<Vec<String>>()
                                .join(", ")
                        )
                    })
                    .collect(),
            })
            .collect();

        StarryPokemon {
            pokemon: starry_pokemon_data,
            sprite_path: image_path,
            encounter_info: Some(starry_encounter_info),
        }
    }

    /// Download Pokémon Sprites to the designed folder
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
}
