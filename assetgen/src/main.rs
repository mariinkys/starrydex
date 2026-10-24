//! StarryDex asset generator.
//!
//! Downloads PokéAPI data and produces the files bundled into the app:
//!
//! * `assets/pokedex.bin.gz` — the whole Pokédex as an rkyv archive
//! * `assets/sprites.tar.gz` — Pokémon sprites (front/back, shiny, female) and item icons
//! * `assets/cries.tar.gz`   — latest and legacy cries

use std::{collections::BTreeSet, io::Write, path::Path, time::Instant};

use flate2::{Compression, write::GzEncoder};
use starrydex_model::{CryKind, Pokedex, SpriteKind};

mod api;
mod assets;
mod dataset;
mod pokemon_resource;
mod text;

struct Options {
    data: bool,
    sprites: bool,
    cries: bool,
    base_url: Option<String>,
    concurrency: usize,
    output: String,
}

fn print_help() {
    let program = std::env::args().next().unwrap_or_else(|| "assetgen".into());
    println!("Usage: {program} [MODE] [OPTIONS]");
    println!();
    println!("MODES (default: -a):");
    println!("  -a    Generate everything (data, sprites and cries)");
    println!("  -p    Generate Pokémon data only (assets/pokedex.bin.gz)");
    println!("  -s    Download sprites only (assets/sprites.tar.gz)");
    println!("  -c    Download cries only (assets/cries.tar.gz)");
    println!();
    println!("OPTIONS:");
    println!("  --base-url <URL>      PokéAPI base url (default: https://pokeapi.co/api/v2/).");
    println!("                        Pointing this to a self-hosted PokéAPI is much faster and");
    println!("                        spares the public servers.");
    println!("  --concurrency <N>     Parallel requests (default: 16)");
    println!("  --output <DIR>        Output directory (default: assets)");
    println!();
    println!("Data, sprites and cries are cross-referenced (the data records which sprites");
    println!("exist), so regenerate them together with -a whenever possible.");
}

fn parse_args() -> Option<Options> {
    let mut options = Options {
        data: true,
        sprites: true,
        cries: true,
        base_url: None,
        concurrency: 16,
        output: "assets".into(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-a" => (options.data, options.sprites, options.cries) = (true, true, true),
            "-p" => (options.data, options.sprites, options.cries) = (true, false, false),
            "-s" => (options.data, options.sprites, options.cries) = (false, true, false),
            "-c" => (options.data, options.sprites, options.cries) = (false, false, true),
            "--base-url" => options.base_url = Some(args.next()?),
            "--concurrency" => options.concurrency = args.next()?.parse().ok()?,
            "--output" => options.output = args.next()?,
            _ => return None,
        }
    }
    Some(options)
}

#[tokio::main]
async fn main() {
    let Some(options) = parse_args() else {
        print_help();
        std::process::exit(2);
    };
    if let Err(e) = run(options).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run(options: Options) -> Result<(), anywho::Error> {
    let started = Instant::now();
    let client = api::client(options.base_url.as_deref())?;
    let raw_api = api::RawApi::new(options.base_url.as_deref())?;
    let output = Path::new(&options.output);

    let raw = dataset::fetch(&client, &raw_api, options.concurrency).await;
    println!("Building Pokédex");
    let (mut dex, plan) = dataset::build(&raw);
    drop(raw);

    let scratch = assets::scratch_dir();
    let mut failed: BTreeSet<String> = BTreeSet::new();
    if options.sprites {
        println!("Downloading sprites");
        failed.extend(
            assets::download_all("sprites", &plan.sprites, &scratch, options.concurrency * 2)
                .await?,
        );
        assets::pack(&scratch, "sprites", &output.join("sprites.tar.gz"))?;
        println!("  Wrote {}", output.join("sprites.tar.gz").display());
    }
    if options.cries {
        println!("Downloading cries");
        failed.extend(
            assets::download_all("cries", &plan.cries, &scratch, options.concurrency * 2).await?,
        );
        assets::pack(&scratch, "cries", &output.join("cries.tar.gz"))?;
        println!("  Wrote {}", output.join("cries.tar.gz").display());
    }

    if options.data {
        drop_missing_assets(&mut dex, &failed);
        write_data(&dex, &output.join("pokedex.bin.gz"))?;
        print_summary(&dex);
    }

    if options.sprites && options.cries {
        let _ = std::fs::remove_dir_all(&scratch);
    }
    println!("Done in {:.0?}", started.elapsed());
    Ok(())
}

/// Clears the sprite/cry flags of files that could not be downloaded, never
/// point at a missing file.
fn drop_missing_assets(dex: &mut Pokedex, failed: &BTreeSet<String>) {
    if failed.is_empty() {
        return;
    }
    for pokemon in &mut dex.pokemon {
        for kind in SpriteKind::ALL {
            if failed.contains(&starrydex_model::sprite_rel_path(pokemon.id, *kind)) {
                pokemon.sprites &= !(*kind as u16);
            }
        }
        for kind in [CryKind::Latest, CryKind::Legacy] {
            if failed.contains(&format!("cries/{}/{}", pokemon.id, kind.file_name())) {
                pokemon.cries &= !(kind as u8);
            }
        }
    }
    for item in &mut dex.items {
        if failed.contains(&starrydex_model::item_sprite_rel_path(&item.name)) {
            item.has_sprite = false;
        }
    }
}

fn write_data(dex: &Pokedex, path: &Path) -> Result<(), anywho::Error> {
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(dex)
        .map_err(|e| anywho::anywho!("Failed to serialize the Pokédex: {e}"))?;

    // make sure we can access the written data (validate)
    rkyv::access::<starrydex_model::ArchivedPokedex, rkyv::rancor::Error>(&bytes)
        .map_err(|e| anywho::anywho!("Serialized Pokédex failed validation: {e}"))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut encoder = GzEncoder::new(std::fs::File::create(path)?, Compression::best());
    encoder.write_all(&bytes)?;
    encoder.finish()?;
    println!(
        "  Wrote {} ({:.1} MiB uncompressed, {:.1} MiB on disk)",
        path.display(),
        bytes.len() as f64 / 1_048_576.0,
        std::fs::metadata(path)?.len() as f64 / 1_048_576.0
    );
    Ok(())
}

fn print_summary(dex: &Pokedex) {
    println!(
        "  {} Pokémon ({} species), {} moves, {} abilities, {} items, {} evolution chains, {} locations",
        dex.pokemon.len(),
        dex.species.len(),
        dex.moves.len(),
        dex.abilities.len(),
        dex.items.len(),
        dex.evolution_chains.len(),
        dex.locations.len()
    );
}
