// SPDX-License-Identifier: GPL-3.0-only

//! Small fieldless enums.
//!
//! Every enum here is `#[repr(u8)]` and archived *as itself* (`#[rkyv(as = Self)]`), so the
//! archived value inside the memory map is the very same Rust type the app works with. No
//! conversion or deserialization is needed to compare, hash or match on them.

use rkyv::{Archive, Deserialize, Portable, Serialize, bytecheck::CheckBytes};

/// Declares a portable fieldless enum together with an `ALL` list and an `api_name` mapping
/// to/from the identifiers used by PokéAPI.
macro_rules! portable_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident { $( $variant:ident => $api:literal ),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(
            Archive, Serialize, Deserialize, Portable, CheckBytes,
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord,
        )]
        #[rkyv(as = $name)]
        #[bytecheck(crate = ::rkyv::bytecheck)]
        #[repr(u8)]
        pub enum $name { $( $variant ),+ }

        impl $name {
            /// Every variant, in declaration order.
            pub const ALL: &'static [Self] = &[ $( Self::$variant ),+ ];

            /// The identifier PokéAPI uses for this value.
            pub const fn api_name(self) -> &'static str {
                match self { $( Self::$variant => $api ),+ }
            }

            /// Parses a PokéAPI identifier.
            pub fn from_api(name: &str) -> Option<Self> {
                match name { $( $api => Some(Self::$variant), )+ _ => None }
            }

            /// Position of the variant inside [`Self::ALL`].
            pub const fn index(self) -> usize {
                self as usize
            }
        }
    };
}

portable_enum! {
    /// The 18 elemental types.
    pub enum PokemonType {
        Normal => "normal",
        Fire => "fire",
        Water => "water",
        Electric => "electric",
        Grass => "grass",
        Ice => "ice",
        Fighting => "fighting",
        Poison => "poison",
        Ground => "ground",
        Flying => "flying",
        Psychic => "psychic",
        Bug => "bug",
        Rock => "rock",
        Ghost => "ghost",
        Dragon => "dragon",
        Dark => "dark",
        Steel => "steel",
        Fairy => "fairy",
    }
}

portable_enum! {
    /// Game generation a species (or move, ability...) was introduced in.
    pub enum Generation {
        One => "generation-i",
        Two => "generation-ii",
        Three => "generation-iii",
        Four => "generation-iv",
        Five => "generation-v",
        Six => "generation-vi",
        Seven => "generation-vii",
        Eight => "generation-viii",
        Nine => "generation-ix",
        Unknown => "unknown",
    }
}

impl Generation {
    /// 1-based generation number (0 for [`Generation::Unknown`]).
    pub const fn number(self) -> u8 {
        match self {
            Self::Unknown => 0,
            other => other as u8 + 1,
        }
    }
}

portable_enum! {
    /// Whether a move uses Attack/Defense, Sp. Atk/Sp. Def or deals no direct damage.
    pub enum DamageClass {
        Physical => "physical",
        Special => "special",
        Status => "status",
    }
}

portable_enum! {
    /// How a Pokémon learns a move in a given game.
    pub enum LearnMethod {
        LevelUp => "level-up",
        Machine => "machine",
        Egg => "egg",
        Tutor => "tutor",
        FormChange => "form-change",
        Other => "other",
    }
}

impl LearnMethod {
    /// Like [`LearnMethod::from_api`] but maps every unknown/rare method to [`LearnMethod::Other`].
    pub fn from_api_lossy(name: &str) -> Self {
        Self::from_api(name).unwrap_or(Self::Other)
    }
}

portable_enum! {
    /// Experience curve of a species.
    pub enum GrowthRate {
        Slow => "slow",
        Medium => "medium",
        Fast => "fast",
        MediumSlow => "medium-slow",
        Erratic => "slow-then-very-fast",
        Fluctuating => "fast-then-very-slow",
    }
}

impl GrowthRate {
    /// Total experience required to reach level 100.
    pub const fn exp_at_100(self) -> u32 {
        match self {
            Self::Slow => 1_250_000,
            Self::Medium => 1_000_000,
            Self::Fast => 800_000,
            Self::MediumSlow => 1_059_860,
            Self::Erratic => 600_000,
            Self::Fluctuating => 1_640_000,
        }
    }
}

portable_enum! {
    /// Breeding compatibility groups.
    pub enum EggGroup {
        Monster => "monster",
        Water1 => "water1",
        Bug => "bug",
        Flying => "flying",
        Field => "ground",
        Fairy => "fairy",
        Grass => "plant",
        HumanLike => "humanshape",
        Water3 => "water3",
        Mineral => "mineral",
        Amorphous => "indeterminate",
        Water2 => "water2",
        Ditto => "ditto",
        Dragon => "dragon",
        Undiscovered => "no-eggs",
    }
}

portable_enum! {
    /// Battle statistics (accuracy/evasion only appear in move stat changes).
    pub enum Stat {
        Hp => "hp",
        Attack => "attack",
        Defense => "defense",
        SpAttack => "special-attack",
        SpDefense => "special-defense",
        Speed => "speed",
        Accuracy => "accuracy",
        Evasion => "evasion",
    }
}

portable_enum! {
    /// Languages kept in the bundled data. PokéAPI coverage varies a lot per language; the
    /// app always falls back to English when a translation is missing.
    pub enum Lang {
        En => "en",
        Es => "es",
        Fr => "fr",
        De => "de",
        It => "it",
        Ja => "ja",
        Ko => "ko",
        ZhHans => "zh-hans",
        ZhHant => "zh-hant",
        Cs => "cs",
        PtBr => "pt-br",
    }
}

impl Lang {
    /// Maps a BCP-47-ish locale (`es-ES`, `pt_BR`, `zh-Hans`...) to the closest bundled language.
    pub fn from_locale(locale: &str) -> Self {
        let l = locale.to_ascii_lowercase().replace('_', "-");
        if let Some(lang) = Self::from_api(&l) {
            return lang;
        }
        if l.starts_with("zh") {
            return if l.contains("hant") || l.contains("tw") || l.contains("hk") {
                Self::ZhHant
            } else {
                Self::ZhHans
            };
        }
        if l.starts_with("pt") {
            return Self::PtBr;
        }
        let primary = l.split('-').next().unwrap_or("en");
        Self::from_api(primary).unwrap_or(Self::En)
    }
}

/// Bit flags describing which sprite files exist for a Pokémon
/// (`sprites/pokemon/{id}/{file}`). Stored as a plain `u16` in the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SpriteKind {
    Front = 1 << 0,
    FrontShiny = 1 << 1,
    Back = 1 << 2,
    BackShiny = 1 << 3,
    FrontFemale = 1 << 4,
    FrontShinyFemale = 1 << 5,
    BackFemale = 1 << 6,
    BackShinyFemale = 1 << 7,
}

impl SpriteKind {
    pub const ALL: &'static [Self] = &[
        Self::Front,
        Self::FrontShiny,
        Self::Back,
        Self::BackShiny,
        Self::FrontFemale,
        Self::FrontShinyFemale,
        Self::BackFemale,
        Self::BackShinyFemale,
    ];

    /// File name of this sprite inside `sprites/pokemon/{id}/`.
    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Front => "front.png",
            Self::FrontShiny => "front_shiny.png",
            Self::Back => "back.png",
            Self::BackShiny => "back_shiny.png",
            Self::FrontFemale => "front_female.png",
            Self::FrontShinyFemale => "front_shiny_female.png",
            Self::BackFemale => "back_female.png",
            Self::BackShinyFemale => "back_shiny_female.png",
        }
    }

    /// Picks the sprite variant for the given view options.
    pub const fn select(back: bool, shiny: bool, female: bool) -> Self {
        match (back, shiny, female) {
            (false, false, false) => Self::Front,
            (false, true, false) => Self::FrontShiny,
            (true, false, false) => Self::Back,
            (true, true, false) => Self::BackShiny,
            (false, false, true) => Self::FrontFemale,
            (false, true, true) => Self::FrontShinyFemale,
            (true, false, true) => Self::BackFemale,
            (true, true, true) => Self::BackShinyFemale,
        }
    }
}

/// Bit flags describing which cry files exist (`cries/{id}/{file}`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CryKind {
    Latest = 1 << 0,
    Legacy = 1 << 1,
}

impl CryKind {
    pub const fn file_name(self) -> &'static str {
        match self {
            Self::Latest => "latest.ogg",
            Self::Legacy => "legacy.ogg",
        }
    }
}

#[allow(clippy::derivable_impls)] // the variants are generated by a macro
impl Default for Generation {
    fn default() -> Self {
        Self::Unknown
    }
}

#[allow(clippy::derivable_impls)] // the variants are generated by a macro
impl Default for GrowthRate {
    fn default() -> Self {
        Self::Medium
    }
}
