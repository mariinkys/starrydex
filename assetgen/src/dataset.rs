//! Fetches every PokéAPI resource StarryDex needs and converts it into the normalized
//! [`starrydex_model::Pokedex`].

use std::collections::{BTreeMap, BTreeSet, HashMap};

use rustemon::model::{
    encounters::{EncounterConditionValue, EncounterMethod},
    evolution::{ChainLink, EvolutionChain, EvolutionDetail, EvolutionTrigger},
    games::{Pokedex, Version, VersionGroup},
    items::Item,
    locations::{Location, LocationArea, Region},
    moves::{Move, MoveTarget},
    pokemon::{
        Ability, LocationAreaEncounter, PokemonColor, PokemonForm, PokemonHabitat, PokemonShape,
        PokemonSpecies, Type,
    },
    resource::NamedApiResource,
};
use starrydex_model::{self as model, CryKind, SpriteKind, Text};

use crate::api::{Client, RawApi, fetch_many, id_of, id16, id32};
use crate::pokemon_resource::Pokemon;
use crate::text::{join, latest_text_from, map_text, or_title, text_from, title_case};

/// Everything downloaded from PokéAPI, keyed by resource id.
#[derive(Default)]
pub struct Raw {
    versions: BTreeMap<i64, Version>,
    version_groups: BTreeMap<i64, VersionGroup>,
    pokedexes: BTreeMap<i64, Pokedex>,
    regions: BTreeMap<i64, Region>,
    encounter_methods: BTreeMap<i64, EncounterMethod>,
    encounter_conditions: BTreeMap<i64, EncounterConditionValue>,
    evolution_triggers: BTreeMap<i64, EvolutionTrigger>,
    types: BTreeMap<i64, Type>,
    move_targets: BTreeMap<i64, MoveTarget>,
    colors: BTreeMap<i64, PokemonColor>,
    shapes: BTreeMap<i64, PokemonShape>,
    habitats: BTreeMap<i64, PokemonHabitat>,
    moves: BTreeMap<i64, Move>,
    machines: BTreeMap<i64, String>,
    abilities: BTreeMap<i64, Ability>,
    pokemon: BTreeMap<i64, Pokemon>,
    encounters: BTreeMap<i64, Vec<LocationAreaEncounter>>,
    species: BTreeMap<i64, PokemonSpecies>,
    /// Pokémon id → its (first) form, only for non-default or renamed varieties.
    forms: BTreeMap<i64, PokemonForm>,
    chains: BTreeMap<i64, EvolutionChain>,
    location_areas: BTreeMap<i64, LocationArea>,
    locations: BTreeMap<i64, Location>,
    items: BTreeMap<i64, Item>,
}

/// A file to download for the sprite/cry archives.
#[derive(Debug, Clone)]
pub struct Download {
    pub url: String,
    /// Path inside the archive, e.g. `sprites/pokemon/25/front.png`.
    pub path: String,
    /// Downscale to the size of regular sprites (used when only HOME/artwork images exist).
    pub resize: bool,
}

/// Everything the asset step needs to download.
#[derive(Default)]
pub struct AssetPlan {
    pub sprites: Vec<Download>,
    pub cries: Vec<Download>,
}

/// Collects the ids out of a "get all entries" response.
fn ids<T>(
    entries: Result<Vec<NamedApiResource<T>>, rustemon::error::Error>,
    label: &str,
) -> Vec<i64> {
    match entries {
        Ok(entries) => entries.iter().filter_map(|e| id_of(&e.url)).collect(),
        Err(e) => {
            eprintln!("  ! could not list {label}: {e}");
            Vec::new()
        }
    }
}

/// Downloads the full dataset.
pub async fn fetch(client: &Client, raw_api: &RawApi, concurrency: usize) -> Raw {
    use rustemon::{encounters, evolution, games, items, locations, machines, moves, pokemon};
    let c = client;
    let n = concurrency;
    let mut raw = Raw::default();

    println!("Fetching lookup tables");
    macro_rules! table {
        ($field:ident, $module:path, $label:literal) => {{
            use $module as module;
            let list = ids(module::get_all_entries(c).await, $label);
            raw.$field = fetch_many($label, list, n, |id| module::get_by_id(id, c)).await;
        }};
    }
    table!(versions, games::version, "versions");
    table!(version_groups, games::version_group, "version groups");
    table!(pokedexes, games::pokedex, "pokédexes");
    table!(regions, locations::region, "regions");
    table!(
        encounter_methods,
        encounters::encounter_method,
        "encounter methods"
    );
    table!(
        encounter_conditions,
        encounters::encounter_condition_value,
        "encounter conditions"
    );
    table!(
        evolution_triggers,
        evolution::evolution_trigger,
        "evolution triggers"
    );
    table!(move_targets, moves::move_target, "move targets");
    table!(colors, pokemon::pokemon_color, "colors");
    table!(shapes, pokemon::pokemon_shape, "shapes");
    table!(habitats, pokemon::pokemon_habitat, "habitats");
    // Only the 18 battle types (ids above 10000 are "shadow"/"unknown"; 19 is "stellar").
    raw.types = fetch_many("types", 1..=18, n, |id| pokemon::type_::get_by_id(id, c)).await;

    println!("Fetching moves & abilities");
    table!(moves, moves::move_, "moves");
    let machine_ids: BTreeSet<i64> = raw
        .moves
        .values()
        .flat_map(|m| m.machines.iter().filter_map(|d| id_of(&d.machine.url)))
        .collect();
    raw.machines = fetch_many("machines", machine_ids, n, |id| {
        machines::machine::get_by_id(id, c)
    })
    .await
    .into_iter()
    .map(|(id, machine)| (id, machine.item.name.to_ascii_uppercase()))
    .collect();
    table!(abilities, pokemon::ability, "abilities");

    println!("Fetching Pokémon");
    let pokemon_ids = ids(pokemon::pokemon::get_all_entries(c).await, "pokémon");
    raw.pokemon = fetch_many("pokémon", pokemon_ids, n, |id| {
        raw_api.get::<Pokemon>("pokemon", id)
    })
    .await;
    raw.encounters = fetch_many("encounter lists", raw.pokemon.keys().copied(), n, |id| {
        pokemon::pokemon::encounters::get_by_id(id, c)
    })
    .await;
    let species_ids: BTreeSet<i64> = raw
        .pokemon
        .values()
        .filter_map(|p| id_of(&p.species.url))
        .collect();
    raw.species = fetch_many("species", species_ids, n, |id| {
        pokemon::pokemon_species::get_by_id(id, c)
    })
    .await;

    // Forms give the proper display names of alternate varieties ("Mega Charizard X").
    let form_ids: BTreeMap<i64, i64> = raw
        .pokemon
        .values()
        .filter(|p| !p.is_default || p.name != p.species.name)
        .filter_map(|p| Some((id_of(&p.forms.first()?.url)?, p.id)))
        .collect();
    let forms = fetch_many("forms", form_ids.keys().copied(), n, |id| {
        pokemon::pokemon_form::get_by_id(id, c)
    })
    .await;
    raw.forms = forms
        .into_iter()
        .filter_map(|(form_id, form)| Some((*form_ids.get(&form_id)?, form)))
        .collect();

    let chain_ids: BTreeSet<i64> = raw
        .species
        .values()
        .filter_map(|s| id_of(&s.evolution_chain.as_ref()?.url))
        .collect();
    // Fetched raw: PokéAPI sends `percentage_chance` as a float, rustemon expects an integer.
    raw.chains = fetch_many("evolution chains", chain_ids, n, |id| {
        raw_api.get_patched::<EvolutionChain>("evolution-chain", id, |v| {
            crate::api::floats_to_ints(v, "percentage_chance")
        })
    })
    .await;

    println!("Fetching locations");
    let area_ids: BTreeSet<i64> = raw
        .encounters
        .values()
        .flatten()
        .filter_map(|e| id_of(&e.location_area.url))
        .collect();
    raw.location_areas = fetch_many("location areas", area_ids, n, |id| {
        locations::location_area::get_by_id(id, c)
    })
    .await;
    let location_ids: BTreeSet<i64> = raw
        .location_areas
        .values()
        .filter_map(|a| id_of(&a.location.url))
        .chain(
            raw.chains
                .values()
                .flat_map(|chain| chain_details(&chain.chain))
                .filter_map(|d| id_of(&d.location.as_ref()?.url)),
        )
        .collect();
    raw.locations = fetch_many("locations", location_ids, n, |id| {
        locations::location::get_by_id(id, c)
    })
    .await;

    println!("Fetching items");
    let mut item_ids: BTreeSet<i64> = raw
        .pokemon
        .values()
        .flat_map(|p| p.held_items.iter().filter_map(|h| id_of(&h.item.url)))
        .collect();
    for chain in raw.chains.values() {
        item_ids.extend(chain.baby_trigger_item.as_ref().and_then(|i| id_of(&i.url)));
        for detail in chain_details(&chain.chain) {
            item_ids.extend(detail.item.as_ref().and_then(|i| id_of(&i.url)));
            item_ids.extend(detail.held_item.as_ref().and_then(|i| id_of(&i.url)));
        }
    }
    raw.items = fetch_many("items", item_ids, n, |id| items::item::get_by_id(id, c)).await;

    raw
}

/// Every evolution detail in a chain (depth first).
fn chain_details(link: &ChainLink) -> Vec<&EvolutionDetail> {
    let mut out: Vec<&EvolutionDetail> = link.evolution_details.iter().collect();
    for next in &link.evolves_to {
        out.extend(chain_details(next));
    }
    out
}

//
// Conversion
//

fn u8s(v: i64) -> u8 {
    v.clamp(0, i64::from(u8::MAX)) as u8
}
fn i8s(v: i64) -> i8 {
    v.clamp(i64::from(i8::MIN), i64::from(i8::MAX)) as i8
}
fn u16s(v: i64) -> u16 {
    v.clamp(0, i64::from(u16::MAX)) as u16
}
fn u32s(v: i64) -> u32 {
    v.clamp(0, i64::from(u32::MAX)) as u32
}
fn generation(name: &str) -> model::Generation {
    model::Generation::from_api(name).unwrap_or_default()
}
/// Localized names; when PokéAPI has no English name the identifier is title-cased instead of
/// promoting another language into the English slot.
fn names_of(names: &[rustemon::model::resource::Name], identifier: &str) -> Text {
    let mut pairs: Vec<(model::Lang, String)> = names
        .iter()
        .filter_map(|n| crate::text::lang_of(&n.language).map(|l| (l, crate::text::clean(&n.name))))
        .collect();
    if !pairs.iter().any(|(lang, _)| *lang == model::Lang::En) {
        pairs.insert(0, (model::Lang::En, title_case(identifier)));
    }
    or_title(Text::from_pairs(pairs), identifier)
}
fn opt_id(resource: Option<&NamedApiResource<impl Sized>>) -> Option<u32> {
    resource.map(|r| id32(&r.url))
}
fn named(id: i64, name: &str, names: &[rustemon::model::resource::Name]) -> model::Named {
    model::Named {
        id: u32s(id),
        name: name.to_string(),
        display: names_of(names, name),
    }
}

/// Converts the raw data into the archive model plus the list of assets to download.
pub fn build(raw: &Raw) -> (model::Pokedex, AssetPlan) {
    let mut plan = AssetPlan::default();

    // Versions & version groups
    let versions: Vec<model::Version> = raw
        .versions
        .values()
        .map(|v| model::Version {
            id: u16s(v.id),
            name: v.name.clone(),
            display: names_of(&v.names, &v.name),
            version_group_id: id16(&v.version_group.url),
        })
        .collect();
    let version_text: HashMap<u16, &Text> = versions.iter().map(|v| (v.id, &v.display)).collect();
    let version_groups: Vec<model::VersionGroup> = raw
        .version_groups
        .values()
        .map(|vg| {
            let version_ids: Vec<u16> = vg.versions.iter().map(|v| id16(&v.url)).collect();
            let texts: Vec<&Text> = version_ids
                .iter()
                .filter_map(|id| version_text.get(id).copied())
                .collect();
            model::VersionGroup {
                id: u16s(vg.id),
                name: vg.name.clone(),
                display: if texts.is_empty() {
                    Text::en(title_case(&vg.name))
                } else {
                    join(&texts, " / ")
                },
                generation: generation(&vg.generation.name),
                order: u16s(vg.order),
                version_ids,
            }
        })
        .collect();

    // Type chart
    let mut type_chart = vec![2u8; model::PokemonType::ALL.len().pow(2)];
    for t in raw.types.values() {
        let Some(attacker) = model::PokemonType::from_api(&t.name) else {
            continue;
        };
        let row = attacker.index() * model::PokemonType::ALL.len();
        let rel = &t.damage_relations;
        for (targets, value) in [
            (&rel.double_damage_to, 4u8),
            (&rel.half_damage_to, 1),
            (&rel.no_damage_to, 0),
        ] {
            for target in targets {
                if let Some(defender) = model::PokemonType::from_api(&target.name) {
                    type_chart[row + defender.index()] = value;
                }
            }
        }
    }

    // Moves
    let targets: HashMap<i64, Text> = raw
        .move_targets
        .values()
        .map(|t| (t.id, names_of(&t.names, &t.name)))
        .collect();
    let moves: Vec<model::Move> = raw
        .moves
        .values()
        .map(|m| {
            let chance = m.effect_chance.map(|c| c.to_string()).unwrap_or_default();
            let fill = |s: &str| s.replace("$effect_chance", &chance);
            let mut machines: Vec<model::MachineEntry> = m
                .machines
                .iter()
                .filter_map(|d| {
                    Some(model::MachineEntry {
                        version_group_id: id16(&d.version_group.url),
                        label: raw.machines.get(&id_of(&d.machine.url)?)?.clone(),
                    })
                })
                .collect();
            machines.sort_by_key(|e| e.version_group_id);
            machines.dedup_by_key(|e| e.version_group_id);
            model::Move {
                id: u32s(m.id),
                name: m.name.clone(),
                display: names_of(&m.names, &m.name),
                move_type: model::PokemonType::from_api(&m.type_.name)
                    .unwrap_or(model::PokemonType::Normal),
                damage_class: model::DamageClass::from_api(&m.damage_class.name)
                    .unwrap_or(model::DamageClass::Status),
                power: m.power.map(u8s),
                accuracy: m.accuracy.map(u8s),
                pp: m.pp.map(u8s),
                priority: i8s(m.priority),
                effect_chance: m.effect_chance.map(u8s),
                generation: generation(&m.generation.name),
                target: id_of(&m.target.url)
                    .and_then(|id| targets.get(&id).cloned())
                    .unwrap_or_else(|| Text::en(title_case(&m.target.name))),
                short_effect: map_text(
                    text_from(&m.effect_entries, |e| &e.language, |e| &e.short_effect),
                    fill,
                ),
                effect: map_text(
                    text_from(&m.effect_entries, |e| &e.language, |e| &e.effect),
                    fill,
                ),
                flavor: latest_text_from(
                    m.flavor_text_entries.iter(),
                    |f| &f.language,
                    |f| &f.flavor_text,
                ),
                meta: m.meta.as_ref().map(|meta| model::MoveMeta {
                    ailment: if meta.ailment.name == "none" {
                        String::new()
                    } else {
                        title_case(&meta.ailment.name)
                    },
                    ailment_chance: u8s(meta.ailment_chance),
                    min_hits: meta.min_hits.map(u8s),
                    max_hits: meta.max_hits.map(u8s),
                    min_turns: meta.min_turns.map(u8s),
                    max_turns: meta.max_turns.map(u8s),
                    drain: i8s(meta.drain),
                    healing: i8s(meta.healing),
                    crit_rate: u8s(meta.crit_rate),
                    flinch_chance: u8s(meta.flinch_chance),
                    stat_chance: u8s(meta.stat_chance),
                }),
                stat_changes: m
                    .stat_changes
                    .iter()
                    .filter_map(|s| {
                        Some(model::StatChange {
                            stat: model::Stat::from_api(&s.stat.name)?,
                            change: i8s(s.change),
                        })
                    })
                    .collect(),
                machines,
            }
        })
        .collect();

    let abilities: Vec<model::Ability> = raw
        .abilities
        .values()
        .map(|a| model::Ability {
            id: u32s(a.id),
            name: a.name.clone(),
            display: names_of(&a.names, &a.name),
            generation: generation(&a.generation.name),
            is_main_series: a.is_main_series,
            short_effect: text_from(&a.effect_entries, |e| &e.language, |e| &e.short_effect),
            effect: text_from(&a.effect_entries, |e| &e.language, |e| &e.effect),
            flavor: latest_text_from(
                a.flavor_text_entries.iter(),
                |f| &f.language,
                |f| &f.flavor_text,
            ),
        })
        .collect();

    let items: Vec<model::Item> = raw
        .items
        .values()
        .map(|i| {
            let has_sprite = if let Some(url) = &i.sprites.default {
                plan.sprites.push(Download {
                    url: url.clone(),
                    path: model::item_sprite_rel_path(&i.name),
                    resize: false,
                });
                true
            } else {
                false
            };
            model::Item {
                id: u32s(i.id),
                name: i.name.clone(),
                display: names_of(&i.names, &i.name),
                category: title_case(&i.category.name),
                cost: i.cost.filter(|c| *c > 0).map(u32s),
                short_effect: text_from(&i.effect_entries, |e| &e.language, |e| &e.short_effect),
                flavor: latest_text_from(
                    i.flavor_text_entries.iter(),
                    |f| &f.language,
                    |f| &f.text,
                ),
                has_sprite,
            }
        })
        .collect();

    // Species
    let lookup = |map: &HashMap<i64, Text>, url: Option<&str>| -> Text {
        url.and_then(id_of)
            .and_then(|id| map.get(&id).cloned())
            .unwrap_or_default()
    };
    let colors: HashMap<i64, Text> = raw
        .colors
        .values()
        .map(|c| (c.id, names_of(&c.names, &c.name)))
        .collect();
    let shapes: HashMap<i64, Text> = raw
        .shapes
        .values()
        .map(|s| (s.id, names_of(&s.names, &s.name)))
        .collect();
    let habitats: HashMap<i64, Text> = raw
        .habitats
        .values()
        .map(|h| (h.id, names_of(&h.names, &h.name)))
        .collect();

    let species: Vec<model::Species> = raw
        .species
        .values()
        .map(|s| {
            let mut flavor_texts: Vec<model::FlavorText> = Vec::new();
            for entry in &s.flavor_text_entries {
                let Some(lang) = crate::text::lang_of(&entry.language) else {
                    continue;
                };
                let text = crate::text::clean(&entry.flavor_text);
                let version = entry.version.as_ref().map(|v| id16(&v.url));
                match flavor_texts
                    .iter_mut()
                    .find(|f| f.lang == lang && f.text == text)
                {
                    Some(existing) => existing.version_ids.extend(version),
                    None => flavor_texts.push(model::FlavorText {
                        lang,
                        text,
                        version_ids: version.into_iter().collect(),
                    }),
                }
            }
            for f in &mut flavor_texts {
                f.version_ids.sort_unstable();
                f.version_ids.dedup();
            }
            let mut varieties: Vec<(bool, u32)> = s
                .varieties
                .iter()
                .map(|v| (!v.is_default, id32(&v.pokemon.url)))
                .collect();
            varieties.sort();
            model::Species {
                id: u32s(s.id),
                name: s.name.clone(),
                display: names_of(&s.names, &s.name),
                genus: text_from(&s.genera, |g| &g.language, |g| &g.genus),
                generation: generation(&s.generation.name),
                order: s.order as i32,
                gender_rate: i8s(s.gender_rate),
                capture_rate: u8s(s.capture_rate),
                base_happiness: s.base_happiness.map(u8s),
                hatch_counter: s.hatch_counter.map(u8s),
                is_baby: s.is_baby,
                is_legendary: s.is_legendary,
                is_mythical: s.is_mythical,
                has_gender_differences: s.has_gender_differences,
                forms_switchable: s.forms_switchable,
                growth_rate: model::GrowthRate::from_api(&s.growth_rate.name).unwrap_or_default(),
                egg_groups: s
                    .egg_groups
                    .iter()
                    .filter_map(|e| model::EggGroup::from_api(&e.name))
                    .collect(),
                color: lookup(&colors, Some(&s.color.url)),
                shape: lookup(&shapes, s.shape.as_ref().map(|r| r.url.as_str())),
                habitat: lookup(&habitats, s.habitat.as_ref().map(|r| r.url.as_str())),
                evolves_from: opt_id(s.evolves_from_species.as_ref()),
                evolution_chain_id: s.evolution_chain.as_ref().map(|c| id32(&c.url)),
                flavor_texts,
                dex_numbers: s
                    .pokedex_numbers
                    .iter()
                    .map(|d| model::DexNumber {
                        pokedex_id: id16(&d.pokedex.url),
                        number: u16s(d.entry_number),
                    })
                    .collect(),
                varieties: varieties.into_iter().map(|(_, id)| id).collect(),
            }
        })
        .collect();
    let species_names: HashMap<u32, &Text> = species.iter().map(|s| (s.id, &s.display)).collect();

    // Pokémon
    let vg_versions: HashMap<u16, &Vec<u16>> = version_groups
        .iter()
        .map(|vg| (vg.id, &vg.version_ids))
        .collect();
    let pokemon: Vec<model::Pokemon> = raw
        .pokemon
        .values()
        .map(|p| {
            let id = u32s(p.id);
            let species_id = id32(&p.species.url);
            let species_name = species_names
                .get(&species_id)
                .map(|t| (*t).clone())
                .unwrap_or_else(|| Text::en(title_case(&p.species.name)));
            let form = raw.forms.get(&p.id);
            let form_name = form
                .map(|f| text_from(&f.form_names, |n| &n.language, |n| &n.name))
                .unwrap_or_default();
            let display = if p.is_default {
                species_name.clone()
            } else {
                let full = form
                    .map(|f| text_from(&f.names, |n| &n.language, |n| &n.name))
                    .unwrap_or_default();
                if !full.is_empty() {
                    full
                } else if !form_name.is_empty() {
                    Text::from_pairs(model::Lang::ALL.iter().map(|lang| {
                        (
                            *lang,
                            format!("{} ({})", species_name.get(*lang), form_name.get(*lang)),
                        )
                    }))
                } else {
                    Text::en(title_case(&p.name))
                }
            };

            let mut stats = model::Stats::default();
            let mut ev_yield = model::Stats::default();
            for s in &p.stats {
                let (base, effort) = (u8s(s.base_stat), u8s(s.effort));
                let (b, e) = match model::Stat::from_api(&s.stat.name) {
                    Some(model::Stat::Hp) => (&mut stats.hp, &mut ev_yield.hp),
                    Some(model::Stat::Attack) => (&mut stats.attack, &mut ev_yield.attack),
                    Some(model::Stat::Defense) => (&mut stats.defense, &mut ev_yield.defense),
                    Some(model::Stat::SpAttack) => (&mut stats.sp_attack, &mut ev_yield.sp_attack),
                    Some(model::Stat::SpDefense) => {
                        (&mut stats.sp_defense, &mut ev_yield.sp_defense)
                    }
                    Some(model::Stat::Speed) => (&mut stats.speed, &mut ev_yield.speed),
                    _ => continue,
                };
                *b = base;
                *e = effort;
            }

            let mut moves: Vec<model::PokemonMove> = p
                .moves
                .iter()
                .map(|pm| {
                    let mut learns: Vec<model::MoveLearn> = pm
                        .version_group_details
                        .iter()
                        .map(|d| model::MoveLearn {
                            version_group_id: id16(&d.version_group.url),
                            method: model::LearnMethod::from_api_lossy(&d.move_learn_method.name),
                            level: u8s(d.level_learned_at),
                        })
                        .collect();
                    learns.sort_by_key(|l| (l.version_group_id, l.method, l.level));
                    learns.dedup_by_key(|l| (l.version_group_id, l.method, l.level));
                    model::PokemonMove {
                        move_id: id32(&pm.move_.url),
                        learns,
                    }
                })
                .collect();
            moves.sort_by_key(|m| m.move_id);

            let mut game_versions: BTreeSet<u16> = p
                .game_indices
                .iter()
                .map(|g| id16(&g.version.url))
                .collect();
            for learn in moves.iter().flat_map(|m| &m.learns) {
                if let Some(vs) = vg_versions.get(&learn.version_group_id) {
                    game_versions.extend(vs.iter().copied());
                }
            }

            let encounters = raw
                .encounters
                .get(&p.id)
                .map(|list| {
                    list.iter()
                        .map(|e| model::Encounter {
                            location_area_id: id32(&e.location_area.url),
                            versions: e
                                .version_details
                                .iter()
                                .map(|vd| {
                                    let details = &vd.encounter_details;
                                    let mut methods: Vec<u16> =
                                        details.iter().map(|d| id16(&d.method.url)).collect();
                                    methods.sort_unstable();
                                    methods.dedup();
                                    let mut conditions: Vec<u16> = details
                                        .iter()
                                        .flat_map(|d| {
                                            d.condition_values.iter().map(|c| id16(&c.url))
                                        })
                                        .collect();
                                    conditions.sort_unstable();
                                    conditions.dedup();
                                    model::EncounterVersion {
                                        version_id: id16(&vd.version.url),
                                        min_level: details
                                            .iter()
                                            .map(|d| u8s(d.min_level))
                                            .min()
                                            .unwrap_or(0),
                                        max_level: details
                                            .iter()
                                            .map(|d| u8s(d.max_level))
                                            .max()
                                            .unwrap_or(0),
                                        max_chance: u8s(vd.max_chance),
                                        methods,
                                        conditions,
                                    }
                                })
                                .collect(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Sprites: regular 96px sprites, falling back to (downscaled) HOME renders or artwork
            // for the handful of forms that have no regular sprite.
            let s = &p.sprites;
            let home = &s.other.home;
            let art = &s.other.official_artwork;
            let candidates: [(SpriteKind, Option<&String>, Option<&String>); 8] = [
                (
                    SpriteKind::Front,
                    s.front_default.as_ref(),
                    home.front_default.as_ref().or(art.front_default.as_ref()),
                ),
                (
                    SpriteKind::FrontShiny,
                    s.front_shiny.as_ref(),
                    home.front_shiny.as_ref(),
                ),
                (SpriteKind::Back, s.back_default.as_ref(), None),
                (SpriteKind::BackShiny, s.back_shiny.as_ref(), None),
                (SpriteKind::FrontFemale, s.front_female.as_ref(), None),
                (
                    SpriteKind::FrontShinyFemale,
                    s.front_shiny_female.as_ref(),
                    None,
                ),
                (SpriteKind::BackFemale, s.back_female.as_ref(), None),
                (
                    SpriteKind::BackShinyFemale,
                    s.back_shiny_female.as_ref(),
                    None,
                ),
            ];
            let mut sprites = 0u16;
            for (kind, regular, fallback) in candidates {
                let (url, resize) = match (regular, fallback) {
                    (Some(url), _) => (url, false),
                    (None, Some(url)) => (url, true),
                    (None, None) => continue,
                };
                sprites |= kind as u16;
                plan.sprites.push(Download {
                    url: url.clone(),
                    path: model::sprite_rel_path(id, kind),
                    resize,
                });
            }
            let mut cries = 0u8;
            for (kind, url) in [
                (CryKind::Latest, &p.cries.latest),
                (CryKind::Legacy, &p.cries.legacy),
            ] {
                if let Some(url) = url {
                    cries |= kind as u8;
                    plan.cries.push(Download {
                        url: url.clone(),
                        path: format!("cries/{id}/{}", kind.file_name()),
                        resize: false,
                    });
                }
            }

            model::Pokemon {
                id,
                name: p.name.clone(),
                species_id,
                is_default: p.is_default,
                order: p.order as i32,
                display,
                form_name,
                height: u16s(p.height),
                weight: u16s(p.weight),
                base_experience: p.base_experience.map(u16s),
                types: types_of(p.types.iter().map(|t| (t.slot, t.type_.name.as_str()))),
                past_types: p
                    .past_types
                    .iter()
                    .map(|pt| model::PastTypes {
                        until: generation(&pt.generation.name),
                        types: types_of(pt.types.iter().map(|t| (t.slot, t.type_.name.as_str()))),
                    })
                    .collect(),
                abilities: p
                    .abilities
                    .iter()
                    .filter_map(|a| {
                        Some(model::AbilitySlot {
                            ability_id: id32(&a.ability.as_ref()?.url),
                            slot: u8s(a.slot),
                            is_hidden: a.is_hidden,
                        })
                    })
                    .collect(),
                stats,
                ev_yield,
                held_items: p
                    .held_items
                    .iter()
                    .map(|h| model::HeldItem {
                        item_id: id32(&h.item.url),
                        versions: h
                            .version_details
                            .iter()
                            .map(|v| model::HeldItemVersion {
                                version_id: id16(&v.version.url),
                                rarity: u8s(v.rarity),
                            })
                            .collect(),
                    })
                    .collect(),
                moves,
                encounters,
                sprites,
                cries,
                game_versions: game_versions.into_iter().collect(),
            }
        })
        .collect();

    // Evolution
    let evolution_chains: Vec<model::EvolutionChain> = raw
        .chains
        .values()
        .map(|chain| {
            let mut nodes = Vec::new();
            flatten_chain(&chain.chain, None, 0, &mut nodes);
            model::EvolutionChain {
                id: u32s(chain.id),
                baby_trigger_item_id: opt_id(chain.baby_trigger_item.as_ref()),
                nodes,
            }
        })
        .collect();

    // Places
    let location_areas: Vec<model::LocationArea> = raw
        .location_areas
        .values()
        .map(|a| model::LocationArea {
            id: u32s(a.id),
            location_id: id32(&a.location.url),
            display: text_from(&a.names, |n| &n.language, |n| &n.name),
        })
        .collect();
    let locations: Vec<model::Location> = raw
        .locations
        .values()
        .map(|l| model::Location {
            id: u32s(l.id),
            name: l.name.clone(),
            display: names_of(&l.names, &l.name),
            region_id: opt_id(l.region.as_ref()),
        })
        .collect();

    let dex = model::Pokedex {
        format_version: model::FORMAT_VERSION,
        generated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        pokemon,
        species,
        moves,
        abilities,
        items,
        evolution_chains,
        versions,
        version_groups,
        pokedexes: raw
            .pokedexes
            .values()
            .map(|d| model::RegionalDex {
                id: u16s(d.id),
                name: d.name.clone(),
                display: names_of(&d.names, &d.name),
                is_main_series: d.is_main_series,
            })
            .collect(),
        location_areas,
        locations,
        regions: raw
            .regions
            .values()
            .map(|r| named(r.id, &r.name, &r.names))
            .collect(),
        encounter_methods: raw
            .encounter_methods
            .values()
            .map(|m| named(m.id, &m.name, &m.names))
            .collect(),
        encounter_conditions: raw
            .encounter_conditions
            .values()
            .map(|c| named(c.id, &c.name, &c.names))
            .collect(),
        evolution_triggers: raw
            .evolution_triggers
            .values()
            .map(|t| named(t.id, &t.name, &t.names))
            .collect(),
        type_chart,
    };
    (dex, plan)
}

fn types_of<'a>(types: impl Iterator<Item = (i64, &'a str)>) -> Vec<model::PokemonType> {
    let mut types: Vec<(i64, &str)> = types.collect();
    types.sort_by_key(|(slot, _)| *slot);
    types
        .into_iter()
        .filter_map(|(_, name)| model::PokemonType::from_api(name))
        .collect()
}

/// Depth-first flattening of an evolution tree.
fn flatten_chain(
    link: &ChainLink,
    parent: Option<u16>,
    stage: u8,
    out: &mut Vec<model::EvolutionNode>,
) {
    let mut methods: Vec<model::EvolutionMethod> = Vec::new();
    for detail in &link.evolution_details {
        let method = evolution_method(detail);
        // Newer PokéAPI data repeats identical conditions once per game.
        if !methods.contains(&method) {
            methods.push(method);
        }
    }
    let index = out.len() as u16;
    out.push(model::EvolutionNode {
        species_id: id32(&link.species.url),
        parent,
        stage,
        is_baby: link.is_baby,
        methods,
    });
    for next in &link.evolves_to {
        flatten_chain(next, Some(index), stage.saturating_add(1), out);
    }
}

fn evolution_method(d: &EvolutionDetail) -> model::EvolutionMethod {
    model::EvolutionMethod {
        trigger_id: id16(&d.trigger.url),
        min_level: d.min_level.map(u8s),
        item_id: opt_id(d.item.as_ref()),
        held_item_id: opt_id(d.held_item.as_ref()),
        known_move_id: opt_id(d.known_move.as_ref()),
        known_move_type: d
            .known_move_type
            .as_ref()
            .and_then(|t| model::PokemonType::from_api(&t.name)),
        location_id: opt_id(d.location.as_ref()),
        gender: d.gender.map(u8s),
        min_happiness: d.min_happiness.map(u8s),
        min_beauty: d.min_beauty.map(u8s),
        min_affection: d.min_affection.map(u8s),
        time_of_day: d.time_of_day.clone(),
        needs_overworld_rain: d.needs_overworld_rain,
        turn_upside_down: d.turn_upside_down,
        needs_multiplayer: d.needs_multiplayer,
        party_species_id: opt_id(d.party_species.as_ref()),
        party_type: d
            .party_type
            .as_ref()
            .and_then(|t| model::PokemonType::from_api(&t.name)),
        trade_species_id: opt_id(d.trade_species.as_ref()),
        relative_physical_stats: d.relative_physical_stats.map(i8s),
        used_move_id: opt_id(d.used_move.as_ref()),
        min_move_count: d.min_move_count.map(u32s),
        min_steps: d.min_steps.map(u32s),
        min_damage_taken: d.min_damage_taken.map(u32s),
        chance: d
            .condition_expression
            .as_ref()
            .and_then(|c| c.percentage_chance)
            .map(u8s),
    }
}
