// SPDX-License-Identifier: GPL-3.0

//! Filters context drawer

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, checkbox, text};
use cosmic::{Element, theme};
use starrydex_model::{Generation, PokemonType};

use crate::app::pages::common::type_icon;
use crate::app::utils::presentation::Label;
use crate::app::utils::{Category, Filters};
use crate::app::{FiltersInput, Message};
use crate::fl;

pub fn view<'a>(filters: &'a Filters, result_count: usize) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;

    // Types: two per row, with their colored icons.
    let types = PokemonType::ALL
        .chunks(2)
        .map(|chunk| -> Element<'a, Message> {
            widget::row::with_children(
                chunk
                    .iter()
                    .map(|t| {
                        let t = *t;
                        widget::container(
                            widget::row::with_children(vec![
                                type_icon(t, 18),
                                text::body(t.label()).into(),
                                widget::space::horizontal().into(),
                                checkbox(filters.types.contains(&t))
                                    .on_toggle(move |v| Message::Filters(FiltersInput::Type(v, t)))
                                    .into(),
                            ])
                            .spacing(8)
                            .align_y(Alignment::Center),
                        )
                        .padding([0, 8])
                        .width(Length::Fill)
                        .into()
                    })
                    .collect::<Vec<_>>(),
            )
            .into()
        });

    let generations = Generation::ALL
        .iter()
        .filter(|g| **g != Generation::Unknown)
        .collect::<Vec<_>>()
        .chunks(3)
        .map(|chunk| {
            widget::row::with_children(
                chunk
                    .iter()
                    .map(|g| {
                        let g = **g;
                        checkbox_cell(g.label(), filters.generations.contains(&g), move |v| {
                            Message::Filters(FiltersInput::Generation(v, g))
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .into()
        })
        .collect::<Vec<Element<'a, Message>>>();

    let categories = Category::ALL
        .chunks(2)
        .map(|chunk| {
            widget::row::with_children(
                chunk
                    .iter()
                    .map(|c| {
                        let c = *c;
                        checkbox_cell(c.label(), filters.categories.contains(&c), move |v| {
                            Message::Filters(FiltersInput::Category(v, c))
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .into()
        })
        .collect::<Vec<Element<'a, Message>>>();

    let stats = widget::column::with_children(vec![
        widget::row::with_children(vec![
            text::body(fl!("minimum-poke-stats"))
                .width(Length::Fill)
                .into(),
            widget::toggler(filters.stats_enabled)
                .on_toggle(|v| Message::Filters(FiltersInput::StatsEnabled(v)))
                .into(),
        ])
        .align_y(Alignment::Center)
        .padding([0, 8])
        .into(),
        widget::row::with_children(vec![
            widget::slider(100..=780u16, filters.min_total, |v| {
                Message::Filters(FiltersInput::MinTotal(v))
            })
            .step(10u16)
            .width(Length::Fill)
            .into(),
            text::body(filters.min_total.to_string())
                .width(Length::Fixed(40.0))
                .into(),
        ])
        .spacing(8)
        .align_y(Alignment::Center)
        .padding([0, 8])
        .into(),
    ])
    .spacing(4);

    widget::column::with_children(vec![
        section(
            fl!("type-filters"),
            widget::column::with_children(types.collect::<Vec<_>>()).spacing(4),
        ),
        text::caption(fl!("type-filter-mode-hint"))
            .width(Length::Fill)
            .into(),
        section(
            fl!("generation-filters"),
            widget::column::with_children(generations).spacing(4),
        ),
        section(
            fl!("category-filters"),
            widget::column::with_children(categories).spacing(4),
        ),
        section(fl!("stats-filter"), stats),
        widget::row::with_children(vec![
            text::body(fl!("results-count", count = result_count))
                .width(Length::Fill)
                .into(),
            button::destructive(fl!("reset-filters"))
                .on_press_maybe(
                    filters
                        .any_applied()
                        .then_some(Message::Filters(FiltersInput::Reset)),
                )
                .into(),
        ])
        .align_y(Alignment::Center)
        .into(),
    ])
    .spacing(spacing.space_m)
    .width(Length::Fill)
    .into()
}

fn section<'a>(title: String, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    widget::column::with_children(vec![text::title4(title).into(), content.into()])
        .spacing(8)
        .into()
}

fn checkbox_cell<'a>(
    label: String,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    widget::container(
        widget::row::with_children(vec![
            text::body(label).into(),
            widget::space::horizontal().into(),
            checkbox(checked).on_toggle(on_toggle).into(),
        ])
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([0, 8])
    .width(Length::Fill)
    .into()
}
