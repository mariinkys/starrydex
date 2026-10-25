// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, text};
use cosmic::{Element, theme};

use super::{Ctx, Layout, grid};
use crate::app::pages::common::{card, empty};
use crate::app::{DetailsInput, Message};
use crate::fl;

/// Below this card width the game name goes above its encounter details.
const SIDE_BY_SIDE_MIN: f32 = 560.0;

pub fn view<'a>(ctx: Ctx<'a>, layout: Layout) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let Ctx {
        dex,
        pokemon,
        details,
        lang,
        ..
    } = ctx;

    if pokemon.encounters.is_empty() {
        return card(fl!("tab-locations"), empty(fl!("no-encounter-info")));
    }

    let mut labels = vec![fl!("all-games")];
    labels.extend(details.encounter_versions.iter().map(|id| {
        dex.version(*id)
            .map_or_else(|| fl!("unknown"), |v| v.display.get(lang).to_string())
    }));
    let selected = details
        .encounter_version
        .and_then(|v| details.encounter_versions.iter().position(|id| *id == v))
        .map_or(0, |i| i + 1);

    let picker = widget::row::with_children(vec![
        text::heading(fl!("game")).into(),
        widget::dropdown(Cow::Owned(labels), Some(selected), |i| {
            Message::Details(DetailsInput::EncounterVersion(i))
        })
        .into(),
    ])
    .spacing(8)
    .align_y(Alignment::Center);

    // Sort areas by location name for a stable, readable list.
    let mut encounters: Vec<_> = pokemon
        .encounters
        .iter()
        .filter_map(|encounter| {
            let area = dex.location_area(encounter.location_area_id.to_native())?;
            let location = dex.location(area.location_id.to_native());
            let versions: Vec<_> = encounter
                .versions
                .iter()
                .filter(|v| {
                    details
                        .encounter_version
                        .is_none_or(|selected| v.version_id.to_native() == selected)
                })
                .collect();
            (!versions.is_empty()).then_some((area, location, versions))
        })
        .collect();
    encounters.sort_by_key(|(_, location, _)| location.map_or("", |l| l.display.get(lang)));

    // At most two columns: encounter lines are long and read badly when squeezed.
    let columns = layout.columns.min(2);
    let side_by_side = layout.card_width(columns, f32::from(spacing.space_s)) >= SIDE_BY_SIDE_MIN;

    let mut cards: Vec<Element<'a, Message>> = Vec::new();
    for (area, location, versions) in encounters {
        let location_name = location.map_or_else(|| fl!("unknown"), |l| l.display.get(lang).to_string());
        let area_name = area.display.get(lang);
        let title = if area_name.is_empty() || area_name == location_name {
            location_name
        } else {
            format!("{location_name} — {area_name}")
        };
        let region = location
            .and_then(|l| l.region_id.as_ref())
            .and_then(|id| dex.region(id.to_native()))
            .map(|r| r.display.get(lang).to_string());

        let mut rows: Vec<Element<'a, Message>> = Vec::new();
        if let Some(region) = region {
            rows.push(text::caption(region).into());
        }
        for v in versions {
            let version = dex
                .version(v.version_id.to_native())
                .map_or_else(|| fl!("unknown"), |x| x.display.get(lang).to_string());
            let methods: Vec<&str> = v
                .methods
                .iter()
                .filter_map(|id| dex.encounter_method(id.to_native()))
                .map(|m| m.display.get(lang))
                .collect();
            let conditions: Vec<&str> = v
                .conditions
                .iter()
                .filter_map(|id| dex.encounter_condition(id.to_native()))
                .map(|c| c.display.get(lang))
                .collect();
            let levels = if v.min_level == v.max_level {
                fl!("level-short", level = v.min_level)
            } else {
                format!("{}–{}", fl!("level-short", level = v.min_level), v.max_level)
            };
            let mut line = vec![methods.join(", "), levels, format!("{}%", v.max_chance)];
            if !conditions.is_empty() {
                line.push(conditions.join(", "));
            }
            let details_text = text::body(line.join(" · "));

            rows.push(if side_by_side {
                widget::row::with_children(vec![
                    text::heading(version).width(Length::FillPortion(2)).into(),
                    details_text.width(Length::FillPortion(5)).into(),
                ])
                .spacing(8)
                .into()
            } else {
                widget::column::with_children(vec![
                    text::heading(version).into(),
                    details_text.width(Length::Fill).into(),
                ])
                .spacing(2)
                .into()
            });
        }
        cards.push(card(title, widget::column::with_children(rows).spacing(6)));
    }

    let body = if cards.is_empty() {
        empty(fl!("no-encounter-info"))
    } else {
        grid(cards, columns, spacing.space_s)
    };

    widget::column::with_children(vec![picker.into(), body])
        .spacing(spacing.space_s)
        .width(Length::Fill)
        .into()
}
