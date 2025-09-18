use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anywho::{Error, anywho};
use flate2::Compression;
use flate2::write::GzEncoder;
use futures::StreamExt;
use ron::to_string;
use rustemon::client::{
    CacheMode, CacheOptions, MokaManager, RustemonClient, RustemonClientBuilder,
};
use tokio::sync::Semaphore;

use crate::models::starry_pokemon::{
    StarryEvolutionData, StarryPokemon, StarryPokemonData, StarryPokemonEncounterInfo,
    StarryPokemonGeneration, StarryPokemonSpecie, StarryPokemonStats,
};

mod models;

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
    /// Fetches the details of all Pokémon in PokéApi and parses it to our own data structure.
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

    /// Retrieve a single Pokémon Data from PokéApi and parse it to our own data structure
    async fn fetch_pokemon_details(
        name: &str,
        client: &rustemon::client::RustemonClient,
    ) -> Result<StarryPokemon, Error> {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(name, client).await?;

        let encounter_info =
            rustemon::pokemon::pokemon::encounters::get_by_id(pokemon.id, client).await?;

        let specie_info = rustemon::pokemon::pokemon_species::get_by_name(name, client).await;

        let evolution_info = if let Ok(specie_info) = &specie_info {
            async {
                let url = specie_info
                    .evolution_chain
                    .as_ref()
                    .ok_or_else(|| anywho!("No evolution chain data"))?
                    .url
                    .clone();

                let id: i64 = url
                    .trim_end_matches('/')
                    .split('/')
                    .next_back()
                    .ok_or_else(|| anywho!("Invalid URL format"))?
                    .parse()
                    .map_err(|_| anywho!("Could not parse evolution chain ID"))?;

                if id == 0 {
                    return Err(anywho!("Invalid evolution chain ID"));
                }

                rustemon::evolution::evolution_chain::get_by_id(id, client)
                    .await
                    .map_err(|err| anywho!("{err}"))
            }
            .await
        } else {
            Err(anywho!("Species info not available"))
        };

        let resources_path = Path::new("resources").join("sprites");

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

        // Parse specie info
        let starry_specie_info = if let Ok(specie_info) = specie_info {
            Some(StarryPokemonSpecie {
                evolution_chain_url: specie_info.evolution_chain.as_ref().map(|x| x.url.clone()),
                flavor_text: specie_info
                    .flavor_text_entries
                    .iter()
                    .find(|x| x.language.name == "en")
                    .map(|x| {
                        x.flavor_text
                            .chars()
                            .map(|c| if c.is_control() { ' ' } else { c })
                            .collect::<String>()
                            .split_whitespace()
                            .collect::<Vec<&str>>()
                            .join(" ")
                    }),
                generation: StarryPokemonGeneration::from_name(&specie_info.generation.name),
                evolution_data: if let Ok(evolution_info) = evolution_info {
                    extract_evolution_data_from_chain_link(&evolution_info.chain, &resources_path)
                } else {
                    Vec::new()
                },
            })
        } else {
            None
        };

        Ok(StarryPokemon {
            pokemon: starry_pokemon_data,
            specie: starry_specie_info,
            sprite_path: image_path,
            encounter_info: Some(starry_encounter_info),
        })
    }

    /// Download Pokémon Sprites to the designed folder
    async fn download_all_pokemon_sprites(&self, download_path: &Path) -> Result<(), Error> {
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
                let download_path = download_path.to_path_buf();

                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let pokemon =
                        rustemon::pokemon::pokemon::get_by_name(&entry.name, &self.client).await?;
                    if let Some(sprite_url) = pokemon.sprites.front_default {
                        download_image(&client, sprite_url, pokemon.name.to_string(), download_path)
                            .await
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

/// Attempts to download a pokemon sprite (image_url) to the preconfigured location following the naming scheme of the app
async fn download_image(
    client: &reqwest::Client,
    image_url: String,
    pokemon_name: String,
    download_path: PathBuf,
) -> Result<(), Error> {
    if !download_path.exists() {
        std::fs::create_dir_all(&download_path).expect("Failed to create the resources path");
    }

    let image_filename = format!("{pokemon_name}_front.png");
    let image_path = download_path.join(&pokemon_name).join(&image_filename);

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

/// Extracts the evolution data from a Rustemon ChainLink
fn extract_evolution_data_from_chain_link(
    chain_link: &rustemon::model::evolution::ChainLink,
    resources_path: &std::path::Path,
) -> Vec<StarryEvolutionData> {
    let mut evolution_data = Vec::new();

    let sprite_path = resources_path
        .join(&chain_link.species.name)
        .join(format!("{}_front.png", chain_link.species.name))
        .to_str()
        .map(String::from);

    evolution_data.push(StarryEvolutionData {
        id: chain_link
            .species
            .url
            .trim_end_matches('/')
            .split('/')
            .next_back()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        name: capitalize_string(&chain_link.species.name),
        sprite_path: sprite_path.clone(),
        needs_to_evolve: None, // base form doesn't need requirements
    });

    // add evolved forms
    for evolution in &chain_link.evolves_to {
        let mut evolved_data = extract_evolution_data_from_chain_link(evolution, resources_path);

        // set the evolution requirement for the first Pokémon in this evolution line
        if let Some(first_evolution) = evolved_data.first_mut() {
            first_evolution.needs_to_evolve =
                extract_evolution_requirement(&evolution.evolution_details);
        }

        evolution_data.extend(evolved_data);
    }

    evolution_data
}

/// Extracts evolution requirements from evolution details
fn extract_evolution_requirement(
    evolution_details: &[rustemon::model::evolution::EvolutionDetail],
) -> Option<String> {
    if evolution_details.is_empty() {
        return None;
    }

    let detail = &evolution_details[0];

    // level requirement
    if let Some(min_level) = detail.min_level {
        return Some(format!("Level {min_level}"));
    }

    // item requirement
    if let Some(ref item) = detail.item {
        return Some(capitalize_string(&item.name));
    }

    // held item requirement
    if let Some(ref held_item) = detail.held_item {
        return Some(format!("Holding {}", capitalize_string(&held_item.name)));
    }

    // happiness requirement
    if let Some(min_happiness) = detail.min_happiness {
        return Some(format!("Happiness {min_happiness}"));
    }

    // time of day requirement
    if !detail.time_of_day.is_empty() {
        return Some(format!(
            "During {}",
            capitalize_string(detail.time_of_day.as_str())
        ));
    }

    // location requirement
    if let Some(ref location) = detail.location {
        return Some(format!("At {}", capitalize_string(&location.name)));
    }

    // known move requirement
    if let Some(ref known_move) = detail.known_move {
        return Some(format!("Knowing {}", capitalize_string(&known_move.name)));
    }

    // relative physical stats
    if let Some(relative_physical_stats) = detail.relative_physical_stats {
        match relative_physical_stats {
            1 => return Some("Attack > Defense".to_string()),
            -1 => return Some("Defense > Attack".to_string()),
            0 => return Some("Attack = Defense".to_string()),
            _ => {}
        }
    }

    None
}

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

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        print_help();
        return;
    }

    let flag = &args[1];

    let api_client = StarryApi::default();

    match flag.as_str() {
        "-a" => {
            println!("Executing all operations...");
            download_pokemon_data(&api_client).await;
            download_sprites(&api_client).await;
        }
        "-p" => {
            println!("Downloading Pokémon data only...");
            download_pokemon_data(&api_client).await;
        }
        "-s" => {
            println!("Downloading sprites only...");
            download_sprites(&api_client).await;
        }
        _ => {
            println!("Invalid flag: {}", flag);
            print_help();
        }
    }
}

fn print_help() {
    println!(
        "Usage: {} [FLAG]",
        std::env::args()
            .next()
            .unwrap_or_else(|| "program".to_string())
    );
    println!();
    println!("FLAGS:");
    println!("  -a    Execute all operations (download data and sprites)");
    println!("  -p    Download and generate Pokémon data only");
    println!("  -s    Download and create sprites data only");
    println!();
    println!("You can only pass one flag at a time.");
}

async fn download_pokemon_data(api_client: &StarryApi) {
    println!("Downloading Pokémon Data");

    let data: BTreeMap<i64, StarryPokemon> = api_client.fetch_all_pokemon().await;
    let ron_string = to_string(&data);

    if let Ok(ron_data) = ron_string {
        if let Err(e) = tokio::fs::create_dir_all("assets").await {
            println!("Failed to create assets directory: {}", e);
            return;
        }

        let data_write_res = tokio::fs::write("assets/pokemon_data.ron", ron_data).await;
        if let Ok(_res) = data_write_res {
            println!("Data written successfully");
        } else {
            println!("Failed to write data to file");
        }
    } else {
        println!("Failed to serialize data to RON format");
    }
}

async fn download_sprites(api_client: &StarryApi) {
    let temp_sprites_dir = std::env::temp_dir().join("starry_sprites");

    println!("Downloading Pokémon Sprites");
    let download_images = api_client
        .download_all_pokemon_sprites(&temp_sprites_dir)
        .await;

    if let Ok(_res) = download_images {
        println!(
            "Sprites downloaded successfully to: {:?}",
            &temp_sprites_dir
        );

        if let Err(e) = tokio::fs::create_dir_all("assets").await {
            println!("Failed to create assets directory: {}", e);
            return;
        }

        let assets_path = Path::new("assets").join("sprites.tar.gz");
        let tar_gz = std::fs::File::create(assets_path).unwrap();
        let enc = GzEncoder::new(tar_gz, Compression::default());
        let mut tar = tar::Builder::new(enc);

        // add the entire sprites directory to the archive
        let _res = tar.append_dir_all("sprites", &temp_sprites_dir);
        tar.finish().unwrap();

        // clean up temp directory
        let _res = std::fs::remove_dir_all(&temp_sprites_dir);
        println!("Archive created successfully");
    } else {
        println!("Failed to download sprites");
    }
}
