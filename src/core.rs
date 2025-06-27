// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::BTreeMap, io::Write, sync::Arc, time::Duration};

use anywho::{Error, anywho};
use memmap2::{Mmap, MmapOptions};
use rkyv::rancor;
use rustemon::client::{
    CacheMode, CacheOptions, MokaManager, RustemonClient, RustemonClientBuilder,
};
use tokio::sync::Semaphore;

use crate::{
    entities::{PokemonInfo, StarryPokemon, StarryPokemonData, StarryPokemonEncounterInfo},
    utils::{capitalize_string, parse_pokemon_stats},
};
use futures::StreamExt;

const APP_ID: &str = "dev.mariinkys.StarryDex";

type ArchivedStarryPokemonMap = rkyv::Archived<BTreeMap<i64, StarryPokemon>>;

#[derive(Debug, Clone)]
pub struct StarryCore {
    inner: Arc<StarryCoreInner>,
}

#[derive(Debug)]
struct StarryCoreInner {
    // we need to keep the mmap alive
    _mmap: Option<Mmap>,
    // this points to the archived data in the mmap
    pokemon_data: Option<&'static ArchivedStarryPokemonMap>,
    client: StarryApi,
}

impl StarryCore {
    /// Initialize the core by loading data from file or fetching from API
    pub async fn initialize() -> Result<Self, Error> {
        use std::result::Result::Ok;

        let mut inner = StarryCoreInner {
            _mmap: None,
            pokemon_data: None,
            client: StarryApi::default(),
        };

        // try to load from cache first
        match Self::load_from_file() {
            Ok(mmap) => {
                // access the archived data from the mmap
                let archived_data =
                    rkyv::access::<ArchivedStarryPokemonMap, rancor::Error>(&mmap[..])
                        .map_err(|e| anywho!("Failed to access archived data: {}", e))?;

                // extend the lifetime of the archived data to 'static
                // This is safe (I think) because we keep the mmap alive in _mmap field
                let static_data: &'static ArchivedStarryPokemonMap =
                    unsafe { std::mem::transmute(archived_data) };

                inner._mmap = Some(mmap);
                inner.pokemon_data = Some(static_data);
                println!("Loaded {} Pokémon from cache", static_data.len());
            }
            Err(_) => {
                // if loading from cache fails, fetch from API and save to cache
                println!("Cache not found, fetching from API...");
                Self::refresh_data_internal(&mut inner).await?;
            }
        }

        Ok(StarryCore {
            inner: Arc::new(inner),
        })
    }

    /// Refresh data from API and save to cache (this creates a new StarryCore instance because I can't mutate the Arc easily, skill issue)
    #[allow(dead_code)]
    pub async fn refresh_data(&self) -> Result<StarryCore, Error> {
        let pokemon_map = self.inner.client.fetch_all_pokemon().await;
        Self::save_to_file(pokemon_map)?;

        // new instance with the refreshed data
        Self::initialize().await
    }

    async fn refresh_data_internal(inner: &mut StarryCoreInner) -> Result<(), Error> {
        let pokemon_map = inner.client.fetch_all_pokemon().await;
        Self::save_to_file(pokemon_map)?;

        let mmap = Self::load_from_file()?;
        let archived_data = rkyv::access::<ArchivedStarryPokemonMap, rancor::Error>(&mmap[..])
            .map_err(|e| anywho!("Failed to access archived data: {}", e))?;

        // extend the lifetime of the archived data to 'static
        // This is safe (I think) because we keep the mmap alive in _mmap field
        let static_data: &'static ArchivedStarryPokemonMap =
            unsafe { std::mem::transmute(archived_data) };

        inner._mmap = Some(mmap);
        inner.pokemon_data = Some(static_data);

        println!("Downloading Sprites");
        if let Err(e) = inner.client.download_all_pokemon_sprites().await {
            eprintln!("Error downloading sprites: {e}");
        }

        Ok(())
    }

    /// Get all Pokémon (returns an iterator to avoid loading everything into memory)
    #[allow(dead_code)]
    pub fn get_all_pokemon(
        &self,
    ) -> Option<impl Iterator<Item = (i64, &rkyv::Archived<StarryPokemon>)>> {
        self.inner
            .pokemon_data
            .map(|data| data.iter().map(|(id, pokemon)| (id.to_native(), pokemon)))
    }

    /// Get a single Pokémon by ID
    pub fn get_pokemon_by_id(&self, id: i64) -> Option<&rkyv::Archived<StarryPokemon>> {
        self.inner
            .pokemon_data?
            .get(&rkyv::rend::i64_le::from_native(id))
    }

    /// Get Pokémon count
    #[allow(dead_code)]
    pub fn pokemon_count(&self) -> usize {
        self.inner.pokemon_data.map_or(0, |data| data.len())
    }

    /// Check if data is loaded
    #[allow(dead_code)]
    pub fn is_loaded(&self) -> bool {
        self.inner.pokemon_data.is_some()
    }

    /// Get a list of all Pokémon (converts to owned data)
    #[allow(dead_code)]
    pub fn get_pokemon_list(&self) -> Vec<PokemonInfo> {
        if let Some(data) = self.inner.pokemon_data {
            data.iter()
                .map(|(id, pokemon)| PokemonInfo {
                    id: id.to_native(),
                    name: pokemon.pokemon.name.as_str().to_string(),
                    sprite_path: pokemon.sprite_path.as_ref().map(|s| s.as_str().to_string()),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get a subset of Pokémon for pagination
    pub fn get_pokemon_page(&self, offset: usize, limit: usize) -> Vec<PokemonInfo> {
        if let Some(data) = self.inner.pokemon_data {
            let total_count = data.len();

            if total_count == 0 || limit == 0 {
                eprintln!("Either data is empty or limit is 0");
                return Vec::new();
            }

            // Clamp offset to valid range
            let adjusted_offset = std::cmp::min(offset, total_count.saturating_sub(1));
            let actual_limit = std::cmp::min(limit, total_count - adjusted_offset);

            data.iter()
                .skip(offset)
                .take(actual_limit)
                .map(|(id, pokemon)| PokemonInfo {
                    id: id.to_native(),
                    name: pokemon.pokemon.name.as_str().to_string(),
                    sprite_path: pokemon.sprite_path.as_ref().map(|s| s.as_str().to_string()),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Search Pokémon by name
    pub fn search_pokemon(&self, query: &str) -> Vec<PokemonInfo> {
        if let Some(data) = self.inner.pokemon_data {
            let query_lower = query.to_lowercase();
            data.iter()
                .filter(|(_, pokemon)| {
                    pokemon
                        .pokemon
                        .name
                        .as_str()
                        .to_lowercase()
                        .contains(&query_lower)
                })
                .map(|(id, pokemon)| PokemonInfo {
                    id: id.to_native(),
                    name: pokemon.pokemon.name.as_str().to_string(),
                    sprite_path: pokemon.sprite_path.as_ref().map(|s| s.as_str().to_string()),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Filter pokémon by type (inclusive)
    pub fn filter_pokemon_inclusive(
        &self,
        selected_types: &std::collections::HashSet<String>,
    ) -> Vec<PokemonInfo> {
        if let Some(data) = &self.inner.pokemon_data {
            data.iter()
                .filter(|(_, pokemon)| {
                    selected_types.is_empty()
                        || pokemon
                            .pokemon
                            .types
                            .iter()
                            .any(|t| selected_types.contains(&t.to_lowercase()))
                })
                .map(|(id, pokemon)| PokemonInfo {
                    id: id.to_native(),
                    name: pokemon.pokemon.name.as_str().to_string(),
                    sprite_path: pokemon.sprite_path.as_ref().map(|s| s.as_str().to_string()),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Filter pokémon by type (exclusive)
    pub fn filter_pokemon_exclusive(
        &self,
        selected_types: &std::collections::HashSet<String>,
    ) -> Vec<PokemonInfo> {
        if let Some(data) = &self.inner.pokemon_data {
            data.iter()
                .filter(|(_, pokemon)| {
                    selected_types.is_empty()
                        || selected_types.iter().all(|selected_type| {
                            pokemon
                                .pokemon
                                .types
                                .iter()
                                .any(|t| t.to_lowercase() == *selected_type)
                        })
                })
                .map(|(id, pokemon)| PokemonInfo {
                    id: id.to_native(),
                    name: pokemon.pokemon.name.as_str().to_string(),
                    sprite_path: pokemon.sprite_path.as_ref().map(|s| s.as_str().to_string()),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Filter the provided Pokémon list by pokémon that have at least X total_power
    pub fn filter_pokemon_stats_with_list(
        &self,
        pokemon_list: &[PokemonInfo],
        total_power: i64,
    ) -> Vec<PokemonInfo> {
        pokemon_list
            .iter()
            .filter(|pokemon_info| {
                if let Some(data) = &self.inner.pokemon_data {
                    if let Some(archived_pokemon) = data.get(&pokemon_info.id.into()) {
                        if let Ok(pokemon) =
                            rkyv::deserialize::<StarryPokemon, rancor::Error>(archived_pokemon)
                        {
                            pokemon.get_total_stats() >= total_power
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Filter by pokémon that have at least X total_power
    pub fn filter_pokemon_stats(&self, total_power: i64) -> Vec<PokemonInfo> {
        if let Some(data) = &self.inner.pokemon_data {
            data.iter()
                .filter_map(|(id, archived_pokemon)| {
                    if let Ok(pokemon) =
                        rkyv::deserialize::<StarryPokemon, rancor::Error>(archived_pokemon)
                    {
                        if pokemon.get_total_stats() >= total_power {
                            Some(PokemonInfo {
                                id: id.to_native(),
                                name: pokemon.pokemon.name.as_str().to_string(),
                                sprite_path: pokemon
                                    .sprite_path
                                    .as_ref()
                                    .map(|s| s.as_str().to_string()),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn save_to_file(pokemons: BTreeMap<i64, StarryPokemon>) -> Result<(), Error> {
        let cache_dir = dirs::data_dir().unwrap().join(APP_ID);

        std::fs::create_dir_all(&cache_dir)?;

        let cache_path = cache_dir.join("pokemon_cache.bin");

        let bytes = rkyv::to_bytes::<rancor::Error>(&pokemons)
            .map_err(|e| anywho!("Failed to serialize data: {}", e))?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(cache_path)?;

        file.write_all(&bytes)?;
        file.flush()?;

        Ok(())
    }

    fn load_from_file() -> Result<Mmap, Error> {
        let cache_path = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join("pokemon_cache.bin");

        let file = std::fs::File::open(cache_path).map_err(|_| anywho!("Cache file not found"))?;

        let mmap = unsafe { MmapOptions::new().map(&file)? };

        rkyv::access::<ArchivedStarryPokemonMap, rancor::Error>(&mmap[..])
            .map_err(|e| anywho!("Failed to access archived data: {}", e))?;

        Ok(mmap)
    }
}

#[derive(Debug, Clone)]
struct StarryApi {
    client: Arc<RustemonClient>,
}

impl Default for StarryApi {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl StarryApi {
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
            .collect::<Vec<Result<StarryPokemon, Error>>>()
            .await
            .into_iter()
            .filter_map(Result::ok) // keep only the success
            .map(|pokemon| (pokemon.pokemon.id, pokemon))
            .collect()
    }

    /// Retrieve a single Pokémon Data from PokéApi
    async fn fetch_pokemon_details(
        name: &str,
        client: &rustemon::client::RustemonClient,
    ) -> Result<StarryPokemon, Error> {
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

    /// Download Pokémon Sprites to the designed folder
    async fn download_all_pokemon_sprites(&self) -> Result<(), Error> {
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
                        rustemon::pokemon::pokemon::get_by_name(&entry.name, &self.client).await?;
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
                eprintln!("Error downloading sprite: {e}");
            }
        }

        Ok(())
    }
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

    let image_filename = format!("{pokemon_name}_front.png");
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
        Err(anywho!(
            "Failed to download image. Status: {}",
            response.status()
        ))
    }
}
