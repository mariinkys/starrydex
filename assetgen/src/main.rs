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
    StarryMoveDetails, StarryMoves, StarryPokemon, StarryPokemonData, StarryPokemonEncounterInfo,
    StarryPokemonSpecie, StarryPokemonStats, StarryPokemonType,
};

mod models;
mod utils;

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

        let semaphore = Arc::new(Semaphore::new(5));

        let pokemon_stream = futures::stream::iter(all_entries)
            .map(|entry| {
                let client = Arc::clone(&self.client);
                let sem = Arc::clone(&semaphore);
                async move {
                    let _permit = sem.acquire().await.unwrap();
                    let details_res = Self::fetch_pokemon_details(&entry.name, client).await;
                    if let Err(details_err) = &details_res {
                        eprintln!(
                            "Error downlading details for {}, Error: {}",
                            &entry.name, &details_err
                        );
                    }
                    details_res
                }
            })
            .buffer_unordered(5);

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
        client: Arc<RustemonClient>,
    ) -> Result<StarryPokemon, Error> {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(name, &client).await?;
        let encounter_info =
            rustemon::pokemon::pokemon::encounters::get_by_id(pokemon.id, &client).await?;
        let specie_info = rustemon::pokemon::pokemon_species::get_by_name(name, &client).await;
        let evolution_info = Self::fetch_evolution_info(&specie_info, &client).await;
        let resources_path = Path::new("sprites");

        let sprite_path = if let Some(_front_default_sprite) = &pokemon.sprites.front_default {
            let image_filename = format!("{}_front.png", pokemon.name);
            let full_image_path = resources_path.join(&pokemon.name).join(&image_filename);
            full_image_path.to_str().map(String::from)
        } else {
            None
        };

        let parsed_moves = Self::fetch_moves(pokemon.moves, Arc::clone(&client)).await;

        let starry_pokemon_data = StarryPokemonData {
            id: pokemon.id,
            name: pokemon.name.clone(),
            weight: pokemon.weight,
            height: pokemon.height,
            types: pokemon
                .types
                .iter()
                .map(|t| {
                    StarryPokemonType::try_from(t.type_.name.as_str())
                        .unwrap_or(StarryPokemonType::Normal)
                })
                .collect(),
            abilities: pokemon
                .abilities
                .iter()
                .map(StarryPokemonData::format_ability)
                .collect(),
            stats: StarryPokemonStats::from_stats(&pokemon.stats),
            moves: parsed_moves,
        };

        let starry_encounter_info = encounter_info
            .iter()
            .map(StarryPokemonEncounterInfo::from)
            .collect();

        let starry_specie_info = specie_info
            .ok()
            .map(|s| StarryPokemonSpecie::try_from_specie(s, evolution_info, resources_path))
            .transpose()?;

        Ok(StarryPokemon {
            pokemon: starry_pokemon_data,
            specie: starry_specie_info,
            sprite_path,
            encounter_info: Some(starry_encounter_info),
        })
    }

    /// Fetches and parses the evolution chain for a pokemon species
    async fn fetch_evolution_info(
        specie_info: &Result<rustemon::model::pokemon::PokemonSpecies, rustemon::error::Error>,
        client: &Arc<RustemonClient>,
    ) -> Result<rustemon::model::evolution::EvolutionChain, Error> {
        let specie_info = specie_info.as_ref().map_err(|e| anywho!("{e}"))?;

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
            .map_err(|e| anywho!("{e}"))
    }

    /// Fetches and parses all moves for a pokemon concurrently
    async fn fetch_moves(
        moves: Vec<rustemon::model::pokemon::PokemonMove>,
        client: Arc<RustemonClient>,
    ) -> Vec<StarryMoves> {
        let semaphore = Arc::new(Semaphore::new(5));

        futures::stream::iter(moves)
            .map(|p_move| {
                let client = Arc::clone(&client);
                let sem = Arc::clone(&semaphore);
                async move {
                    let _permit = sem.acquire().await.map_err(|e| anywho!("{e}"))?;

                    let poke_move =
                        rustemon::moves::move_::get_by_name(&p_move.move_.name, &client)
                            .await
                            .map_err(|e| anywho!("{e}"))?;

                    let movement_type = StarryPokemonType::try_from(poke_move.type_.name.as_str())
                        .unwrap_or(StarryPokemonType::Normal);

                    // let primary_vgd = p_move.version_group_details.first().ok_or_else(|| {
                    //     anywho!("No version group details for '{}'", p_move.move_.name)
                    // })?;

                    let move_details: Vec<StarryMoveDetails> = p_move
                        .version_group_details
                        .iter()
                        .map(StarryMoveDetails::from)
                        .collect();

                    Ok::<StarryMoves, Error>(StarryMoves {
                        name: poke_move.name.clone(),
                        movement_type,
                        move_details,
                    })
                }
            })
            .buffer_unordered(5)
            .collect::<Vec<Result<StarryMoves, Error>>>()
            .await
            .into_iter()
            .filter_map(|r| r.map_err(|e| eprintln!("Move fetch error: {e}")).ok())
            .collect()
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
        eprintln!("Error downloading image for Pokémon: {}", &pokemon_name);
        Err(anywho!(
            "Failed to download image. Status: {}",
            response.status()
        ))
    }
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
