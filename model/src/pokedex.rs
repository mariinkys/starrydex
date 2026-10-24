// SPDX-License-Identifier: GPL-3.0-only

//! The archive layout.
//!
//! Everything is *normalized*: moves, abilities, items, locations, games... are stored once in
//! their own table and Pokémon refer to them by id. Every table is sorted by id so lookups are a
//! binary search straight on the memory-mapped bytes, no index has to be built in RAM.

use rkyv::{Archive, Deserialize, Serialize};

use crate::{
    DamageClass, EggGroup, Generation, GrowthRate, Lang, LearnMethod, PokemonType, Stat, Text,
};

/// Bump whenever the layout of any archived type changes.
pub const FORMAT_VERSION: u32 = 1;

/// Root of the archive.
#[derive(Archive, Serialize, Deserialize, Debug, Default)]
#[rkyv(derive(Debug))]
pub struct Pokedex {
    pub format_version: u32,
    /// Unix timestamp (seconds) of when assetgen produced the data.
    pub generated_at: u64,
    /// All Pokémon, including alternate forms (megas, regional forms...). Sorted by id.
    pub pokemon: Vec<Pokemon>,
    /// Sorted by id.
    pub species: Vec<Species>,
    /// Sorted by id.
    pub moves: Vec<Move>,
    /// Sorted by id.
    pub abilities: Vec<Ability>,
    /// Only the items referenced by Pokémon data (held items, evolution items). Sorted by id.
    pub items: Vec<Item>,
    /// Sorted by id.
    pub evolution_chains: Vec<EvolutionChain>,
    /// Sorted by id.
    pub versions: Vec<Version>,
    /// Sorted by id.
    pub version_groups: Vec<VersionGroup>,
    /// Regional dexes. Sorted by id.
    pub pokedexes: Vec<RegionalDex>,
    /// Only the location areas that appear in encounters. Sorted by id.
    pub location_areas: Vec<LocationArea>,
    /// Sorted by id.
    pub locations: Vec<Location>,
    /// Sorted by id.
    pub regions: Vec<Named>,
    /// Encounter methods (walking, surfing, old rod...). Sorted by id.
    pub encounter_methods: Vec<Named>,
    /// Encounter condition values (morning, swarm, radar on...). Sorted by id.
    pub encounter_conditions: Vec<Named>,
    /// Evolution triggers (level up, trade, use item...). Sorted by id.
    pub evolution_triggers: Vec<Named>,
    /// Attack-major 18×18 matrix (`attacker * 18 + defender`, see [`PokemonType::index`]).
    /// Values are the damage multiplier ×2: `0` = immune, `1` = ½, `2` = 1×, `4` = 2×.
    pub type_chart: Vec<u8>,
}

/// A generic id → localized name entry for small lookup tables.
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Named {
    pub id: u32,
    pub name: String,
    pub display: Text,
}

//
// Pokémon
//

/// A battle-relevant Pokémon variety (Bulbasaur, Mega Charizard X, Alolan Raichu...).
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Pokemon {
    pub id: u32,
    /// PokéAPI identifier, e.g. `charizard-mega-x`.
    pub name: String,
    pub species_id: u32,
    /// `true` for the default variety of its species.
    pub is_default: bool,
    /// Sort order used by PokéAPI (keeps forms next to their base species).
    pub order: i32,
    /// Full display name ("Mega Charizard X", "Alolan Raichu", "Bulbasaur").
    pub display: Text,
    /// Form name only ("Mega X", "Alolan Form"), empty for default forms.
    pub form_name: Text,
    /// Decimetres.
    pub height: u16,
    /// Hectograms.
    pub weight: u16,
    pub base_experience: Option<u16>,
    pub types: Vec<PokemonType>,
    /// Types this Pokémon had in older generations.
    pub past_types: Vec<PastTypes>,
    pub abilities: Vec<AbilitySlot>,
    pub stats: Stats,
    /// EVs awarded when defeated.
    pub ev_yield: Stats,
    pub held_items: Vec<HeldItem>,
    /// Sorted by move id.
    pub moves: Vec<PokemonMove>,
    pub encounters: Vec<Encounter>,
    /// [`crate::SpriteKind`] bit flags.
    pub sprites: u16,
    /// [`crate::CryKind`] bit flags.
    pub cries: u8,
    /// Ids of the versions (games) this Pokémon appears in, sorted.
    pub game_versions: Vec<u16>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct PastTypes {
    /// Last generation in which the Pokémon had these types.
    pub until: Generation,
    pub types: Vec<PokemonType>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct AbilitySlot {
    pub ability_id: u32,
    pub slot: u8,
    pub is_hidden: bool,
}

/// Base stats or EV yields. Base stats never exceed 255.
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct Stats {
    pub hp: u8,
    pub attack: u8,
    pub defense: u8,
    pub sp_attack: u8,
    pub sp_defense: u8,
    pub speed: u8,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct HeldItem {
    pub item_id: u32,
    pub versions: Vec<HeldItemVersion>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct HeldItemVersion {
    pub version_id: u16,
    /// Percentage chance of the wild Pokémon holding the item.
    pub rarity: u8,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct PokemonMove {
    pub move_id: u32,
    pub learns: Vec<MoveLearn>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy)]
#[rkyv(derive(Debug))]
pub struct MoveLearn {
    pub version_group_id: u16,
    pub method: LearnMethod,
    /// 0 when not learned by level up.
    pub level: u8,
}

/// Where and how a Pokémon can be found in the wild.
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Encounter {
    pub location_area_id: u32,
    pub versions: Vec<EncounterVersion>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct EncounterVersion {
    pub version_id: u16,
    pub min_level: u8,
    pub max_level: u8,
    /// Highest encounter chance (percentage) among the details for this version.
    pub max_chance: u8,
    /// Ids into [`Pokedex::encounter_methods`].
    pub methods: Vec<u16>,
    /// Ids into [`Pokedex::encounter_conditions`].
    pub conditions: Vec<u16>,
}

//
// Species
//

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Species {
    pub id: u32,
    pub name: String,
    pub display: Text,
    /// "Seed Pokémon", "Mouse Pokémon"...
    pub genus: Text,
    pub generation: Generation,
    pub order: i32,
    /// Chance of being female in eighths, or `-1` for genderless.
    pub gender_rate: i8,
    pub capture_rate: u8,
    pub base_happiness: Option<u8>,
    /// Egg cycles.
    pub hatch_counter: Option<u8>,
    pub is_baby: bool,
    pub is_legendary: bool,
    pub is_mythical: bool,
    pub has_gender_differences: bool,
    pub forms_switchable: bool,
    pub growth_rate: GrowthRate,
    pub egg_groups: Vec<EggGroup>,
    pub color: Text,
    pub shape: Text,
    pub habitat: Text,
    pub evolves_from: Option<u32>,
    pub evolution_chain_id: Option<u32>,
    /// Pokédex entries, deduplicated per language (identical texts share one entry).
    pub flavor_texts: Vec<FlavorText>,
    pub dex_numbers: Vec<DexNumber>,
    /// Pokémon ids of every variety of this species (the default one first).
    pub varieties: Vec<u32>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct FlavorText {
    pub lang: Lang,
    pub text: String,
    /// Versions whose Pokédex shows this text, sorted.
    pub version_ids: Vec<u16>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct DexNumber {
    pub pokedex_id: u16,
    pub number: u16,
}

//
// Moves, abilities, items
//

#[derive(Archive, Serialize, Deserialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct Move {
    pub id: u32,
    pub name: String,
    pub display: Text,
    pub move_type: PokemonType,
    pub damage_class: DamageClass,
    pub power: Option<u8>,
    pub accuracy: Option<u8>,
    pub pp: Option<u8>,
    pub priority: i8,
    pub effect_chance: Option<u8>,
    pub generation: Generation,
    pub target: Text,
    /// One-liner effect (with the effect chance already substituted).
    pub short_effect: Text,
    /// Full mechanics description.
    pub effect: Text,
    /// In-game description from the most recent game.
    pub flavor: Text,
    pub meta: Option<MoveMeta>,
    pub stat_changes: Vec<StatChange>,
    /// TM/HM/TR numbers per game.
    pub machines: Vec<MachineEntry>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct MoveMeta {
    /// Status condition inflicted ("Paralysis", "Burn"...), empty if none.
    pub ailment: String,
    pub ailment_chance: u8,
    pub min_hits: Option<u8>,
    pub max_hits: Option<u8>,
    pub min_turns: Option<u8>,
    pub max_turns: Option<u8>,
    /// HP drained (positive) or recoil (negative) as % of damage dealt.
    pub drain: i8,
    /// HP restored as % of max HP.
    pub healing: i8,
    /// Critical hit stage bonus.
    pub crit_rate: u8,
    pub flinch_chance: u8,
    pub stat_chance: u8,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Copy)]
#[rkyv(derive(Debug))]
pub struct StatChange {
    pub stat: Stat,
    pub change: i8,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct MachineEntry {
    pub version_group_id: u16,
    /// "TM24", "HM03", "TR12"...
    pub label: String,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Ability {
    pub id: u32,
    pub name: String,
    pub display: Text,
    pub generation: Generation,
    pub is_main_series: bool,
    pub short_effect: Text,
    pub effect: Text,
    /// In-game description from the most recent game.
    pub flavor: Text,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Item {
    pub id: u32,
    pub name: String,
    pub display: Text,
    pub category: String,
    pub cost: Option<u32>,
    pub short_effect: Text,
    pub flavor: Text,
    /// `true` if `sprites/items/{name}.png` exists.
    pub has_sprite: bool,
}

//
// Evolution
//

/// An evolution family, flattened into a list of nodes (each pointing at its parent).
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct EvolutionChain {
    pub id: u32,
    /// Incense item a parent must hold to breed the baby species.
    pub baby_trigger_item_id: Option<u32>,
    /// Depth-first order: a node always comes after its parent.
    pub nodes: Vec<EvolutionNode>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct EvolutionNode {
    pub species_id: u32,
    /// Index into [`EvolutionChain::nodes`] of the species this one evolves from.
    pub parent: Option<u16>,
    /// 0 for the base of the chain.
    pub stage: u8,
    pub is_baby: bool,
    /// Every way of evolving from the parent into this species.
    pub methods: Vec<EvolutionMethod>,
}

/// One set of conditions that must all hold to evolve.
#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub struct EvolutionMethod {
    /// Id into [`Pokedex::evolution_triggers`].
    pub trigger_id: u16,
    pub min_level: Option<u8>,
    pub item_id: Option<u32>,
    pub held_item_id: Option<u32>,
    pub known_move_id: Option<u32>,
    pub known_move_type: Option<PokemonType>,
    pub location_id: Option<u32>,
    /// 1 = female, 2 = male.
    pub gender: Option<u8>,
    pub min_happiness: Option<u8>,
    pub min_beauty: Option<u8>,
    pub min_affection: Option<u8>,
    /// "day", "night", "dusk" or empty.
    pub time_of_day: String,
    pub needs_overworld_rain: bool,
    pub turn_upside_down: bool,
    pub needs_multiplayer: bool,
    pub party_species_id: Option<u32>,
    pub party_type: Option<PokemonType>,
    pub trade_species_id: Option<u32>,
    /// 1 = Attack > Defense, -1 = Attack < Defense, 0 = equal.
    pub relative_physical_stats: Option<i8>,
    pub used_move_id: Option<u32>,
    pub min_move_count: Option<u32>,
    pub min_steps: Option<u32>,
    pub min_damage_taken: Option<u32>,
    /// Random evolutions (Wurmple, Tandemaus...): percentage chance of this outcome.
    pub chance: Option<u8>,
}

//
// Games & places
//

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Version {
    pub id: u16,
    pub name: String,
    pub display: Text,
    pub version_group_id: u16,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct VersionGroup {
    pub id: u16,
    pub name: String,
    /// Versions joined with " / ", e.g. "Scarlet / Violet".
    pub display: Text,
    pub generation: Generation,
    /// Chronological order.
    pub order: u16,
    pub version_ids: Vec<u16>,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct RegionalDex {
    pub id: u16,
    pub name: String,
    pub display: Text,
    pub is_main_series: bool,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct LocationArea {
    pub id: u32,
    pub location_id: u32,
    /// Area name, often empty when the location only has one area.
    pub display: Text,
}

#[derive(Archive, Serialize, Deserialize, Debug, Clone, Default)]
#[rkyv(derive(Debug))]
pub struct Location {
    pub id: u32,
    pub name: String,
    pub display: Text,
    pub region_id: Option<u32>,
}
