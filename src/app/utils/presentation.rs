// SPDX-License-Identifier: GPL-3.0-only

//! Turning data into user-facing strings and colors.

use std::sync::OnceLock;

use cosmic::iced::Color;
use starrydex_model::{
    ArchivedEvolutionMethod, ArchivedPokedex, DamageClass, EggGroup, Generation, GrowthRate, Lang,
    LearnMethod, PokemonType, Stat,
};

use crate::app::utils::{Category, SortBy};
use crate::fl;

/// Language used for Pokédex data (names, descriptions), derived from the UI language.
pub fn data_lang() -> Lang {
    static LANG: OnceLock<Lang> = OnceLock::new();
    *LANG.get_or_init(|| {
        crate::i18n::LANGUAGE_LOADER
            .current_languages()
            .first()
            .map(|l| Lang::from_locale(&l.to_string()))
            .unwrap_or(Lang::En)
    })
}

/// Transforms a kebab-case string into a space-separated string where each word starts with an uppercase letter.
pub fn capitalize_string(input: &str) -> String {
    input
        .split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// "urban" → "Urban" (some PokéAPI names are lowercase).
pub fn uppercase_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// "#0025"
pub fn dex_number(number: u32) -> String {
    format!("#{number:04}")
}

/// Decimetres → "1.7 m (5′07″)"
pub fn format_height(decimetres: u16) -> String {
    let metres = f32::from(decimetres) / 10.0;
    let total_inches = (f32::from(decimetres) * 3.937_008).round() as u32;
    format!(
        "{metres:.1} m ({}′{:02}″)",
        total_inches / 12,
        total_inches % 12
    )
}

/// Hectograms → "90.5 kg (199.5 lbs)"
pub fn format_weight(hectograms: u16) -> String {
    let kg = f32::from(hectograms) / 10.0;
    format!("{kg:.1} kg ({:.1} lbs)", kg * 2.204_623)
}

/// Approximate chance (%) of catching a full-HP Pokémon with a Poké Ball (gen III/IV formula).
pub fn capture_chance(capture_rate: u8) -> f32 {
    let a = f64::from(capture_rate) / 3.0;
    if a <= 0.0 {
        return 0.0;
    }
    if a >= 255.0 {
        return 100.0;
    }
    let b = 1_048_560.0 / (16_711_680.0 / a).sqrt().sqrt();
    ((b / 65_535.0).powi(4) * 100.0) as f32
}

/// Gender ratio as (male %, female %), or `None` for genderless species.
pub fn gender_split(gender_rate: i8) -> Option<(f32, f32)> {
    (gender_rate >= 0).then(|| {
        let female = f32::from(gender_rate) * 12.5;
        (100.0 - female, female)
    })
}

pub fn type_color(t: PokemonType) -> Color {
    match t {
        PokemonType::Normal => Color::from_rgb8(160, 162, 159),
        PokemonType::Fire => Color::from_rgb8(251, 165, 76),
        PokemonType::Water => Color::from_rgb8(83, 157, 223),
        PokemonType::Electric => Color::from_rgb8(242, 217, 78),
        PokemonType::Grass => Color::from_rgb8(95, 189, 88),
        PokemonType::Ice => Color::from_rgb8(117, 208, 193),
        PokemonType::Fighting => Color::from_rgb8(211, 66, 95),
        PokemonType::Poison => Color::from_rgb8(183, 99, 207),
        PokemonType::Ground => Color::from_rgb8(218, 124, 77),
        PokemonType::Flying => Color::from_rgb8(161, 187, 236),
        PokemonType::Psychic => Color::from_rgb8(250, 133, 129),
        PokemonType::Bug => Color::from_rgb8(146, 188, 44),
        PokemonType::Rock => Color::from_rgb8(201, 187, 138),
        PokemonType::Ghost => Color::from_rgb8(95, 109, 188),
        PokemonType::Dragon => Color::from_rgb8(12, 105, 200),
        PokemonType::Dark => Color::from_rgb8(89, 87, 97),
        PokemonType::Steel => Color::from_rgb8(86, 149, 163),
        PokemonType::Fairy => Color::from_rgb8(238, 144, 230),
    }
}

pub fn type_icon_name(t: PokemonType) -> String {
    format!("type-{}", t.api_name())
}

/// SVG style that tints a symbolic icon with a type's color.
pub fn type_svg_class(t: PokemonType) -> cosmic::theme::Svg {
    let color = type_color(t);
    cosmic::theme::Svg::Custom(std::rc::Rc::new(move |_| cosmic::widget::svg::Style {
        color: Some(color),
    }))
}

/// Localized label for data enums.
pub trait Label {
    fn label(&self) -> String;
}

impl Label for PokemonType {
    fn label(&self) -> String {
        match self {
            Self::Normal => fl!("normal"),
            Self::Fire => fl!("fire"),
            Self::Water => fl!("water"),
            Self::Electric => fl!("electric"),
            Self::Grass => fl!("grass"),
            Self::Ice => fl!("ice"),
            Self::Fighting => fl!("fighting"),
            Self::Poison => fl!("poison"),
            Self::Ground => fl!("ground"),
            Self::Flying => fl!("flying"),
            Self::Psychic => fl!("psychic"),
            Self::Bug => fl!("bug"),
            Self::Rock => fl!("rock"),
            Self::Ghost => fl!("ghost"),
            Self::Dragon => fl!("dragon"),
            Self::Dark => fl!("dark"),
            Self::Steel => fl!("steel"),
            Self::Fairy => fl!("fairy"),
        }
    }
}

impl Label for Generation {
    fn label(&self) -> String {
        match self {
            Self::Unknown => fl!("unknown"),
            Self::One => fl!("gen-i"),
            Self::Two => fl!("gen-ii"),
            Self::Three => fl!("gen-iii"),
            Self::Four => fl!("gen-iv"),
            Self::Five => fl!("gen-v"),
            Self::Six => fl!("gen-vi"),
            Self::Seven => fl!("gen-vii"),
            Self::Eight => fl!("gen-viii"),
            Self::Nine => fl!("gen-ix"),
        }
    }
}

impl Label for DamageClass {
    fn label(&self) -> String {
        match self {
            Self::Physical => fl!("physical"),
            Self::Special => fl!("special"),
            Self::Status => fl!("status"),
        }
    }
}

impl Label for LearnMethod {
    fn label(&self) -> String {
        match self {
            Self::LevelUp => fl!("level-up"),
            Self::Machine => fl!("tm"),
            Self::Egg => fl!("egg"),
            Self::Tutor => fl!("tutor"),
            Self::FormChange => fl!("form-change"),
            Self::Other => fl!("other-method"),
        }
    }
}

impl Label for GrowthRate {
    fn label(&self) -> String {
        match self {
            Self::Slow => fl!("growth-slow"),
            Self::Medium => fl!("growth-medium"),
            Self::Fast => fl!("growth-fast"),
            Self::MediumSlow => fl!("growth-medium-slow"),
            Self::Erratic => fl!("growth-erratic"),
            Self::Fluctuating => fl!("growth-fluctuating"),
        }
    }
}

impl Label for EggGroup {
    fn label(&self) -> String {
        match self {
            Self::Monster => fl!("egg-monster"),
            Self::Water1 => fl!("egg-water1"),
            Self::Bug => fl!("egg-bug"),
            Self::Flying => fl!("egg-flying"),
            Self::Field => fl!("egg-field"),
            Self::Fairy => fl!("egg-fairy"),
            Self::Grass => fl!("egg-grass"),
            Self::HumanLike => fl!("egg-human-like"),
            Self::Water3 => fl!("egg-water3"),
            Self::Mineral => fl!("egg-mineral"),
            Self::Amorphous => fl!("egg-amorphous"),
            Self::Water2 => fl!("egg-water2"),
            Self::Ditto => fl!("egg-ditto"),
            Self::Dragon => fl!("egg-dragon"),
            Self::Undiscovered => fl!("egg-undiscovered"),
        }
    }
}

impl Label for Stat {
    fn label(&self) -> String {
        match self {
            Self::Hp => fl!("hp"),
            Self::Attack => fl!("attack"),
            Self::Defense => fl!("defense"),
            Self::SpAttack => fl!("sp-a"),
            Self::SpDefense => fl!("sp-d"),
            Self::Speed => fl!("spd"),
            Self::Accuracy => fl!("accuracy"),
            Self::Evasion => fl!("evasion"),
        }
    }
}

/// Short stat labels for compact tables and charts.
pub fn stat_short(stat: Stat) -> String {
    match stat {
        Stat::Hp => fl!("hp"),
        Stat::Attack => fl!("atk-short"),
        Stat::Defense => fl!("def-short"),
        Stat::SpAttack => fl!("spa-short"),
        Stat::SpDefense => fl!("spd-short"),
        Stat::Speed => fl!("spe-short"),
        Stat::Accuracy => fl!("accuracy"),
        Stat::Evasion => fl!("evasion"),
    }
}

impl Label for Category {
    fn label(&self) -> String {
        match self {
            Self::Regular => fl!("category-regular"),
            Self::Baby => fl!("category-baby"),
            Self::Legendary => fl!("category-legendary"),
            Self::Mythical => fl!("category-mythical"),
        }
    }
}

impl Label for SortBy {
    fn label(&self) -> String {
        match self {
            Self::Number => fl!("sort-number"),
            Self::Name => fl!("sort-name"),
            Self::Total => fl!("total"),
            Self::Hp => fl!("hp"),
            Self::Attack => fl!("attack"),
            Self::Defense => fl!("defense"),
            Self::SpAttack => fl!("sp-a"),
            Self::SpDefense => fl!("sp-d"),
            Self::Speed => fl!("spd"),
            Self::Height => fl!("height-label"),
            Self::Weight => fl!("weight-label"),
        }
    }
}

/// Human readable description of one way to evolve, e.g. "Level 16", "Use Fire Stone",
/// "Trade · Holding Metal Coat", "Level up · Friendship 160+ · Daytime".
pub fn describe_evolution(
    dex: &ArchivedPokedex,
    method: &ArchivedEvolutionMethod,
    lang: Lang,
) -> String {
    let item_name = |id: u32| {
        dex.item(id)
            .map_or_else(|| fl!("unknown"), |i| i.display.get(lang).to_string())
    };
    let move_name = |id: u32| {
        dex.move_(id)
            .map_or_else(|| fl!("unknown"), |m| m.display.get(lang).to_string())
    };
    let species_name = |id: u32| {
        dex.species(id)
            .map_or_else(|| fl!("unknown"), |s| s.display.get(lang).to_string())
    };

    let trigger = dex.evolution_trigger(method.trigger_id.to_native());
    let mut parts: Vec<String> = Vec::new();

    match trigger.map(|t| t.name.as_str()).unwrap_or("") {
        "level-up" => match method.min_level.as_ref() {
            Some(level) => parts.push(fl!("evo-level", level = (*level))),
            None => parts.push(fl!("evo-level-up")),
        },
        "trade" => match method.trade_species_id.as_ref() {
            Some(id) => parts.push(fl!("evo-trade-for", species = species_name(id.to_native()))),
            None => parts.push(fl!("evo-trade")),
        },
        // The item itself is described below.
        "use-item" => {}
        other => {
            let name = trigger
                .map(|t| t.display.get(lang).to_string())
                .unwrap_or_else(|| capitalize_string(other));
            if !name.is_empty() {
                parts.push(name);
            }
        }
    }
    if let Some(id) = method.item_id.as_ref() {
        parts.push(fl!("evo-use-item", item = item_name(id.to_native())));
    }
    if let Some(id) = method.held_item_id.as_ref() {
        parts.push(fl!("evo-holding", item = item_name(id.to_native())));
    }
    if let Some(v) = method.min_happiness.as_ref() {
        parts.push(fl!("evo-happiness", value = (*v)));
    }
    if let Some(v) = method.min_affection.as_ref() {
        parts.push(fl!("evo-affection", value = (*v)));
    }
    if let Some(v) = method.min_beauty.as_ref() {
        parts.push(fl!("evo-beauty", value = (*v)));
    }
    match method.time_of_day.as_str() {
        "day" => parts.push(fl!("evo-day")),
        "night" => parts.push(fl!("evo-night")),
        "dusk" => parts.push(fl!("evo-dusk")),
        "" => {}
        other => parts.push(capitalize_string(other)),
    }
    match method.gender.as_ref() {
        Some(1) => parts.push(fl!("evo-female")),
        Some(2) => parts.push(fl!("evo-male")),
        _ => {}
    }
    if let Some(id) = method.known_move_id.as_ref() {
        parts.push(fl!("evo-knowing", name = move_name(id.to_native())));
    }
    if let Some(t) = method.known_move_type.as_ref() {
        parts.push(fl!("evo-knowing-type", name = t.label()));
    }
    if let Some(id) = method.location_id.as_ref() {
        let place = dex
            .location(id.to_native())
            .map_or_else(|| fl!("unknown"), |l| l.display.get(lang).to_string());
        parts.push(fl!("evo-at", place = place));
    }
    if let Some(id) = method.party_species_id.as_ref() {
        parts.push(fl!(
            "evo-party-species",
            name = species_name(id.to_native())
        ));
    }
    if let Some(t) = method.party_type.as_ref() {
        parts.push(fl!("evo-party-type", name = t.label()));
    }
    match method.relative_physical_stats.as_ref() {
        Some(1) => parts.push(fl!("evo-atk-gt-def")),
        Some(-1) => parts.push(fl!("evo-atk-lt-def")),
        Some(0) => parts.push(fl!("evo-atk-eq-def")),
        _ => {}
    }
    if let Some(id) = method.used_move_id.as_ref() {
        let count = method.min_move_count.as_ref().map_or(1, |c| c.to_native());
        parts.push(fl!(
            "evo-used-move",
            name = move_name(id.to_native()),
            count = count
        ));
    }
    if let Some(steps) = method.min_steps.as_ref() {
        parts.push(fl!("evo-steps", count = steps.to_native()));
    }
    if let Some(damage) = method.min_damage_taken.as_ref() {
        parts.push(fl!("evo-damage", count = damage.to_native()));
    }
    if method.needs_overworld_rain {
        parts.push(fl!("evo-rain"));
    }
    if method.turn_upside_down {
        parts.push(fl!("evo-upside-down"));
    }
    if method.needs_multiplayer {
        parts.push(fl!("evo-multiplayer"));
    }
    if let Some(chance) = method.chance.as_ref() {
        parts.push(fl!("evo-chance", value = (*chance)));
    }

    if parts.is_empty() {
        fl!("unknown")
    } else {
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_measurements() {
        assert_eq!(format_height(7), "0.7 m (2′04″)");
        assert_eq!(format_weight(69), "6.9 kg (15.2 lbs)");
        assert_eq!(dex_number(25), "#0025");
    }

    #[test]
    fn capture_chance_matches_known_values() {
        // Rate 255 at full HP with a Poké Ball is ~33.3%
        assert!((capture_chance(255) - 33.3).abs() < 0.5);
        assert!(capture_chance(3) < 2.0);
    }

    #[test]
    fn gender() {
        assert_eq!(gender_split(-1), None);
        assert_eq!(gender_split(1), Some((87.5, 12.5)));
    }
}
