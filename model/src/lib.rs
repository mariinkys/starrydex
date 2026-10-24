// SPDX-License-Identifier: GPL-3.0-only

//! Shared data model for StarryDex.
//!
//! `assetgen` builds a [`Pokedex`] from PokéAPI and serializes it with rkyv; the app memory-maps
//! the resulting file and reads the archived types (`ArchivedPokedex`, `ArchivedPokemon`...)
//! in place, without deserializing. The helpers in this crate operate on those archived types.

mod enums;
mod pokedex;
mod text;

pub use enums::*;
pub use pokedex::*;
pub use text::*;

pub use rkyv;

/// Binary search a table sorted by id.
fn find_by_id<T>(table: &[T], id: u32, key: impl Fn(&T) -> u32) -> Option<&T> {
    table
        .binary_search_by_key(&id, key)
        .ok()
        .map(|index| &table[index])
}

impl ArchivedPokedex {
    pub fn pokemon(&self, id: u32) -> Option<&ArchivedPokemon> {
        find_by_id(&self.pokemon, id, |p| p.id.to_native())
    }

    /// Position of a Pokémon inside [`Pokedex::pokemon`].
    pub fn pokemon_index(&self, id: u32) -> Option<usize> {
        self.pokemon
            .binary_search_by_key(&id, |p| p.id.to_native())
            .ok()
    }

    pub fn species(&self, id: u32) -> Option<&ArchivedSpecies> {
        find_by_id(&self.species, id, |s| s.id.to_native())
    }

    /// Species of the given Pokémon.
    pub fn species_of(&self, pokemon: &ArchivedPokemon) -> Option<&ArchivedSpecies> {
        self.species(pokemon.species_id.to_native())
    }

    pub fn move_(&self, id: u32) -> Option<&ArchivedMove> {
        find_by_id(&self.moves, id, |m| m.id.to_native())
    }

    pub fn ability(&self, id: u32) -> Option<&ArchivedAbility> {
        find_by_id(&self.abilities, id, |a| a.id.to_native())
    }

    pub fn item(&self, id: u32) -> Option<&ArchivedItem> {
        find_by_id(&self.items, id, |i| i.id.to_native())
    }

    pub fn evolution_chain(&self, id: u32) -> Option<&ArchivedEvolutionChain> {
        find_by_id(&self.evolution_chains, id, |c| c.id.to_native())
    }

    pub fn version(&self, id: u16) -> Option<&ArchivedVersion> {
        find_by_id(&self.versions, u32::from(id), |v| {
            u32::from(v.id.to_native())
        })
    }

    pub fn version_group(&self, id: u16) -> Option<&ArchivedVersionGroup> {
        find_by_id(&self.version_groups, u32::from(id), |v| {
            u32::from(v.id.to_native())
        })
    }

    pub fn regional_dex(&self, id: u16) -> Option<&ArchivedRegionalDex> {
        find_by_id(&self.pokedexes, u32::from(id), |d| {
            u32::from(d.id.to_native())
        })
    }

    pub fn location_area(&self, id: u32) -> Option<&ArchivedLocationArea> {
        find_by_id(&self.location_areas, id, |a| a.id.to_native())
    }

    pub fn location(&self, id: u32) -> Option<&ArchivedLocation> {
        find_by_id(&self.locations, id, |l| l.id.to_native())
    }

    pub fn region(&self, id: u32) -> Option<&ArchivedNamed> {
        find_by_id(&self.regions, id, |r| r.id.to_native())
    }

    pub fn encounter_method(&self, id: u16) -> Option<&ArchivedNamed> {
        find_by_id(&self.encounter_methods, u32::from(id), |n| n.id.to_native())
    }

    pub fn encounter_condition(&self, id: u16) -> Option<&ArchivedNamed> {
        find_by_id(&self.encounter_conditions, u32::from(id), |n| {
            n.id.to_native()
        })
    }

    pub fn evolution_trigger(&self, id: u16) -> Option<&ArchivedNamed> {
        find_by_id(&self.evolution_triggers, u32::from(id), |n| {
            n.id.to_native()
        })
    }

    /// Damage multiplier ×2 of `attacker` hitting a pure `defender` type
    /// (0 = immune, 1 = ½, 2 = neutral, 4 = super effective).
    pub fn effectiveness_x2(&self, attacker: PokemonType, defender: PokemonType) -> u8 {
        self.type_chart
            .get(attacker.index() * PokemonType::ALL.len() + defender.index())
            .copied()
            .unwrap_or(2)
    }

    /// Damage multiplier of `attacker` against a (possibly dual-typed) defender.
    pub fn effectiveness(&self, attacker: PokemonType, defender: &[PokemonType]) -> f32 {
        defender
            .iter()
            .map(|d| f32::from(self.effectiveness_x2(attacker, *d)) / 2.0)
            .product()
    }
}

impl Stats {
    pub fn total(&self) -> u16 {
        [
            self.hp,
            self.attack,
            self.defense,
            self.sp_attack,
            self.sp_defense,
            self.speed,
        ]
        .iter()
        .map(|v| u16::from(*v))
        .sum()
    }
}

impl ArchivedStats {
    pub fn total(&self) -> u16 {
        self.values().iter().map(|(_, v)| u16::from(*v)).sum()
    }

    pub fn get(&self, stat: Stat) -> u8 {
        match stat {
            Stat::Hp => self.hp,
            Stat::Attack => self.attack,
            Stat::Defense => self.defense,
            Stat::SpAttack => self.sp_attack,
            Stat::SpDefense => self.sp_defense,
            Stat::Speed => self.speed,
            Stat::Accuracy | Stat::Evasion => 0,
        }
    }

    /// The six battle stats in canonical order.
    pub fn values(&self) -> [(Stat, u8); 6] {
        [
            (Stat::Hp, self.hp),
            (Stat::Attack, self.attack),
            (Stat::Defense, self.defense),
            (Stat::SpAttack, self.sp_attack),
            (Stat::SpDefense, self.sp_defense),
            (Stat::Speed, self.speed),
        ]
    }
}

impl ArchivedPokemon {
    pub fn has_sprite(&self, kind: SpriteKind) -> bool {
        self.sprites.to_native() & kind as u16 != 0
    }

    pub fn has_cry(&self, kind: CryKind) -> bool {
        self.cries & kind as u8 != 0
    }

    /// Path of a sprite relative to the extracted sprites directory, if that sprite exists.
    pub fn sprite_path(&self, kind: SpriteKind) -> Option<String> {
        self.has_sprite(kind)
            .then(|| sprite_rel_path(self.id.to_native(), kind))
    }

    /// Path of a cry relative to the extracted cries directory, if that cry exists.
    pub fn cry_path(&self, kind: CryKind) -> Option<String> {
        self.has_cry(kind)
            .then(|| format!("cries/{}/{}", self.id.to_native(), kind.file_name()))
    }

    pub fn has_type(&self, t: PokemonType) -> bool {
        self.types.contains(&t)
    }
}

/// Relative path of a Pokémon sprite inside the sprites archive.
pub fn sprite_rel_path(pokemon_id: u32, kind: SpriteKind) -> String {
    format!("sprites/pokemon/{pokemon_id}/{}", kind.file_name())
}

/// Relative path of an item sprite inside the sprites archive.
pub fn item_sprite_rel_path(item_name: &str) -> String {
    format!("sprites/items/{item_name}.png")
}

/// Level 100 stat range: (minimum, maximum).
///
/// Minimum is 0 IVs, 0 EVs and a hindering nature; maximum is 31 IVs, 252 EVs and a
/// beneficial nature. HP is unaffected by natures (and Shedinja always has 1 HP).
pub fn stat_range_at_100(stat: Stat, base: u8, pokemon_name: &str) -> (u16, u16) {
    let base = u32::from(base);
    let (min, max) = if stat == Stat::Hp {
        if pokemon_name == "shedinja" {
            (1, 1)
        } else {
            (2 * base + 110, 2 * base + 204)
        }
    } else {
        ((2 * base + 5) * 9 / 10, (2 * base + 99) * 11 / 10)
    };
    (min as u16, max as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_ranges_match_known_values() {
        // Pikachu: HP 35 → 180..274, Speed 90 → 166..306
        assert_eq!(stat_range_at_100(Stat::Hp, 35, "pikachu"), (180, 274));
        assert_eq!(stat_range_at_100(Stat::Speed, 90, "pikachu"), (166, 306));
        assert_eq!(stat_range_at_100(Stat::Hp, 1, "shedinja"), (1, 1));
    }

    #[test]
    fn lang_from_locale() {
        assert_eq!(Lang::from_locale("es-ES"), Lang::Es);
        assert_eq!(Lang::from_locale("pt_BR"), Lang::PtBr);
        assert_eq!(Lang::from_locale("zh-TW"), Lang::ZhHant);
        assert_eq!(Lang::from_locale("nl"), Lang::En);
    }

    #[test]
    fn text_from_pairs() {
        let t = Text::from_pairs([
            (Lang::Es, "Hola".into()),
            (Lang::En, "Hello".into()),
            (Lang::Fr, "Hello".into()),
        ]);
        assert_eq!(t.en, "Hello");
        assert_eq!(t.translations.len(), 1);
        assert_eq!(t.get(Lang::Es), "Hola");
        assert_eq!(t.get(Lang::Fr), "Hello");
    }

    #[test]
    fn archive_round_trip() {
        let dex = Pokedex {
            format_version: FORMAT_VERSION,
            pokemon: vec![
                Pokemon {
                    id: 1,
                    name: "bulbasaur".into(),
                    types: vec![PokemonType::Grass, PokemonType::Poison],
                    sprites: SpriteKind::Front as u16 | SpriteKind::Back as u16,
                    ..Default::default()
                },
                Pokemon {
                    id: 25,
                    name: "pikachu".into(),
                    types: vec![PokemonType::Electric],
                    stats: Stats {
                        hp: 35,
                        attack: 55,
                        defense: 40,
                        sp_attack: 50,
                        sp_defense: 50,
                        speed: 90,
                    },
                    ..Default::default()
                },
            ],
            type_chart: {
                let mut chart = vec![2u8; 18 * 18];
                chart[PokemonType::Electric.index() * 18 + PokemonType::Ground.index()] = 0;
                chart
            },
            ..Default::default()
        };
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&dex).unwrap();
        let archived = rkyv::access::<ArchivedPokedex, rkyv::rancor::Error>(&bytes).unwrap();
        let pika = archived.pokemon(25).unwrap();
        assert_eq!(pika.stats.total(), 320);
        assert!(pika.has_type(PokemonType::Electric));
        assert!(archived.pokemon(2).is_none());
        let bulba = archived.pokemon(1).unwrap();
        assert_eq!(
            bulba.sprite_path(SpriteKind::Back).as_deref(),
            Some("sprites/pokemon/1/back.png")
        );
        assert!(bulba.sprite_path(SpriteKind::FrontShiny).is_none());
        assert_eq!(
            archived.effectiveness(
                PokemonType::Electric,
                &[PokemonType::Water, PokemonType::Ground]
            ),
            0.0
        );
    }
}
