// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;

use cosmic::iced::{Alignment, Length};
use cosmic::widget::{self, container, text};
use cosmic::{Element, theme};
use starrydex_model::{ArchivedMove, LearnMethod};

use super::{Ctx, Layout, grid};
use crate::app::pages::common::{card, empty, type_icon_tooltip};
use crate::app::utils::presentation::Label;
use crate::app::{DetailsInput, Message};
use crate::{fl, icons};

const METHODS: [LearnMethod; 6] = [
    LearnMethod::LevelUp,
    LearnMethod::Machine,
    LearnMethod::Egg,
    LearnMethod::Tutor,
    LearnMethod::FormChange,
    LearnMethod::Other,
];

/// A move table needs roughly this much room to show every column comfortably.
const FULL_TABLE_WIDTH: f32 = 620.0;

struct Entry<'a> {
    method: LearnMethod,
    level: u8,
    machine: Option<&'a str>,
    data: &'a ArchivedMove,
}

pub fn view<'a>(ctx: Ctx<'a>, layout: Layout) -> Element<'a, Message> {
    let spacing = theme::active().cosmic().spacing;
    let Ctx {
        dex,
        pokemon,
        details,
        lang,
        ..
    } = ctx;

    let Some(selected) = details.move_version_group else {
        return empty(fl!("no-moves"));
    };

    let labels: Vec<String> = details
        .move_version_groups
        .iter()
        .map(|id| {
            dex.version_group(*id)
                .map_or_else(|| fl!("unknown"), |vg| vg.display.get(lang).to_string())
        })
        .collect();
    let picker = widget::row::with_children(vec![
        text::heading(fl!("game")).into(),
        widget::dropdown(
            Cow::Owned(labels),
            details
                .move_version_groups
                .iter()
                .position(|id| *id == selected),
            |i| Message::Details(DetailsInput::MoveVersionGroup(i)),
        )
        .into(),
    ])
    .spacing(8)
    .align_y(Alignment::Center);

    let mut entries: Vec<Entry<'_>> = Vec::new();
    for pm in pokemon.moves.iter() {
        let Some(data) = dex.move_(pm.move_id.to_native()) else {
            continue;
        };
        for learn in pm
            .learns
            .iter()
            .filter(|l| l.version_group_id.to_native() == selected)
        {
            entries.push(Entry {
                method: learn.method,
                level: learn.level,
                machine: (learn.method == LearnMethod::Machine)
                    .then(|| {
                        data.machines
                            .iter()
                            .find(|m| m.version_group_id.to_native() == selected)
                            .map(|m| m.label.as_str())
                    })
                    .flatten(),
                data,
            });
        }
    }

    // Two tables side by side only when each still gets a full-width table.
    let table_columns =
        if layout.card_width(2, f32::from(spacing.space_s)) >= FULL_TABLE_WIDTH + 24.0 {
            2
        } else {
            1
        };
    let full = layout.card_width(table_columns, f32::from(spacing.space_s)) >= FULL_TABLE_WIDTH;

    let mut tables: Vec<Element<'_, Message>> = Vec::new();
    for method in METHODS {
        let mut group: Vec<&Entry<'_>> = entries.iter().filter(|e| e.method == method).collect();
        if group.is_empty() {
            continue;
        }
        match method {
            LearnMethod::LevelUp => group.sort_by_key(|e| (e.level, e.data.display.get(lang))),
            LearnMethod::Machine => group.sort_by_key(|e| machine_sort_key(e.machine)),
            _ => group.sort_by_key(|e| e.data.display.get(lang)),
        }
        let mut rows = vec![header_row(full)];
        for entry in &group {
            rows.push(move_row(ctx, entry, full));
        }
        tables.push(card(
            format!("{} ({})", method.label(), group.len()),
            widget::column::with_children(rows).spacing(2),
        ));
    }

    let body = if tables.is_empty() {
        empty(fl!("no-moves"))
    } else {
        grid(tables, table_columns, spacing.space_s)
    };

    widget::column::with_children(vec![picker.into(), body])
        .spacing(spacing.space_s)
        .width(Length::Fill)
        .into()
}

/// "TM05" < "TM24" < "HM01" < "TR03"; unknown machines last.
fn machine_sort_key(label: Option<&str>) -> (u8, u32) {
    let Some(label) = label else {
        return (9, 0);
    };
    let prefix = match label.get(..2) {
        Some("TM") => 0,
        Some("HM") => 1,
        Some("TR") => 2,
        _ => 3,
    };
    let number = label
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap_or(0);
    (prefix, number)
}

/// One table row. `full` shows the Category and Accuracy columns, which are dropped in
/// small windows so the move name keeps enough room.
struct Cells<'a> {
    learned: Element<'a, Message>,
    type_icon: Element<'a, Message>,
    name: Element<'a, Message>,
    category: Element<'a, Message>,
    power: Element<'a, Message>,
    accuracy: Element<'a, Message>,
    pp: Element<'a, Message>,
    chevron: Element<'a, Message>,
}

fn columns(cells: Cells<'_>, full: bool) -> Element<'_, Message> {
    let mut row = vec![
        container(cells.learned)
            .width(Length::Fixed(if full { 64.0 } else { 52.0 }))
            .into(),
        container(cells.type_icon).width(Length::Fixed(24.0)).into(),
        container(cells.name).width(Length::Fill).into(),
    ];
    if full {
        row.push(container(cells.category).width(Length::Fixed(84.0)).into());
    }
    row.push(container(cells.power).width(Length::Fixed(48.0)).into());
    if full {
        row.push(container(cells.accuracy).width(Length::Fixed(48.0)).into());
    }
    row.push(container(cells.pp).width(Length::Fixed(32.0)).into());
    row.push(cells.chevron);
    widget::row::with_children(row)
        .spacing(6)
        .padding([4, 8])
        .align_y(Alignment::Center)
        .into()
}

fn header_row<'a>(full: bool) -> Element<'a, Message> {
    columns(
        Cells {
            learned: text::caption_heading(fl!("learned")).into(),
            type_icon: widget::Space::new().width(Length::Fixed(20.0)).into(),
            name: text::caption_heading(fl!("move")).into(),
            category: text::caption_heading(fl!("category")).into(),
            power: text::caption_heading(fl!("power-short")).into(),
            accuracy: text::caption_heading(fl!("accuracy-short")).into(),
            pp: text::caption_heading(fl!("pp")).into(),
            chevron: widget::Space::new().width(Length::Fixed(16.0)).into(),
        },
        full,
    )
}

fn move_row<'a>(ctx: Ctx<'a>, entry: &Entry<'a>, full: bool) -> Element<'a, Message> {
    let lang = ctx.lang;
    let data = entry.data;
    let id = data.id.to_native();
    let expanded = ctx.details.expanded_moves.contains(&id);

    let learned = match entry.method {
        LearnMethod::LevelUp if entry.level == 0 => fl!("evolve-short"),
        LearnMethod::LevelUp => fl!("level-short", level = entry.level),
        LearnMethod::Machine => entry.machine.unwrap_or("—").to_string(),
        _ => "—".to_string(),
    };
    let opt = |v: Option<&u8>| v.map_or_else(|| "—".to_string(), ToString::to_string);

    let row = columns(
        Cells {
            learned: text::body(learned).into(),
            type_icon: type_icon_tooltip(data.move_type, 18),
            name: text::heading(data.display.get(lang)).into(),
            category: text::body(data.damage_class.label()).into(),
            power: text::body(opt(data.power.as_ref())).into(),
            accuracy: text::body(opt(data.accuracy.as_ref())).into(),
            pp: text::body(opt(data.pp.as_ref())).into(),
            chevron: widget::icon(icons::get_handle(
                if expanded {
                    "go-up-symbolic"
                } else {
                    "go-down-symbolic"
                },
                16,
            ))
            .size(16)
            .into(),
        },
        full,
    );

    let toggle = widget::mouse_area(container(row).width(Length::Fill).class(if expanded {
        theme::Container::List
    } else {
        theme::Container::Transparent
    }))
    .interaction(cosmic::iced::mouse::Interaction::Pointer)
    .on_press(Message::Details(DetailsInput::ToggleMove(id)));

    if !expanded {
        return toggle.into();
    }

    let mut info: Vec<Element<'a, Message>> = Vec::new();
    let effect = data.short_effect.get(lang);
    if !effect.is_empty() {
        info.push(text::body(effect).into());
    }
    let flavor = data.flavor.get(lang);
    if !flavor.is_empty() && flavor != effect {
        info.push(text::body(flavor).class(theme::Text::Accent).into());
    }
    let mut facts = Vec::new();
    // Hidden columns are shown here instead when the table is compact.
    if !full {
        facts.push(data.damage_class.label());
        facts.push(format!(
            "{}: {}",
            fl!("accuracy-short"),
            opt(data.accuracy.as_ref())
        ));
    }
    facts.push(format!("{}: {}", fl!("target"), data.target.get(lang)));
    if data.priority != 0 {
        facts.push(format!("{}: {:+}", fl!("priority"), data.priority));
    }
    if let Some(meta) = data.meta.as_ref() {
        if !meta.ailment.is_empty() && meta.ailment_chance > 0 {
            facts.push(format!(
                "{} {}%",
                meta.ailment.as_str(),
                meta.ailment_chance
            ));
        }
        if meta.crit_rate > 0 {
            facts.push(fl!("high-crit"));
        }
        if meta.drain > 0 {
            facts.push(fl!("drain", value = meta.drain));
        } else if meta.drain < 0 {
            facts.push(fl!("recoil", value = (-i16::from(meta.drain))));
        }
        if meta.flinch_chance > 0 {
            facts.push(fl!("flinch", value = meta.flinch_chance));
        }
        if let (Some(min), Some(max)) = (meta.min_hits.as_ref(), meta.max_hits.as_ref()) {
            facts.push(fl!("hits", min = (*min), max = (*max)));
        }
    }
    facts.push(format!(
        "{}: {}",
        fl!("introduced"),
        data.generation.label()
    ));
    info.push(text::caption(facts.join(" · ")).into());

    // Indent under the move name when there is room, otherwise use the full width.
    let indent = if full { 102 } else { 8 };
    widget::column::with_children(vec![
        toggle.into(),
        container(widget::column::with_children(info).spacing(4))
            .padding([4, 16, 8, indent])
            .width(Length::Fill)
            .into(),
    ])
    .into()
}

#[cfg(test)]
mod tests {
    use super::machine_sort_key;

    #[test]
    fn machines_sort_naturally() {
        let mut labels = vec![Some("TR03"), Some("TM24"), None, Some("HM01"), Some("TM05")];
        labels.sort_by_key(|l| machine_sort_key(*l));
        assert_eq!(
            labels,
            vec![Some("TM05"), Some("TM24"), Some("HM01"), Some("TR03"), None]
        );
    }
}
