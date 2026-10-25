// SPDX-License-Identifier: GPL-3.0-only

//! Owns the memory-mapped Pokédex and answers every data question the UI asks.
//!
//! On first start (or after an update/cache reset) the bundled archive is decompressed to disk
//! and the sprite/cry archives are extracted next to it. After that, startup is just an `mmap`
//! plus a one-time validation pass: Pokémon data is read straight from the mapped file and only
//! the pages that are actually touched end up in memory.

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
};

use anywho::{Error, anywho};
use memmap2::{Mmap, MmapOptions};
use rkyv::rancor;
use starrydex_model::{
    ArchivedPokedex, ArchivedPokemon, ArchivedSpecies, FORMAT_VERSION, Lang, SpriteKind,
};

use crate::app::utils::filters::{Category, Filters, SortBy};
use crate::config::TypeFilteringMode;

pub const APP_ID: &str = "dev.mariinkys.StarryDex";
/// Bump to force re-extraction of the bundled data on the users' machines.
pub const CACHE_VERSION: u32 = 12;

const BUNDLED_POKEDEX: &[u8] = include_bytes!("../../assets/pokedex.bin.gz");
const BUNDLED_SPRITES: &[u8] = include_bytes!("../../assets/sprites.tar.gz");
const BUNDLED_CRIES: &[u8] = include_bytes!("../../assets/cries.tar.gz");

/// Cheap-to-clone handle to the loaded Pokédex.
#[derive(Clone)]
pub struct StarryCore {
    inner: Arc<Inner>,
}

struct Inner {
    /// The archive. Validated once in [`StarryCore::open`]; never written to afterwards.
    mmap: Mmap,
    /// Directory containing the extracted `sprites/` and `cries/` folders.
    assets_dir: PathBuf,
}

impl std::fmt::Debug for StarryCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StarryCore")
            .field("bytes", &self.inner.mmap.len())
            .finish()
    }
}

/// Where everything lives on disk.
struct Paths {
    root: PathBuf,
    data_file: PathBuf,
    assets_dir: PathBuf,
    /// Written once the assets are completely extracted.
    assets_marker: PathBuf,
}

impl Paths {
    fn new() -> Result<Self, Error> {
        let root = dirs::data_dir()
            .ok_or_else(|| anywho!("Could not determine the user data directory"))?
            .join(APP_ID);
        let assets_dir = root.join(format!("assets_v{CACHE_VERSION}"));
        Ok(Self {
            data_file: root
                .join("cache")
                .join(format!("v{CACHE_VERSION}"))
                .join("pokedex.bin"),
            assets_marker: assets_dir.join(".complete"),
            assets_dir,
            root,
        })
    }
}

impl StarryCore {
    /// Loads the Pokédex, installing the bundled data first if needed.
    pub async fn initialize() -> Result<Self, Error> {
        tokio::task::spawn_blocking(Self::load)
            .await
            .map_err(|e| anywho!("Loading task failed: {e}"))?
    }

    fn load() -> Result<Self, Error> {
        let paths = Paths::new()?;
        if !paths.data_file.exists() || !paths.assets_marker.exists() {
            Self::install(&paths)?;
        }
        match Self::open(&paths) {
            Ok(core) => Ok(core),
            Err(e) => {
                // Corrupted or outdated file on disk: reinstall once and retry.
                eprintln!("Cached Pokédex unusable ({e}), reinstalling");
                Self::install(&paths)?;
                Self::open(&paths)
            }
        }
    }

    /// Writes the bundled archive and extracts the bundled assets.
    fn install(paths: &Paths) -> Result<(), Error> {
        println!("Installing bundled Pokédex data");
        Self::remove_stale_versions(&paths.root);

        if let Some(parent) = paths.data_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut bytes = Vec::new();
        flate2::read::GzDecoder::new(BUNDLED_POKEDEX).read_to_end(&mut bytes)?;
        // Write to a temporary file first so a crash never leaves a truncated archive behind.
        let tmp = paths.data_file.with_extension("tmp");
        {
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        std::fs::rename(&tmp, &paths.data_file)?;

        let _ = std::fs::remove_dir_all(&paths.assets_dir);
        std::fs::create_dir_all(&paths.assets_dir)?;
        for archive in [BUNDLED_SPRITES, BUNDLED_CRIES] {
            tar::Archive::new(flate2::read::GzDecoder::new(archive)).unpack(&paths.assets_dir)?;
        }
        std::fs::write(&paths.assets_marker, CACHE_VERSION.to_string())?;
        Ok(())
    }

    /// Deletes data left behind by previous versions of the app.
    fn remove_stale_versions(root: &Path) {
        let current_cache = format!("v{CACHE_VERSION}");
        let current_assets = format!("assets_v{CACHE_VERSION}");
        if let Ok(entries) = std::fs::read_dir(root.join("cache")) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy() != current_cache {
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                let stale = (name.starts_with("resources_v")
                    || name.starts_with("cries_v")
                    || name.starts_with("assets_v"))
                    && name != current_assets;
                if stale {
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
    }

    fn open(paths: &Paths) -> Result<Self, Error> {
        let file = std::fs::File::open(&paths.data_file)?;
        // SAFETY: the file is private to the app and only ever replaced atomically (rename),
        // never modified in place, so the mapped bytes cannot change and this is SAFE (I thnik).
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        let dex = rkyv::access::<ArchivedPokedex, rancor::Error>(&mmap[..])
            .map_err(|e| anywho!("Invalid Pokédex archive: {e}"))?;
        if dex.format_version.to_native() != FORMAT_VERSION {
            return Err(anywho!(
                "Pokédex format {} does not match the expected {FORMAT_VERSION}",
                dex.format_version.to_native()
            ));
        }
        println!("Loaded {} Pokémon", dex.pokemon.len());
        Ok(Self {
            inner: Arc::new(Inner {
                mmap,
                assets_dir: paths.assets_dir.clone(),
            }),
        })
    }

    /// The archived Pokédex, read in place from the memory map.
    pub fn dex(&self) -> &ArchivedPokedex {
        // SAFETY: the bytes were fully validated in `open` and are immutable for the lifetime of
        // `self.inner`, which the returned reference borrows from, so again this is SAFE, I think.
        unsafe { rkyv::access_unchecked::<ArchivedPokedex>(&self.inner.mmap[..]) }
    }

    /// Absolute path of a file inside the extracted assets.
    pub fn asset(&self, relative: &str) -> PathBuf {
        self.inner.assets_dir.join(relative)
    }

    /// Absolute path of a Pokémon sprite, falling back to the front sprite.
    pub fn sprite(&self, pokemon: &ArchivedPokemon, kind: SpriteKind) -> Option<PathBuf> {
        pokemon
            .sprite_path(kind)
            .or_else(|| pokemon.sprite_path(SpriteKind::Front))
            .map(|rel| self.asset(&rel))
    }

    /// Runs search, filters and sorting over every Pokémon. Returns indices into [`ArchivedPokedex::pokemon`].
    pub fn query(&self, query: &ListQuery) -> Vec<u32> {
        let dex = self.dex();
        let needle = fold(query.search.trim());
        let number: Option<u32> = needle.trim_start_matches('#').parse().ok();

        let mut results: Vec<u32> = dex
            .pokemon
            .iter()
            .enumerate()
            .filter(|(_, pokemon)| {
                let species = dex.species_of(pokemon);
                // Alternate forms are hidden unless asked for, but searches always find them.
                if !pokemon.is_default && !query.show_forms && needle.is_empty() {
                    return false;
                }
                if !needle.is_empty() && !matches_search(pokemon, &needle, number, query.lang) {
                    return false;
                }
                matches_filters(pokemon, species, query.filters, query.type_mode)
            })
            .map(|(index, _)| index as u32)
            .collect();

        let pokemon = &dex.pokemon;
        match query.sort {
            SortBy::Number => results.sort_by_key(|&i| {
                let p = &pokemon[i as usize];
                (p.species_id.to_native(), !p.is_default, p.id.to_native())
            }),
            SortBy::Name => {
                results.sort_by_cached_key(|&i| fold(pokemon[i as usize].display.get(query.lang)))
            }
            other => results.sort_by_cached_key(|&i| {
                let p = &pokemon[i as usize];
                let value = match other {
                    SortBy::Total => p.stats.total(),
                    SortBy::Hp => u16::from(p.stats.hp),
                    SortBy::Attack => u16::from(p.stats.attack),
                    SortBy::Defense => u16::from(p.stats.defense),
                    SortBy::SpAttack => u16::from(p.stats.sp_attack),
                    SortBy::SpDefense => u16::from(p.stats.sp_defense),
                    SortBy::Speed => u16::from(p.stats.speed),
                    SortBy::Height => p.height.to_native(),
                    SortBy::Weight => p.weight.to_native(),
                    SortBy::Number | SortBy::Name => 0,
                };
                // Stats sort high → low by default; ties keep dex order.
                (
                    std::cmp::Reverse(value),
                    p.species_id.to_native(),
                    p.id.to_native(),
                )
            }),
        }
        if query.descending {
            results.reverse();
        }
        results
    }
}

/// Everything that decides which Pokémon the list shows, and in which order.
pub struct ListQuery<'a> {
    pub search: &'a str,
    pub filters: &'a Filters,
    pub type_mode: TypeFilteringMode,
    pub show_forms: bool,
    pub sort: SortBy,
    pub descending: bool,
    pub lang: Lang,
}

fn matches_search(
    pokemon: &ArchivedPokemon,
    needle: &str,
    number: Option<u32>,
    lang: Lang,
) -> bool {
    if let Some(number) = number {
        return pokemon.species_id.to_native() == number || pokemon.id.to_native() == number;
    }
    fold(pokemon.display.get(lang)).contains(needle)
        || fold(&pokemon.display.en).contains(needle)
        || pokemon.name.as_str().replace('-', " ").contains(needle)
}

fn matches_filters(
    pokemon: &ArchivedPokemon,
    species: Option<&ArchivedSpecies>,
    filters: &Filters,
    type_mode: TypeFilteringMode,
) -> bool {
    if !filters.types.is_empty() {
        let ok = match type_mode {
            TypeFilteringMode::Inclusive => filters.types.iter().any(|t| pokemon.has_type(*t)),
            TypeFilteringMode::Exclusive => filters.types.iter().all(|t| pokemon.has_type(*t)),
        };
        if !ok {
            return false;
        }
    }
    if filters.stats_enabled && pokemon.stats.total() < filters.min_total {
        return false;
    }
    if !filters.generations.is_empty()
        && !species.is_some_and(|s| filters.generations.contains(&s.generation))
    {
        return false;
    }
    if !filters.categories.is_empty() {
        let category = species.map_or(Category::Regular, |s| {
            if s.is_mythical {
                Category::Mythical
            } else if s.is_legendary {
                Category::Legendary
            } else if s.is_baby {
                Category::Baby
            } else {
                Category::Regular
            }
        });
        if !filters.categories.contains(&category) {
            return false;
        }
    }
    true
}

/// Lowercases and strips the most common diacritics so "flabebe" finds "Flabébé".
pub fn fold(text: &str) -> String {
    text.chars()
        .flat_map(char::to_lowercase)
        .filter_map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' => Some('a'),
            'é' | 'è' | 'ê' | 'ë' => Some('e'),
            'í' | 'ì' | 'î' | 'ï' => Some('i'),
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => Some('o'),
            'ú' | 'ù' | 'û' | 'ü' => Some('u'),
            'ñ' => Some('n'),
            'ç' => Some('c'),
            '♀' | '♂' | '’' | '\'' | '.' => None,
            '-' | ':' => Some(' '),
            other => Some(other),
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use starrydex_model::PokemonType;

    /// Installs the real bundled assets into a temporary data dir and queries them.
    #[test]
    fn loads_and_queries_bundled_pokedex() {
        let tmp = std::env::temp_dir().join(format!("starrydex-test-{}", std::process::id()));
        // SAFETY: set before any other thread reads the environment in this test binary.
        unsafe { std::env::set_var("XDG_DATA_HOME", &tmp) };
        let core = StarryCore::load().expect("bundled data loads");
        let dex = core.dex();
        assert!(dex.pokemon.len() > 1300);
        assert!(dex.species.len() >= 1025);

        let filters = Filters::default();
        let base = ListQuery {
            search: "",
            filters: &filters,
            type_mode: TypeFilteringMode::Exclusive,
            show_forms: false,
            sort: SortBy::Number,
            descending: false,
            lang: Lang::En,
        };
        let all = core.query(&base);
        assert_eq!(
            all.len(),
            dex.species.len(),
            "one entry per species without forms"
        );
        assert_eq!(dex.pokemon[all[0] as usize].name.as_str(), "bulbasaur");

        let pika = core.query(&ListQuery {
            search: "pika",
            ..base
        });
        assert!(
            pika.iter()
                .any(|&i| dex.pokemon[i as usize].name.as_str() == "pikachu")
        );
        let by_number = core.query(&ListQuery {
            search: "#25",
            ..base
        });
        assert_eq!(dex.pokemon[by_number[0] as usize].name.as_str(), "pikachu");
        let accents = core.query(&ListQuery {
            search: "flabebe",
            ..base
        });
        assert_eq!(accents.len(), 1);

        let mut fire_flying = Filters::default();
        fire_flying.types.insert(PokemonType::Fire);
        fire_flying.types.insert(PokemonType::Flying);
        let found = core.query(&ListQuery {
            filters: &fire_flying,
            ..base
        });
        assert!(
            found
                .iter()
                .any(|&i| dex.pokemon[i as usize].name.as_str() == "charizard")
        );
        assert!(found.iter().all(|&i| {
            let p = &dex.pokemon[i as usize];
            p.has_type(PokemonType::Fire) && p.has_type(PokemonType::Flying)
        }));

        let strongest = core.query(&ListQuery {
            sort: SortBy::Total,
            ..base
        });
        let top = &dex.pokemon[strongest[0] as usize];
        assert!(
            top.stats.total() >= 720,
            "{} has {}",
            top.name.as_str(),
            top.stats.total()
        );

        // Assets referenced by the data exist on disk.
        let pikachu = dex.pokemon(25).unwrap();
        assert!(core.sprite(pikachu, SpriteKind::Front).unwrap().exists());
        assert!(
            core.asset(&pikachu.cry_path(starrydex_model::CryKind::Latest).unwrap())
                .exists()
        );

        // Evolution data, including branching chains and conditions.
        let eevee = dex.species(133).unwrap();
        let chain = dex
            .evolution_chain(eevee.evolution_chain_id.as_ref().unwrap().to_native())
            .unwrap();
        assert_eq!(chain.nodes.len(), 9);
        let bulba = dex
            .evolution_chain(
                dex.species(1)
                    .unwrap()
                    .evolution_chain_id
                    .as_ref()
                    .unwrap()
                    .to_native(),
            )
            .unwrap();
        let ivysaur = &bulba.nodes[1];
        let text =
            crate::app::utils::presentation::describe_evolution(dex, &ivysaur.methods[0], Lang::En);
        assert!(text.contains("16"), "{text}");

        // Type chart: Electric does nothing to Ground, Water is weak to Electric.
        assert_eq!(
            dex.effectiveness(PokemonType::Electric, &[PokemonType::Ground]),
            0.0
        );
        assert_eq!(
            dex.effectiveness(
                PokemonType::Electric,
                &[PokemonType::Water, PokemonType::Flying]
            ),
            4.0
        );

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn folds_names() {
        assert_eq!(fold("Flabébé"), "flabebe");
        assert_eq!(fold("Mr. Mime"), "mr mime");
        assert_eq!(fold("Nidoran♀"), "nidoran");
        assert_eq!(fold("Farfetch’d"), "farfetchd");
    }
}
