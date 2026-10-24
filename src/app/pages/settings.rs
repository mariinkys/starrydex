// SPDX-License-Identifier: GPL-3.0

use cosmic::Element;
use cosmic::widget::{self, text};

use crate::app::{AppModel, Message, State};
use crate::config::{AppTheme, ConfigInput, TypeFilteringMode, ViewMode};
use crate::fl;

pub fn view(app: &AppModel) -> Element<'_, Message> {
    let config = &app.config;
    let app_theme_selected = match config.app_theme {
        AppTheme::Dark => 1,
        AppTheme::Light => 2,
        AppTheme::System => 0,
    };
    let type_filter_mode_selected = match config.type_filtering_mode {
        TypeFilteringMode::Inclusive => 1,
        TypeFilteringMode::Exclusive => 0,
    };
    let view_mode_selected = match config.view_mode {
        ViewMode::Responsive => 0,
        ViewMode::Manual { .. } => 1,
    };

    let mut appearance = widget::settings::section()
        .title(fl!("appearance"))
        .add(
            widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                &app.app_themes,
                Some(app_theme_selected),
                |v| Message::ConfigInput(ConfigInput::UpdateTheme(v)),
            )),
        )
        .add(
            widget::settings::item::builder(fl!("view-mode")).control(widget::dropdown(
                &app.view_modes,
                Some(view_mode_selected),
                |v| Message::ConfigInput(ConfigInput::UpdateViewMode(v)),
            )),
        );
    if let ViewMode::Manual { pokemon_per_row } = config.view_mode {
        appearance = appearance.add(
            widget::settings::item::builder(fl!("pokemon-per-row"))
                .description(pokemon_per_row.to_string())
                .control(
                    widget::slider(1..=10, pokemon_per_row as u16, |v| {
                        Message::ConfigInput(ConfigInput::UpdatePokemonPerRow(v))
                    })
                    .step(1u16),
                ),
        );
    }
    appearance = appearance.add(
        widget::settings::item::builder(fl!("pokemon-per-page"))
            .description(config.pokemon_per_page.to_string())
            .control(
                widget::slider(10..=1500, config.pokemon_per_page as u16, |v| {
                    Message::ConfigInput(ConfigInput::UpdatePokemonPerPage(v))
                })
                .step(10u16),
            ),
    );

    let list = widget::settings::section()
        .title(fl!("pokedex-section"))
        .add(
            widget::settings::item::builder(fl!("show-forms"))
                .description(fl!("show-forms-description"))
                .toggler(config.show_forms, |v| {
                    Message::ConfigInput(ConfigInput::UpdateShowForms(v))
                }),
        )
        .add(
            widget::settings::item::builder(fl!("type-filter-mode")).control(widget::dropdown(
                &app.type_filter_modes,
                Some(type_filter_mode_selected),
                |v| Message::ConfigInput(ConfigInput::UpdateTypeFilterMode(v)),
            )),
        );

    let mut data = widget::settings::section().title(fl!("other")).add(
        widget::settings::item::builder(fl!("renew-cache"))
            .description(fl!("renew-cache-description"))
            .control(
                widget::button::destructive(fl!("renew-cache-button"))
                    .on_press(Message::ConfigInput(ConfigInput::DeleteCache)),
            ),
    );
    if let State::Ready(ready) = &app.state {
        let dex = ready.core.dex();
        data = data.add(text::caption(fl!(
            "data-info",
            pokemon = dex.pokemon.len(),
            species = dex.species.len(),
            moves = dex.moves.len(),
            date = format_date(dex.generated_at.to_native())
        )));
    }

    widget::settings::view_column(vec![appearance.into(), list.into(), data.into()]).into()
}

/// Unix timestamp → `YYYY-MM-DD` (UTC), without pulling in a date crate.
fn format_date(timestamp: u64) -> String {
    // Howard Hinnant's civil_from_days
    let days = (timestamp / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_dates() {
        assert_eq!(super::format_date(0), "1970-01-01");
        assert_eq!(super::format_date(1_793_000_000), "2026-10-26");
    }
}
