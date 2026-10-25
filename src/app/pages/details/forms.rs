// SPDX-License-Identifier: GPL-3.0-only

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, text};
use cosmic::{Element, theme};
use starrydex_model::SpriteKind;

use super::{Ctx, Layout, grid};
use crate::app::Message;
use crate::app::pages::common::{card, empty, sprite, type_icon};
use crate::fl;

pub fn view<'a>(ctx: Ctx<'a>, layout: Layout) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    // Side by side only on large screens; each part wraps its tiles on its own.
    let columns = if layout.columns >= 3 { 2 } else { 1 };
    grid(
        vec![varieties(ctx, layout), gallery(ctx, layout)],
        columns,
        spacing.space_s,
    )
}

/// Every variety of the species (base form, megas, regional forms, Gigantamax...).
fn varieties(ctx: Ctx<'_>, layout: Layout) -> Element<'_, Message> {
    let Ctx {
        core,
        dex,
        pokemon,
        species,
        lang,
        ..
    } = ctx;
    let Some(species) = species else {
        return card(fl!("forms"), empty(fl!("no-forms")));
    };
    let (tile_width, sprite_size) = if layout.compact() {
        (140.0, 72.0)
    } else {
        (180.0, 96.0)
    };

    let cards: Vec<Element<'_, Message>> = species
        .varieties
        .iter()
        .filter_map(|id| dex.pokemon(id.to_native()))
        .map(|variety| {
            let is_current = variety.id == pokemon.id;
            let types = widget::row::with_children(
                variety
                    .types
                    .iter()
                    .map(|t| type_icon(*t, 16))
                    .collect::<Vec<_>>(),
            )
            .spacing(4);
            button::custom(
                widget::column::with_children(vec![
                    sprite(core.sprite(variety, SpriteKind::Front), sprite_size),
                    text::heading(variety.display.get(lang))
                        .align_x(Alignment::Center)
                        .width(Length::Fill)
                        .into(),
                    types.into(),
                    text::caption(format!("{} {}", fl!("total"), variety.stats.total())).into(),
                ])
                .spacing(2)
                .align_x(Alignment::Center)
                .width(Length::Fill),
            )
            .width(Length::Fixed(tile_width))
            .padding(8)
            .selected(is_current)
            .class(theme::Button::IconVertical)
            .on_press_maybe(
                (!is_current).then_some(Message::OpenPokemon(variety.id.to_native())),
            )
            .into()
        })
        .collect();

    let content: Element<'_, Message> = if cards.len() <= 1 {
        empty(fl!("no-forms"))
    } else {
        widget::flex_row(cards)
            .row_spacing(8)
            .column_spacing(8)
            .justify_content(widget::JustifyContent::Center)
            .width(Length::Fill)
            .into()
    };
    card(fl!("forms"), content)
}

/// Every sprite bundled for the current Pokémon.
fn gallery(ctx: Ctx<'_>, layout: Layout) -> Element<'_, Message> {
    let size = if layout.compact() { 96.0 } else { 128.0 };
    let labels = [
        (SpriteKind::Front, fl!("sprite-front")),
        (SpriteKind::Back, fl!("sprite-back")),
        (SpriteKind::FrontShiny, fl!("sprite-front-shiny")),
        (SpriteKind::BackShiny, fl!("sprite-back-shiny")),
        (SpriteKind::FrontFemale, fl!("sprite-front-female")),
        (SpriteKind::BackFemale, fl!("sprite-back-female")),
        (SpriteKind::FrontShinyFemale, fl!("sprite-front-shiny-female")),
        (SpriteKind::BackShinyFemale, fl!("sprite-back-shiny-female")),
    ];
    let tiles: Vec<Element<'_, Message>> = labels
        .into_iter()
        .filter_map(|(kind, label)| {
            let rel = ctx.pokemon.sprite_path(kind)?;
            Some(
                widget::column::with_children(vec![
                    sprite(Some(ctx.core.asset(&rel)), size),
                    text::caption(label).into(),
                ])
                .align_x(Alignment::Center)
                .spacing(2)
                .into(),
            )
        })
        .collect();
    let content: Element<'_, Message> = if tiles.is_empty() {
        empty(fl!("no-sprites"))
    } else {
        widget::flex_row(tiles)
            .row_spacing(12)
            .column_spacing(12)
            .justify_content(widget::JustifyContent::Center)
            .width(Length::Fill)
            .into()
    };
    card(fl!("sprites"), content)
}
