// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::BTreeMap, io::Write, sync::Arc};

use anywho::{Error, anywho};
use memmap2::{Mmap, MmapOptions};
use rkyv::rancor;

use crate::entities::{pokemon_info::PokemonInfo, starry_pokemon::StarryPokemon};

/// Unique identifier in RDNN (reverse domain name notation) format.
pub const APP_ID: &str = "dev.mariinkys.StarryDex";
pub const CACHE_VERSION: i32 = 2;

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
}

impl StarryCore {
    /// Initialize the core by loading data from file or fetching from API
    pub async fn initialize() -> Result<Self, Error> {
        use std::result::Result::Ok;

        let mut inner = StarryCoreInner {
            _mmap: None,
            pokemon_data: None,
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
                // if loading from cache fails, parse asset gen files and save to cache
                println!("Cache not found, getting bundled data");
                Self::get_bundled_data(&mut inner).await?;
            }
        }

        Ok(StarryCore {
            inner: Arc::new(inner),
        })
    }

    /// Executed if loading from cache fails, loads the data from the bundled assets
    async fn get_bundled_data(inner: &mut StarryCoreInner) -> Result<(), Error> {
        let pokemon_map = Self::extract_pokemon_data().await;
        if let Err(err) = pokemon_map {
            panic!("Failed to extract bundled Pokémon data with error: {}", err)
        }
        Self::save_to_file(pokemon_map.unwrap())?;

        let mmap = Self::load_from_file()?;
        let archived_data = rkyv::access::<ArchivedStarryPokemonMap, rancor::Error>(&mmap[..])
            .map_err(|e| anywho!("Failed to access archived data: {}", e))?;

        // extend the lifetime of the archived data to 'static
        // This is safe (I think) because we keep the mmap alive in _mmap field
        let static_data: &'static ArchivedStarryPokemonMap =
            unsafe { std::mem::transmute(archived_data) };

        inner._mmap = Some(mmap);
        inner.pokemon_data = Some(static_data);

        println!("Extracting Sprites");
        // we don't join sprites because the archive already has a /sprites folder
        let sprites_directory = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join(format!("resources_v{}", CACHE_VERSION));
        if let Err(e) = Self::extract_sprite_archive(&sprites_directory).await {
            eprintln!("Error downloading sprites: {e}");
        }

        Ok(())
    }

    /// Deserialize Pokémon data in .ron format to a BTreeMap<i64, StarryPokemon>
    async fn extract_pokemon_data() -> Result<BTreeMap<i64, StarryPokemon>, Error> {
        // Bundle sprites as tar.gz and extract
        const POKEMON_DATA: &[u8] = include_bytes!("../assets/pokemon_data.ron");

        let ron_str = std::str::from_utf8(POKEMON_DATA)?;
        let mut pokemon_data: BTreeMap<i64, StarryPokemon> = ron::from_str(ron_str)?;

        let base_sprite_path = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join(format!("resources_v{}", CACHE_VERSION));

        // Modify sprite_path for all pokémon
        pokemon_data = pokemon_data
            .into_iter()
            .map(|(id, mut pokemon)| {
                if let Some(sprite_path) = pokemon.sprite_path {
                    pokemon.sprite_path = std::path::Path::new(&base_sprite_path)
                        .join(sprite_path)
                        .to_str()
                        .map(String::from);
                }

                if let Some(mut specie) = pokemon.specie {
                    specie.evolution_data.iter_mut().for_each(|evo_data| {
                        if let Some(evo_data_sprite_path) = &evo_data.sprite_path {
                            evo_data.sprite_path = std::path::Path::new(&base_sprite_path)
                                .join(evo_data_sprite_path)
                                .to_str()
                                .map(String::from);
                        }
                    });
                    pokemon.specie = Some(specie);
                }

                (id, pokemon)
            })
            .collect();

        Ok(pokemon_data)
    }

    /// Extract sprites archive
    async fn extract_sprite_archive(target_dir: &std::path::Path) -> Result<(), Error> {
        // Bundle sprites as tar.gz and extract
        const BUNDLED_SPRITES: &[u8] = include_bytes!("../assets/sprites.tar.gz");

        // Extract using tar crate
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(BUNDLED_SPRITES));
        archive.unpack(target_dir)?;

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

    /// Filter pokémon by generation
    pub fn filter_pokemon_by_generation(
        &self,
        pokemon_list: &[PokemonInfo],
        selected_generations: &std::collections::HashSet<String>,
    ) -> Vec<PokemonInfo> {
        pokemon_list
            .iter()
            .filter(|pokemon_info| {
                if let Some(data) = &self.inner.pokemon_data {
                    if let Some(archived_pokemon) = data.get(&pokemon_info.id.into()) {
                        if let Ok(pokemon) =
                            rkyv::deserialize::<StarryPokemon, rancor::Error>(archived_pokemon)
                        {
                            if let Some(pokemon_specie) = pokemon.specie {
                                selected_generations
                                    .contains(&pokemon_specie.generation.to_string())
                            } else {
                                false
                            }
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
    #[allow(dead_code)]
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

    /// Attempts to serialize the given data and save it to our cache replacing the old file if exists
    fn save_to_file(pokemons: BTreeMap<i64, StarryPokemon>) -> Result<(), Error> {
        let cache_dir = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join("cache")
            .join(format!("v{}", CACHE_VERSION));

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

    /// Attempts to load the application cache from it's preconfigured location and creates a MMap out of it
    fn load_from_file() -> Result<Mmap, Error> {
        let cache_path = dirs::data_dir()
            .unwrap()
            .join(APP_ID)
            .join("cache")
            .join(format!("v{}", CACHE_VERSION))
            .join("pokemon_cache.bin");

        let file = std::fs::File::open(cache_path).map_err(|_| anywho!("Cache file not found"))?;

        let mmap = unsafe { MmapOptions::new().map(&file)? };

        rkyv::access::<ArchivedStarryPokemonMap, rancor::Error>(&mmap[..])
            .map_err(|e| anywho!("Failed to access archived data: {}", e))?;

        Ok(mmap)
    }
}
