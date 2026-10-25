// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use cosmic::iced::core::image::FilterMethod;
use cosmic::iced::{Alignment, Background, Border, ContentFit, Length};
use cosmic::widget::{self, container, text};
use cosmic::{Element, theme};
use starrydex_model::PokemonType;

use crate::app::Message;
use crate::app::utils::presentation::{Label, type_color, type_icon_name, type_svg_class};
use crate::{icons, images};

/// A sprite scaled with nearest-neighbour filtering so pixel art stays crisp.
pub fn sprite<'a>(path: Option<PathBuf>, size: f32) -> Element<'a, Message> {
    let handle = match path {
        Some(path) => widget::image::Handle::from_path(path),
        None => images::get("fallback"),
    };
    widget::image(handle)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .content_fit(ContentFit::Contain)
        .filter_method(FilterMethod::Nearest)
        .into()
}

/// Colored type icon.
pub fn type_icon<'a>(t: PokemonType, size: u16) -> Element<'a, Message> {
    widget::icon(icons::get_handle_owned(type_icon_name(t), 18))
        .size(size)
        .class(type_svg_class(t))
        .into()
}

/// Type icon with a tooltip.
pub fn type_icon_tooltip<'a>(t: PokemonType, size: u16) -> Element<'a, Message> {
    widget::tooltip(
        type_icon(t, size),
        text::body(t.label()),
        widget::tooltip::Position::Bottom,
    )
    .into()
}

/// Pill with the type icon and name, tinted with the type color.
pub fn type_badge<'a>(t: PokemonType) -> Element<'a, Message> {
    let color = type_color(t);
    container(
        widget::row::with_children(vec![type_icon(t, 16), text::body(t.label()).into()])
            .spacing(6)
            .align_y(Alignment::Center),
    )
    .padding([3, 10])
    .class(theme::Container::custom(move |_| container::Style {
        background: Some(Background::Color(cosmic::iced::Color { a: 0.18, ..color })),
        border: Border {
            color,
            width: 1.0,
            radius: 14.0.into(),
        },
        ..Default::default()
    }))
    .into()
}

/// Small rounded label ("Hidden", "Legendary"...).
pub fn tag<'a>(label: impl Into<String>) -> Element<'a, Message> {
    container(text::caption_heading(label.into()))
        .padding([2, 8])
        .class(theme::Container::custom(|theme| {
            let cosmic = theme.cosmic();
            container::Style {
                background: Some(Background::Color(cosmic.accent.base.into())),
                text_color: Some(cosmic.accent.on.into()),
                border: Border {
                    radius: 10.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }))
        .into()
}

/// A titled card.
pub fn card<'a>(
    title: impl Into<String>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        widget::column::with_children(vec![text::title4(title.into()).into(), content.into()])
            .spacing(8)
            .width(Length::Fill),
    )
    .padding(12)
    .width(Length::Fill)
    .class(theme::Container::Card)
    .into()
}

/// Label/value row used in info cards.
pub fn info_row<'a>(
    label: impl Into<String>,
    value: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    widget::row::with_children(vec![
        text::body(label.into())
            .class(theme::Text::Accent)
            .width(Length::FillPortion(2))
            .into(),
        container(value.into()).width(Length::FillPortion(3)).into(),
    ])
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// Centered placeholder for empty sections.
pub fn empty<'a>(message: impl Into<String>) -> Element<'a, Message> {
    container(text::body(message.into()))
        .width(Length::Fill)
        .padding(24)
        .align_x(Alignment::Center)
        .into()
}
