// SPDX-License-Identifier: GPL-3.0-only

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, text};
use cosmic::{Element, theme};
use starrydex_model::{PokemonType, stat_range_at_100};

use super::{Ctx, Layout};
use crate::app::Message;
use crate::app::pages::common::{card, type_badge};
use crate::app::utils::presentation::{Label, stat_short};
use crate::app::widgets::barchart::BarChart;
use crate::fl;

pub fn view<'a>(ctx: Ctx<'a>, layout: Layout) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;

    let mut side = vec![type_defenses(ctx)];
    if !ctx.pokemon.past_types.is_empty() {
        side.push(past_types(ctx, layout));
    }

    if layout.narrow() {
        let mut column = vec![base_stats(ctx, layout)];
        column.extend(side);
        widget::column::with_children(column)
            .spacing(spacing.space_s)
            .width(Length::Fill)
            .into()
    } else {
        // Stats get the larger share; matchups sit beside them instead of below.
        let (main, aside) = if layout.columns >= 3 { (3, 2) } else { (1, 1) };
        widget::row::with_children(vec![
            widget::container(base_stats(ctx, layout))
                .width(Length::FillPortion(main))
                .into(),
            widget::column::with_children(side)
                .spacing(spacing.space_s)
                .width(Length::FillPortion(aside))
                .into(),
        ])
        .spacing(spacing.space_s)
        .align_y(Alignment::Start)
        .into()
    }
}

fn base_stats(ctx: Ctx<'_>, layout: Layout) -> Element<'_, Message> {
    let pokemon = ctx.pokemon;
    let stats = pokemon.stats.values();
    let compact = layout.compact();

    let chart = stats
        .iter()
        .fold(
            BarChart::new()
                .column_spacing(3.)
                .padding(3.)
                .height(Length::Fixed(if compact { 180.0 } else { 260.0 })),
            |chart, (stat, value)| chart.push(stat_short(*stat), f32::from(*value)),
        );

    // Table: stat | base | min | max | (EVs, hidden when compact)
    let mut rows = vec![row(
        fl!("stat"),
        fl!("base"),
        fl!("min-100"),
        fl!("max-100"),
        (!compact).then(|| fl!("ev-yield")),
        true,
    )];
    for ((stat, base), (_, ev)) in stats.iter().zip(pokemon.ev_yield.values()) {
        let (min, max) = stat_range_at_100(*stat, *base, pokemon.name.as_str());
        let name = if compact { stat_short(*stat) } else { stat.label() };
        rows.push(row(
            name,
            base.to_string(),
            min.to_string(),
            max.to_string(),
            (!compact).then(|| if ev > 0 { format!("+{ev}") } else { String::new() }),
            false,
        ));
    }
    rows.push(row(
        fl!("total"),
        pokemon.stats.total().to_string(),
        String::new(),
        String::new(),
        (!compact).then(String::new),
        true,
    ));

    card(
        fl!("poke-stats"),
        widget::column::with_children(vec![
            chart.into(),
            widget::column::with_children(rows).spacing(4).into(),
            text::caption(fl!("stat-range-hint")).into(),
        ])
        .spacing(10),
    )
}

fn row<'a>(
    stat: String,
    base: String,
    min: String,
    max: String,
    ev: Option<String>,
    bold: bool,
) -> Element<'a, Message> {
    let cell = |value: String, width: Length| -> Element<'a, Message> {
        if bold {
            text::heading(value).width(width).into()
        } else {
            text::body(value).width(width).into()
        }
    };
    let mut cells = vec![
        cell(stat, Length::FillPortion(3)),
        cell(base, Length::FillPortion(1)),
        cell(min, Length::FillPortion(1)),
        cell(max, Length::FillPortion(1)),
    ];
    if let Some(ev) = ev {
        cells.push(cell(ev, Length::FillPortion(1)));
    }
    widget::row::with_children(cells).spacing(8).into()
}

/// Damage taken from each attacking type, grouped by multiplier.
fn type_defenses(ctx: Ctx<'_>) -> Element<'_, Message> {
    let types: Vec<PokemonType> = ctx.pokemon.types.iter().copied().collect();
    let groups: [(f32, String); 5] = [
        (4.0, fl!("weak-4x")),
        (2.0, fl!("weak-2x")),
        (0.5, fl!("resist-half")),
        (0.25, fl!("resist-quarter")),
        (0.0, fl!("immune")),
    ];
    let mut rows: Vec<Element<'_, Message>> = Vec::new();
    for (multiplier, label) in groups {
        let matching: Vec<Element<'_, Message>> = PokemonType::ALL
            .iter()
            .filter(|attacker| {
                (ctx.dex.effectiveness(**attacker, &types) - multiplier).abs() < 0.01
            })
            .map(|t| type_badge(*t))
            .collect();
        if matching.is_empty() {
            continue;
        }
        rows.push(
            widget::column::with_children(vec![
                text::heading(label).into(),
                widget::flex_row(matching)
                    .row_spacing(6)
                    .column_spacing(6)
                    .into(),
            ])
            .spacing(6)
            .into(),
        );
    }
    card(
        fl!("type-defenses"),
        widget::column::with_children(rows).spacing(12),
    )
}

fn past_types(ctx: Ctx<'_>, layout: Layout) -> Element<'_, Message> {
    let lines: Vec<Element<'_, Message>> = ctx
        .pokemon
        .past_types
        .iter()
        .map(|past| {
            let label = text::body(fl!("types-until", generation = past.until.label()));
            let badges = widget::row::with_children(
                past.types.iter().map(|t| type_badge(*t)).collect::<Vec<_>>(),
            )
            .spacing(6);
            if layout.compact() {
                widget::column::with_children(vec![label.into(), badges.into()])
                    .spacing(4)
                    .into()
            } else {
                widget::row::with_children(vec![
                    label.width(Length::Fixed(140.0)).into(),
                    badges.into(),
                ])
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
            }
        })
        .collect();
    card(
        fl!("past-types"),
        widget::column::with_children(lines).spacing(6),
    )
}
