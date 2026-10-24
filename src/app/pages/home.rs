// SPDX-License-Identifier: GPL-3.0

use std::borrow::Cow;

use cosmic::iced::core::text::LineHeight;
use cosmic::iced::{Alignment, Length, Pixels};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokemon, SpriteKind};

use crate::app::context_page::ContextPage;
use crate::app::core::StarryCore;
use crate::app::pages::common::{sprite, type_icon};
use crate::app::utils::PaginationAction;
use crate::app::utils::presentation::{data_lang, dex_number};
use crate::app::{AppModel, ListInput, Message, Ready, page_count};
use crate::config::ViewMode;
use crate::{fl, icons};

const CARD_WIDTH: f32 = 168.0;
const CARD_HEIGHT: f32 = 168.0;

pub fn view<'a>(app: &'a AppModel, ready: &'a Ready) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let per_page = app.config.pokemon_per_page.max(1);
    let pages = page_count(ready.results.len(), per_page);
    let page = ready.page.min(pages - 1);
    let start = page * per_page;
    let end = (start + per_page).min(ready.results.len());

    // TOOLBAR
    let filter_count = ready.filters.active_count();
    let filter_label = if filter_count > 0 {
        format!("{} ({filter_count})", fl!("filters-page"))
    } else {
        fl!("filters-page")
    };
    let has_query =
        !ready.search.is_empty() || ready.filters.any_applied() || ready.sort != Default::default();
    let order_label = if ready.descending {
        fl!("order-descending")
    } else {
        fl!("order-ascending")
    };

    let toolbar = widget::row::with_children(vec![
        widget::search_input(fl!("search-placeholder"), &ready.search)
            .on_input(|v| Message::List(ListInput::Search(v)))
            .on_clear(Message::List(ListInput::Search(String::new())))
            .line_height(LineHeight::Absolute(Pixels(30.0)))
            .width(Length::Fill)
            .into(),
        widget::dropdown(
            Cow::Borrowed(app.sort_options.as_slice()),
            crate::app::utils::SortBy::ALL
                .iter()
                .position(|s| *s == ready.sort),
            |i| Message::List(ListInput::Sort(i)),
        )
        .into(),
        widget::tooltip(
            button::standard(order_label).on_press(Message::List(ListInput::ToggleOrder)),
            text::body(fl!("reverse-order")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
        button::standard(filter_label)
            .leading_icon(icons::get_handle("filter-symbolic", 18))
            .on_press(Message::ToggleContextPage(ContextPage::FiltersPage))
            .into(),
        widget::tooltip(
            button::icon(icons::get_handle("edit-clear-all-symbolic", 18))
                .class(theme::Button::Destructive)
                .on_press_maybe(has_query.then_some(Message::List(ListInput::ClearAll))),
            text::body(fl!("clear-all")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    ])
    .spacing(spacing.space_xxs)
    .align_y(Alignment::Center)
    .width(Length::Fill);

    // LIST
    let core = &ready.core;
    let dex = core.dex();
    let visible = &ready.results[start..end];
    let content: Element<'a, Message> = if visible.is_empty() {
        widget::column::with_children(vec![
            text::title3(fl!("no-results")).into(),
            text::body(fl!("no-results-hint")).into(),
            button::suggested(fl!("clear-all"))
                .on_press(Message::List(ListInput::ClearAll))
                .into(),
        ])
        .spacing(12)
        .align_x(Alignment::Center)
        .padding(48)
        .width(Length::Fill)
        .into()
    } else {
        let cards = visible
            .iter()
            .map(|&index| pokemon_card(core, &dex.pokemon[index as usize]));
        match app.config.view_mode {
            ViewMode::Manual { pokemon_per_row } => {
                let mut grid = widget::Grid::new()
                    .column_spacing(spacing.space_xs)
                    .row_spacing(spacing.space_xs);
                for (i, card) in cards.enumerate() {
                    if i > 0 && i % pokemon_per_row.max(1) == 0 {
                        grid = grid.insert_row();
                    }
                    grid = grid.push(card);
                }
                container(grid)
                    .width(Length::Fill)
                    .align_x(Alignment::Center)
                    .into()
            }
            ViewMode::Responsive => widget::flex_row(cards.collect())
                .row_spacing(spacing.space_xs)
                .column_spacing(spacing.space_xs)
                .justify_content(widget::JustifyContent::Center)
                .width(Length::Fill)
                .into(),
        }
    };

    // FOOTER
    let summary = if ready.results.is_empty() {
        fl!("results-none")
    } else {
        fl!(
            "results-range",
            start = (start + 1),
            end = end,
            total = ready.results.len()
        )
    };
    let footer = widget::row::with_children(vec![
        text::caption(summary).width(Length::Fill).into(),
        button::icon(icons::get_handle("go-previous-symbolic", 18))
            .on_press_maybe(
                (page > 0).then_some(Message::List(ListInput::Page(PaginationAction::Back))),
            )
            .into(),
        text::body(fl!("page-of", page = (page + 1), pages = pages)).into(),
        button::icon(icons::get_handle("go-next-symbolic", 18))
            .on_press_maybe(
                (page + 1 < pages)
                    .then_some(Message::List(ListInput::Page(PaginationAction::Next))),
            )
            .into(),
        widget::Space::new().width(Length::Fill).into(),
    ])
    .spacing(spacing.space_s)
    .align_y(Alignment::Center);

    widget::column::with_children(vec![
        toolbar.into(),
        widget::scrollable(container(content).padding([spacing.space_xs, 0]))
            .height(Length::Fill)
            .width(Length::Fill)
            .into(),
        footer.into(),
    ])
    .spacing(spacing.space_xxs)
    .padding([spacing.space_xxs, spacing.space_s])
    .width(Length::Fill)
    .into()
}

fn pokemon_card<'a>(core: &'a StarryCore, pokemon: &'a ArchivedPokemon) -> Element<'a, Message> {
    let lang = data_lang();
    let types = widget::row::with_children(
        pokemon
            .types
            .iter()
            .map(|t| type_icon(*t, 16))
            .collect::<Vec<_>>(),
    )
    .spacing(4);

    let mut column = widget::column::with_children(vec![
        sprite(core.sprite(pokemon, SpriteKind::Front), 96.0),
        text::caption(dex_number(pokemon.species_id.to_native())).into(),
        text::heading(pokemon.display.get(lang))
            .align_x(Alignment::Center)
            .wrapping(cosmic::iced::core::text::Wrapping::WordOrGlyph)
            .into(),
    ])
    .spacing(2)
    .width(Length::Fill)
    .align_x(Alignment::Center);
    column = column.push(types);

    button::custom(column)
        .width(Length::Fixed(CARD_WIDTH))
        .height(Length::Fixed(CARD_HEIGHT))
        .padding(6)
        .on_press(Message::OpenPokemon(pokemon.id.to_native()))
        .class(theme::Button::IconVertical)
        .into()
}
