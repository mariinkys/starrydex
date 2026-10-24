// SPDX-License-Identifier: GPL-3.0-only

//! Full-page Pokémon details with tabs.

use std::collections::BTreeSet;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, segmented_button, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedPokedex, ArchivedPokemon, ArchivedSpecies, Lang};

use crate::app::core::StarryCore;
use crate::app::pages::common::type_badge;
use crate::app::utils::PaginationAction;
use crate::app::utils::presentation::{Label, data_lang, dex_number};
use crate::app::{DetailsInput, Message};
use crate::{fl, icons};

mod evolution;
mod forms;
mod locations;
mod moves;
mod overview;
mod stats;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailTab {
    #[default]
    Overview,
    Stats,
    Moves,
    Evolution,
    Locations,
    Forms,
}

impl DetailTab {
    pub const ALL: &'static [Self] = &[
        Self::Overview,
        Self::Stats,
        Self::Moves,
        Self::Evolution,
        Self::Locations,
        Self::Forms,
    ];
}

impl Label for DetailTab {
    fn label(&self) -> String {
        match self {
            Self::Overview => fl!("tab-overview"),
            Self::Stats => fl!("tab-stats"),
            Self::Moves => fl!("tab-moves"),
            Self::Evolution => fl!("tab-evolution"),
            Self::Locations => fl!("tab-locations"),
            Self::Forms => fl!("tab-forms"),
        }
    }
}

/// State of the details page. Only indices and view options are stored: all data is read
/// from the memory-mapped Pokédex when drawing.
#[derive(Debug, Clone, Default)]
pub struct Details {
    /// Index into [`ArchivedPokedex::pokemon`].
    pub index: usize,
    /// Previously viewed Pokémon (followed evolution/form links), for the back button.
    pub history: Vec<usize>,
    pub shiny: bool,
    pub back: bool,
    pub female: bool,
    /// Selected Pokédex entry (clamped when drawing; `usize::MAX` = most recent).
    pub flavor_entry: usize,
    /// Version groups the Pokémon has learnsets for, newest first.
    pub move_version_groups: Vec<u16>,
    pub move_version_group: Option<u16>,
    pub expanded_moves: BTreeSet<u32>,
    /// Versions the Pokémon can be encountered in.
    pub encounter_versions: Vec<u16>,
    /// `None` shows every game.
    pub encounter_version: Option<u16>,
}

impl Details {
    pub fn new(index: usize, dex: &ArchivedPokedex) -> Self {
        let mut details = Self::default();
        details.show(index, dex);
        details
    }

    /// Switches to another Pokémon, keeping view options that still make sense.
    pub fn show(&mut self, index: usize, dex: &ArchivedPokedex) {
        let Some(pokemon) = dex.pokemon.get(index) else {
            return;
        };
        self.index = index;
        self.female = false;
        self.flavor_entry = usize::MAX;
        self.expanded_moves.clear();

        let mut groups: Vec<u16> = pokemon
            .moves
            .iter()
            .flat_map(|m| m.learns.iter().map(|l| l.version_group_id.to_native()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        groups.sort_by_key(|id| {
            std::cmp::Reverse(dex.version_group(*id).map_or(0, |vg| vg.order.to_native()))
        });
        // Keep the selected game when the new Pokémon is in it, otherwise use the newest.
        if !self
            .move_version_group
            .is_some_and(|selected| groups.contains(&selected))
        {
            self.move_version_group = groups.first().copied();
        }
        self.move_version_groups = groups;

        self.encounter_versions = pokemon
            .encounters
            .iter()
            .flat_map(|e| e.versions.iter().map(|v| v.version_id.to_native()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !self
            .encounter_version
            .is_some_and(|v| self.encounter_versions.contains(&v))
        {
            self.encounter_version = None;
        }
    }
}

/// Everything a tab needs to draw itself.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub core: &'a StarryCore,
    pub dex: &'a ArchivedPokedex,
    pub pokemon: &'a ArchivedPokemon,
    pub species: Option<&'a ArchivedSpecies>,
    pub details: &'a Details,
    pub lang: Lang,
}

/// Widest the tab content gets; beyond this it stays centered instead of stretching.
const MAX_CONTENT_WIDTH: f32 = 1440.0;

/// Responsive layout information handed to every tab, derived from the available width.
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Usable content width (already capped at [`MAX_CONTENT_WIDTH`]).
    pub width: f32,
    /// How many card columns fit: 1 (small window), 2 or 3 (large screens).
    pub columns: usize,
}

impl Layout {
    pub fn from_width(width: f32) -> Self {
        let width = width.min(MAX_CONTENT_WIDTH);
        let columns = if width >= 1180.0 {
            3
        } else if width >= 740.0 {
            2
        } else {
            1
        };
        Self { width, columns }
    }

    /// Small window: drop secondary columns, stack label/value pairs.
    pub fn compact(&self) -> bool {
        self.width < 560.0
    }

    /// Single-column layout (anything below tablet-ish width).
    pub fn narrow(&self) -> bool {
        self.columns == 1
    }

    /// Approximate width of one card when `columns` cards share a row.
    pub fn card_width(&self, columns: usize, spacing: f32) -> f32 {
        let columns = columns.max(1) as f32;
        (self.width - spacing * (columns - 1.0)) / columns
    }
}

/// Lays `items` out in `columns` equal-width columns (masonry style: items are dealt
/// round-robin so short and tall cards mix instead of leaving one column much longer).
pub fn grid<'a>(items: Vec<Element<'a, Message>>, columns: usize, spacing: u16) -> Element<'a, Message> {
    let columns = columns.max(1).min(items.len().max(1));
    if columns == 1 {
        return widget::column::with_children(items)
            .spacing(spacing)
            .width(Length::Fill)
            .into();
    }
    let mut buckets: Vec<Vec<Element<'a, Message>>> = (0..columns).map(|_| Vec::new()).collect();
    for (index, item) in items.into_iter().enumerate() {
        buckets[index % columns].push(item);
    }
    widget::row::with_children(
        buckets
            .into_iter()
            .map(|bucket| {
                widget::column::with_children(bucket)
                    .spacing(spacing)
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<_>>(),
    )
    .spacing(spacing)
    .width(Length::Fill)
    .into()
}

pub fn view<'a>(
    core: &'a StarryCore,
    details: &'a Details,
    tabs: &'a segmented_button::SingleSelectModel,
    tab: DetailTab,
    _results: &'a [u32],
) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let dex = core.dex();
    let Some(pokemon) = dex.pokemon.get(details.index) else {
        return text::body(fl!("unknown")).into();
    };
    let ctx = Ctx {
        core,
        dex,
        pokemon,
        species: dex.species_of(pokemon),
        details,
        lang: data_lang(),
    };

    // The tab is rebuilt with the real available width, so every tab can pick its layout.
    let body = widget::responsive(move |size| {
        let padding = f32::from(spacing.space_xxs) * 2.0;
        let layout = Layout::from_width(size.width - padding);
        let content = match tab {
            DetailTab::Overview => overview::view(ctx, layout),
            DetailTab::Stats => stats::view(ctx, layout),
            DetailTab::Moves => moves::view(ctx, layout),
            DetailTab::Evolution => evolution::view(ctx, layout),
            DetailTab::Locations => locations::view(ctx, layout),
            DetailTab::Forms => forms::view(ctx, layout),
        };
        widget::scrollable(
            widget::container(
                widget::container(content)
                    .max_width(MAX_CONTENT_WIDTH)
                    .width(Length::Fill),
            )
            .padding([spacing.space_xs, spacing.space_xxs])
            .width(Length::Fill)
            .align_x(Alignment::Center),
        )
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
    });

    widget::column::with_children(vec![
        header(ctx),
        widget::segmented_control::horizontal(tabs)
            .on_activate(Message::DetailTab)
            .width(Length::Fill)
            .into(),
        body.into(),
    ])
    .spacing(spacing.space_xs)
    .padding([spacing.space_xxs, spacing.space_s])
    .width(Length::Fill)
    .into()
}

fn header(ctx: Ctx<'_>) -> Element<'_, Message> {
    let Ctx {
        pokemon,
        species,
        lang,
        details,
        ..
    } = ctx;

    let mut subtitle = vec![dex_number(pokemon.species_id.to_native())];
    if let Some(species) = species {
        let genus = species.genus.get(lang);
        if !genus.is_empty() {
            subtitle.push(genus.to_string());
        }
        subtitle.push(species.generation.label());
    }

    let back_label = if details.history.is_empty() {
        fl!("back-to-list")
    } else {
        fl!("back")
    };

    let title = widget::column::with_children(vec![
        text::title2(pokemon.display.get(lang)).into(),
        text::body(subtitle.join(" · ")).into(),
    ])
    .spacing(2)
    .width(Length::Fill);

    let types = widget::row::with_children(pokemon.types.iter().map(|t| type_badge(*t)).collect::<Vec<_>>())
        .spacing(6);

    widget::row::with_children(vec![
        button::text(back_label)
            .leading_icon(icons::get_handle("go-previous-symbolic", 18))
            .on_press(Message::Back)
            .into(),
        title.into(),
        types.into(),
        widget::tooltip(
            button::icon(icons::get_handle("go-previous-symbolic", 18))
                .on_press(Message::Details(DetailsInput::Step(PaginationAction::Back))),
            text::body(fl!("previous-pokemon")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
        widget::tooltip(
            button::icon(icons::get_handle("go-next-symbolic", 18))
                .on_press(Message::Details(DetailsInput::Step(PaginationAction::Next))),
            text::body(fl!("next-pokemon")),
            widget::tooltip::Position::Bottom,
        )
        .into(),
    ])
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// Label/value pair: side by side normally, stacked when the layout is compact.
pub fn field<'a>(
    layout: Layout,
    label: impl Into<String>,
    value: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    if layout.compact() {
        widget::column::with_children(vec![
            text::caption_heading(label.into())
                .class(theme::Text::Accent)
                .into(),
            value.into(),
        ])
        .spacing(2)
        .width(Length::Fill)
        .into()
    } else {
        crate::app::pages::common::info_row(label, value)
    }
}
