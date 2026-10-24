// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, button, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedSpecies, CryKind, Lang, SpriteKind, item_sprite_rel_path};

use super::{Ctx, Layout, field, grid};
use crate::app::pages::common::{card, empty, sprite, tag};
use crate::app::utils::presentation::{
    Label, capture_chance, format_height, format_weight, gender_split, stat_short, uppercase_first,
};
use crate::app::{DetailsInput, Message};
use crate::{fl, icons};

pub fn view<'a>(ctx: Ctx<'a>, layout: Layout) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;

    let mut cards = vec![profile(ctx, layout), training(ctx, layout)];
    let top: Element<'a, Message> = if layout.narrow() {
        cards.insert(0, abilities(ctx));
        widget::column::with_children(vec![hero(ctx, layout), pokedex_entry(ctx, layout)])
            .spacing(spacing.space_s)
            .into()
    } else {
        widget::row::with_children(vec![
            hero(ctx, layout),
            widget::column::with_children(vec![pokedex_entry(ctx, layout), abilities(ctx)])
                .spacing(spacing.space_s)
                .width(Length::Fill)
                .into(),
        ])
        .align_y(Alignment::Center)
        .spacing(spacing.space_s)
        .into()
    };

    if let Some(species) = ctx.species {
        cards.push(breeding(species, layout));
        cards.push(dex_numbers(ctx, species, layout));
    }
    if !ctx.pokemon.held_items.is_empty() {
        cards.push(held_items(ctx));
    }
    cards.push(games(ctx));

    widget::column::with_children(vec![top, grid(cards, layout.columns, spacing.space_s)])
        .spacing(spacing.space_s)
        .width(Length::Fill)
        .into()
}

/// Big sprite with shiny/back/female toggles and cry buttons.
fn hero<'a>(ctx: Ctx<'a>, layout: Layout) -> Element<'a, Message> {
    let Ctx {
        core,
        pokemon,
        details,
        ..
    } = ctx;
    let kind = SpriteKind::select(details.back, details.shiny, details.female);
    let sprite_size = if layout.compact() { 160.0 } else { 220.0 };

    let toggle = |label: String,
                  active: bool,
                  message: DetailsInput,
                  available: bool|
     -> Element<'_, Message> {
        let b = if active {
            button::suggested(label)
        } else {
            button::standard(label)
        };
        b.on_press_maybe(available.then_some(Message::Details(message)))
            .into()
    };

    let mut toggles = vec![
        toggle(
            fl!("shiny"),
            details.shiny,
            DetailsInput::ToggleShiny,
            pokemon.has_sprite(SpriteKind::FrontShiny),
        ),
        toggle(
            fl!("back-sprite"),
            details.back,
            DetailsInput::ToggleBack,
            pokemon.has_sprite(SpriteKind::Back),
        ),
    ];
    if pokemon.has_sprite(SpriteKind::FrontFemale) {
        toggles.push(toggle(
            fl!("female"),
            details.female,
            DetailsInput::ToggleFemale,
            true,
        ));
    }

    let mut cries: Vec<Element<'_, Message>> = Vec::new();
    for (kind, label) in [
        (CryKind::Latest, fl!("cry")),
        (CryKind::Legacy, fl!("cry-legacy")),
    ] {
        if let Some(rel) = pokemon.cry_path(kind) {
            let path = core.asset(&rel).to_string_lossy().into_owned();
            cries.push(
                button::text(label)
                    .leading_icon(icons::get_handle("audio-speaker-symbolic", 18))
                    .on_press(Message::Details(DetailsInput::PlayAudio(path)))
                    .into(),
            );
        }
    }

    let centered_row = |items: Vec<Element<'a, Message>>| -> Element<'a, Message> {
        container(widget::row::with_children(items).spacing(6))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    };

    let mut column = vec![
        container(sprite(core.sprite(pokemon, kind), sprite_size))
            .width(Length::Fill)
            .align_x(Alignment::Center)
            .into(),
        centered_row(toggles),
    ];
    if !cries.is_empty() {
        column.push(centered_row(cries));
    }

    container(
        widget::column::with_children(column)
            .spacing(8)
            .align_x(Alignment::Center),
    )
    .padding(12)
    // Fixed width beside the entry; full width when stacked.
    .width(if layout.narrow() {
        Length::Fill
    } else {
        Length::Fixed(300.0)
    })
    .class(theme::Container::Card)
    .into()
}

fn pokedex_entry(ctx: Ctx<'_>, layout: Layout) -> Element<'_, Message> {
    let Ctx {
        dex,
        species,
        details,
        lang,
        ..
    } = ctx;

    let Some(species) = species else {
        return card(fl!("pokedex-entry"), empty(fl!("no-flavor-text")));
    };
    // Entries in the UI language, falling back to English.
    let mut entries: Vec<_> = species
        .flavor_texts
        .iter()
        .filter(|f| f.lang == lang)
        .collect();
    if entries.is_empty() {
        entries = species
            .flavor_texts
            .iter()
            .filter(|f| f.lang == Lang::En)
            .collect();
    }
    if entries.is_empty() {
        return card(fl!("pokedex-entry"), empty(fl!("no-flavor-text")));
    }
    let selected = details.flavor_entry.min(entries.len() - 1);
    let labels: Vec<String> = entries
        .iter()
        .map(|entry| {
            let names: Vec<&str> = entry
                .version_ids
                .iter()
                .filter_map(|id| dex.version(id.to_native()))
                .map(|v| v.display.get(lang))
                .collect();
            if names.is_empty() {
                fl!("unknown")
            } else {
                names.join(", ")
            }
        })
        .collect();

    let mut content = vec![text::body(entries[selected].text.as_str()).into()];
    if entries.len() > 1 {
        let picker = widget::dropdown(Cow::Owned(labels), Some(selected), |i| {
            Message::Details(DetailsInput::FlavorEntry(i))
        });
        content.push(if layout.compact() {
            widget::column::with_children(vec![
                text::caption(fl!("entry-from")).into(),
                picker.into(),
            ])
            .spacing(4)
            .into()
        } else {
            widget::row::with_children(vec![text::caption(fl!("entry-from")).into(), picker.into()])
                .spacing(8)
                .align_y(Alignment::Center)
                .into()
        });
    } else {
        content.push(text::caption(labels.into_iter().next().unwrap_or_default()).into());
    }

    card(
        fl!("pokedex-entry"),
        widget::column::with_children(content).spacing(10),
    )
}

fn profile(ctx: Ctx<'_>, layout: Layout) -> Element<'_, Message> {
    let Ctx {
        pokemon,
        species,
        lang,
        ..
    } = ctx;
    let mut rows = vec![
        field(
            layout,
            fl!("height-label"),
            text::body(format_height(pokemon.height.to_native())),
        ),
        field(
            layout,
            fl!("weight-label"),
            text::body(format_weight(pokemon.weight.to_native())),
        ),
    ];
    if let Some(species) = species {
        for (label, value) in [
            (fl!("habitat"), &species.habitat),
            (fl!("color"), &species.color),
            (fl!("shape"), &species.shape),
        ] {
            if !value.is_empty() {
                rows.push(field(
                    layout,
                    label,
                    text::body(uppercase_first(value.get(lang))),
                ));
            }
        }
    }
    if !pokemon.form_name.is_empty() {
        rows.push(field(
            layout,
            fl!("form"),
            text::body(pokemon.form_name.get(lang)),
        ));
    }

    let mut tags: Vec<Element<'_, Message>> = Vec::new();
    if let Some(species) = species {
        if species.is_legendary {
            tags.push(tag(fl!("category-legendary")));
        }
        if species.is_mythical {
            tags.push(tag(fl!("category-mythical")));
        }
        if species.is_baby {
            tags.push(tag(fl!("category-baby")));
        }
    }
    if !pokemon.is_default {
        tags.push(tag(fl!("alternate-form")));
    }
    if !tags.is_empty() {
        rows.push(
            widget::flex_row(tags)
                .row_spacing(6)
                .column_spacing(6)
                .into(),
        );
    }
    card(
        fl!("profile"),
        widget::column::with_children(rows).spacing(6),
    )
}

fn abilities(ctx: Ctx<'_>) -> Element<'_, Message> {
    let Ctx {
        dex, pokemon, lang, ..
    } = ctx;
    let rows: Vec<Element<'_, Message>> = pokemon
        .abilities
        .iter()
        .filter_map(|slot| {
            let ability = dex.ability(slot.ability_id.to_native())?;
            let mut title = vec![text::heading(ability.display.get(lang)).into()];
            if slot.is_hidden {
                title.push(tag(fl!("hidden-ability")));
            }
            let description = if ability.short_effect.has(lang) || ability.flavor.is_empty() {
                ability.short_effect.get(lang)
            } else {
                ability.flavor.get(lang)
            };
            Some(
                widget::column::with_children(vec![
                    widget::row::with_children(title)
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .into(),
                    text::body(description).into(),
                ])
                .spacing(2)
                .into(),
            )
        })
        .collect();
    card(
        fl!("pokemon-abilities"),
        widget::column::with_children(rows).spacing(10),
    )
}

fn training(ctx: Ctx<'_>, layout: Layout) -> Element<'_, Message> {
    let Ctx {
        pokemon, species, ..
    } = ctx;
    let ev_yield: Vec<String> = pokemon
        .ev_yield
        .values()
        .iter()
        .filter(|(_, v)| *v > 0)
        .map(|(stat, v)| format!("{v} {}", stat_short(*stat)))
        .collect();

    let mut rows = vec![field(
        layout,
        fl!("ev-yield"),
        text::body(if ev_yield.is_empty() {
            "—".to_string()
        } else {
            ev_yield.join(", ")
        }),
    )];
    if let Some(exp) = pokemon.base_experience.as_ref() {
        rows.push(field(
            layout,
            fl!("base-exp"),
            text::body(exp.to_native().to_string()),
        ));
    }
    if let Some(species) = species {
        rows.push(field(
            layout,
            fl!("catch-rate"),
            text::body(format!(
                "{} ({:.1}%)",
                species.capture_rate,
                capture_chance(species.capture_rate)
            )),
        ));
        if let Some(happiness) = species.base_happiness.as_ref() {
            rows.push(field(
                layout,
                fl!("base-friendship"),
                text::body(happiness.to_string()),
            ));
        }
        rows.push(field(
            layout,
            fl!("growth-rate"),
            text::body(format!(
                "{} ({})",
                species.growth_rate.label(),
                fl!("exp-to-100", exp = species.growth_rate.exp_at_100())
            )),
        ));
    }
    card(
        fl!("training"),
        widget::column::with_children(rows).spacing(6),
    )
}

fn breeding(species: &ArchivedSpecies, layout: Layout) -> Element<'_, Message> {
    let egg_groups: Vec<String> = species.egg_groups.iter().map(Label::label).collect();
    let gender: Element<'_, Message> = match gender_split(species.gender_rate) {
        None => text::body(fl!("genderless")).into(),
        Some((male, female)) => widget::column::with_children(vec![
            text::body(format!("♂ {male}%   ♀ {female}%")).into(),
            widget::progress_bar::determinate_linear(male / 100.0)
                .girth(6)
                .into(),
        ])
        .spacing(4)
        .into(),
    };
    let mut rows = vec![
        field(layout, fl!("egg-groups"), text::body(egg_groups.join(", "))),
        field(layout, fl!("gender"), gender),
    ];
    if let Some(cycles) = species.hatch_counter.as_ref() {
        rows.push(field(
            layout,
            fl!("egg-cycles"),
            text::body(fl!(
                "egg-cycles-value",
                cycles = (*cycles),
                steps = (u32::from(*cycles) * 257)
            )),
        ));
    }
    card(
        fl!("breeding"),
        widget::column::with_children(rows).spacing(6),
    )
}

fn dex_numbers<'a>(
    ctx: Ctx<'a>,
    species: &'a ArchivedSpecies,
    layout: Layout,
) -> Element<'a, Message> {
    let rows: Vec<Element<'a, Message>> = species
        .dex_numbers
        .iter()
        .filter_map(|n| {
            let dex = ctx.dex.regional_dex(n.pokedex_id.to_native())?;
            Some(field(
                layout,
                dex.display.get(ctx.lang).to_string(),
                text::body(format!("#{:03}", n.number.to_native())),
            ))
        })
        .collect();
    card(
        fl!("regional-numbers"),
        widget::column::with_children(rows).spacing(4),
    )
}

fn held_items(ctx: Ctx<'_>) -> Element<'_, Message> {
    let Ctx {
        core,
        dex,
        pokemon,
        lang,
        ..
    } = ctx;
    let rows: Vec<Element<'_, Message>> = pokemon
        .held_items
        .iter()
        .filter_map(|held| {
            let item = dex.item(held.item_id.to_native())?;
            let max_rarity = held.versions.iter().map(|v| v.rarity).max().unwrap_or(0);
            let icon = if item.has_sprite {
                sprite(
                    Some(core.asset(&item_sprite_rel_path(item.name.as_str()))),
                    30.0,
                )
            } else {
                widget::Space::new().width(30).into()
            };
            let mut versions: Vec<&str> = held
                .versions
                .iter()
                .filter_map(|v| dex.version(v.version_id.to_native()))
                .map(|v| v.display.get(lang))
                .collect();
            versions.dedup();
            Some(
                widget::row::with_children(vec![
                    icon,
                    widget::column::with_children(vec![
                        text::heading(format!("{} · {max_rarity}%", item.display.get(lang))).into(),
                        text::caption(versions.join(", ")).into(),
                    ])
                    .width(Length::Fill)
                    .into(),
                ])
                .spacing(8)
                .align_y(Alignment::Center)
                .into(),
            )
        })
        .collect();
    card(
        fl!("held-items"),
        widget::column::with_children(rows).spacing(6),
    )
}

fn games(ctx: Ctx<'_>) -> Element<'_, Message> {
    let chips: Vec<Element<'_, Message>> = ctx
        .pokemon
        .game_versions
        .iter()
        .filter_map(|id| ctx.dex.version(id.to_native()))
        .map(|v| {
            container(text::caption(v.display.get(ctx.lang)))
                .padding([2, 8])
                .class(theme::Container::List)
                .into()
        })
        .collect();
    let content: Element<'_, Message> = if chips.is_empty() {
        empty(fl!("unknown"))
    } else {
        widget::flex_row(chips)
            .row_spacing(4)
            .column_spacing(4)
            .into()
    };
    card(fl!("appears-in"), content)
}
