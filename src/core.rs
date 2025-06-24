// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::BTreeMap, fs::File, sync::Arc, time::Duration};

use anyhow::{Error, anyhow};
use dashmap::DashMap;
use futures::StreamExt;
use memmap2::Mmap;
use rustemon::client::{
    CacheMode, CacheOptions, MokaManager, RustemonClient, RustemonClientBuilder,
};
use tokio::sync::Semaphore;

use crate::{
    entities::{StarryPokemon, StarryPokemonData, StarryPokemonEncounterInfo},
    utils::{StarryError, capitalize_string, parse_pokemon_stats},
};

const APP_ID: &str = "dev.mariinkys.StarryDex";

#[derive(Clone)]
pub struct StarryCore {
    cache: StarryCache,
    api: StarryApi,
}

impl StarryCore {
    pub fn init() -> Self {
        let cache = StarryCache {
            cache: Arc::new(DashMap::new()),
        };

        let api = StarryApi {
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
        };

        StarryCore { cache, api }
    }

    /// Loads all the Pokémon data, tries to load it from cache, if not fetches the data and saves the cache
    pub async fn load_all(&self) -> Arc<DashMap<i64, StarryPokemon>> {
        self.cache.load_or_init(&self.api).await
    }
}

#[derive(Clone)]
struct StarryApi {
    client: Arc<RustemonClient>,
}

impl StarryApi {
    async fn fetch_all_pokemon(&self) -> Arc<DashMap<i64, StarryPokemon>> {
        let all_entries = rustemon::pokemon::pokemon::get_all_entries(&self.client)
            .await
            .unwrap_or_default();

        let semaphore = Arc::new(Semaphore::new(30));
        let map = Arc::new(DashMap::with_capacity(all_entries.len()));

        println!("Downloading Pokémon data...");

        futures::stream::iter(all_entries)
            .for_each_concurrent(50, |entry| {
                let client = self.client.clone();
                let sem = Arc::clone(&semaphore);
                let map = Arc::clone(&map);

                async move {
                    let _permit = sem.acquire().await.unwrap();
                    match Self::fetch_pokemon_details(&entry.name, &client).await {
                        Ok(pokemon) => {
                            map.insert(pokemon.pokemon.id, pokemon);
                        }
                        Err(_) => {
                            eprintln!("Failed to fetch details for: {}", entry.name);
                        }
                    }
                }
            })
            .await;

        map
    }

    async fn fetch_pokemon_details(
        name: &str,
        client: &rustemon::client::RustemonClient,
    ) -> Result<StarryPokemon, rustemon::error::Error> {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(name, client).await?;

        let encounter_info =
            rustemon::pokemon::pokemon::encounters::get_by_id(pokemon.id, client).await?;

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

        Ok(StarryPokemon {
            pokemon: starry_pokemon_data,
            sprite_path: image_path,
            encounter_info: Some(starry_encounter_info),
        })
    }

    async fn download_all_sprites(&self) -> Result<(), Error> {
        let all_entries = rustemon::pokemon::pokemon::get_all_entries(&self.client)
            .await
            .unwrap_or_default();

        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .build()?;

        let semaphore = Arc::new(Semaphore::new(20));

        println!("Downloading sprites...");

        let results = futures::stream::iter(all_entries)
            .map(|entry| {
                let client = client.clone();
                let semaphore = Arc::clone(&semaphore);
                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let pokemon =
                        rustemon::pokemon::pokemon::get_by_name(&entry.name, &self.client).await?;
                    if let Some(sprite_url) = pokemon.sprites.front_default {
                        Self::download_image(&client, sprite_url, pokemon.name.to_string()).await
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

    async fn download_image(
        client: &reqwest::Client,
        image_url: String,
        pokemon_name: String,
    ) -> Result<(), Error> {
        let resources_path = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join("resources")
            .join("sprites");

        if !resources_path.exists() {
            std::fs::create_dir_all(&resources_path).expect("Failed to create the resources path");
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
            Err(anyhow!(
                "Failed to download image. Status: {}",
                response.status()
            ))
        }
    }
}

#[derive(Clone)]
struct StarryCache {
    cache: Arc<DashMap<i64, StarryPokemon>>,
}

impl StarryCache {
    async fn load_or_init(&self, api: &StarryApi) -> Arc<DashMap<i64, StarryPokemon>> {
        if let Err(e) = self.load().await {
            eprintln!("Failed to load cache: {}", e);
        }

        if !self.cache.is_empty() {
            return self.cache.clone(); // Already loaded
        }

        if let Err(e) = api.download_all_sprites().await {
            eprintln!("Error downloading sprites: {}", e);
        }

        let pokemon = api.fetch_all_pokemon().await;
        pokemon.iter().for_each(|entry| {
            self.cache.insert(*entry.key(), entry.value().clone());
        });

        if let Err(e) = self.save_cache().await {
            eprintln!("Failed to save cache: {}", e);
        }

        self.cache.clone()
    }

    async fn load(&self) -> Result<(), StarryError> {
        let path = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join("pokemon_cache.json");
        if !path.exists() {
            return Err(StarryError::NoCacheFound);
        }

        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        let reader = std::io::Cursor::new(&mmap[..]);

        let pokemon_map: BTreeMap<i64, StarryPokemon> = serde_json::from_reader(reader)?;

        for (id, pokemon) in pokemon_map {
            self.cache.insert(id, pokemon);
        }

        Ok(())
    }

    async fn save_cache(&self) -> Result<(), Error> {
        let path = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join("pokemon_cache.json");
        let map_snapshot: BTreeMap<_, _> = self
            .cache
            .iter()
            .map(|entry| (*entry.key(), entry.value().clone()))
            .collect();

        let serialized_data =
            tokio::task::spawn_blocking(move || serde_json::to_string(&map_snapshot)).await??;
        tokio::fs::write(path, serialized_data).await?;

        Ok(())
    }
}
