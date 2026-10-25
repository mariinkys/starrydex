// SPDX-License-Identifier: GPL-3.0-only

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedEvolutionNode, SpriteKind, item_sprite_rel_path};

use super::{Ctx, Layout};
use crate::app::Message;
use crate::app::pages::common::{card, empty, sprite};
use crate::app::utils::presentation::{describe_evolution, dex_number};
use crate::{fl, icons};

/// Space taken by an arrow between two stages (icon + row spacing on both sides).
const ARROW_WIDTH: f32 = 48.0;

pub fn view<'a>(ctx: Ctx<'a>, layout: Layout) -> Element<'a, Message> {
    let Ctx {
        core,
        dex,
        species,
        lang,
        ..
    } = ctx;
    let chain = species
        .and_then(|s| s.evolution_chain_id.as_ref())
        .and_then(|id| dex.evolution_chain(id.to_native()));
    let Some(chain) = chain else {
        return card(fl!("poke-evo-data"), empty(fl!("no-evo-data")));
    };

    let card_width = if layout.compact() { 150.0 } else { 190.0 };
    let max_stage = chain.nodes.iter().map(|n| n.stage).max().unwrap_or(0);
    let stage_cards = |stage: u8| -> Vec<Element<'a, Message>> {
        chain
            .nodes
            .iter()
            .filter(|n| n.stage == stage)
            .map(|node| node_card(ctx, node, card_width))
            .collect()
    };
    let widest = (0..=max_stage)
        .map(|stage| chain.nodes.iter().filter(|n| n.stage == stage).count())
        .max()
        .unwrap_or(0);

    // Left to right when the whole chain fits on one line (and branches are few),
    // otherwise top to bottom with each stage wrapping as needed.
    let stages = f32::from(max_stage) + 1.0;
    let horizontal_width = stages * card_width + (stages - 1.0) * ARROW_WIDTH;
    let horizontal = widest <= 2 && horizontal_width <= layout.width - 32.0;

    let tree: Element<'a, Message> = if horizontal {
        let mut row: Vec<Element<'a, Message>> = Vec::new();
        for stage in 0..=max_stage {
            if stage > 0 {
                row.push(
                    widget::icon(icons::get_handle("go-next-symbolic", 18))
                        .size(24)
                        .into(),
                );
            }
            row.push(
                widget::column::with_children(stage_cards(stage))
                    .spacing(8)
                    .align_x(Alignment::Center)
                    .into(),
            );
        }
        container(
            widget::row::with_children(row)
                .spacing(12)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    } else {
        let mut column: Vec<Element<'a, Message>> = Vec::new();
        for stage in 0..=max_stage {
            if stage > 0 {
                column.push(
                    widget::icon(icons::get_handle("go-down-symbolic", 18))
                        .size(24)
                        .into(),
                );
            }
            let cards = stage_cards(stage);
            column.push(if cards.len() == 1 {
                container(widget::column::with_children(cards))
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .into()
            } else {
                widget::flex_row(cards)
                    .row_spacing(8)
                    .column_spacing(8)
                    .justify_content(widget::JustifyContent::Center)
                    .width(Length::Fill)
                    .into()
            });
        }
        widget::column::with_children(column)
            .spacing(8)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .into()
    };

    let mut content = vec![tree];
    if let Some(item) = chain
        .baby_trigger_item_id
        .as_ref()
        .and_then(|id| dex.item(id.to_native()))
    {
        let mut row = vec![];
        if item.has_sprite {
            row.push(sprite(
                Some(core.asset(&item_sprite_rel_path(item.name.as_str()))),
                30.0,
            ));
        }
        row.push(
            text::body(fl!("baby-trigger", item = item.display.get(lang)))
                .width(Length::Fill)
                .into(),
        );
        content.push(
            widget::row::with_children(row)
                .spacing(6)
                .align_y(Alignment::Center)
                .into(),
        );
    }

    card(
        fl!("poke-evo-data"),
        widget::column::with_children(content).spacing(12),
    )
}

fn node_card<'a>(ctx: Ctx<'a>, node: &'a ArchivedEvolutionNode, width: f32) -> Element<'a, Message> {
    let Ctx {
        core,
        dex,
        species: current,
        lang,
        ..
    } = ctx;
    let species_id = node.species_id.to_native();
    let species = dex.species(species_id);
    let pokemon_id = species
        .and_then(|s| s.varieties.first())
        .map_or(species_id, |id| id.to_native());
    let pokemon = dex.pokemon(pokemon_id);
    let is_current = current.is_some_and(|s| s.id.to_native() == species_id);

    let name = species.map_or("?", |s| s.display.get(lang));
    let sprite_size = if width < 170.0 { 64.0 } else { 80.0 };
    let mut column: Vec<Element<'a, Message>> = vec![
        sprite(
            pokemon.and_then(|p| core.sprite(p, SpriteKind::Front)),
            sprite_size,
        ),
        text::caption(dex_number(species_id)).into(),
        text::heading(name).into(),
    ];
    for method in node.methods.iter() {
        column.push(
            text::caption(describe_evolution(dex, method, lang))
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .into(),
        );
    }

    button::custom(
        widget::column::with_children(column)
            .spacing(2)
            .align_x(Alignment::Center)
            .width(Length::Fill),
    )
    .width(Length::Fixed(width))
    .padding(8)
    .selected(is_current)
    .class(theme::Button::IconVertical)
    .on_press_maybe(
        (!is_current && pokemon.is_some()).then_some(Message::OpenPokemon(pokemon_id)),
    )
    .into()
}
