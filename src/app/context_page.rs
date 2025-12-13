// SPDX-License-Identifier: GPL-3.0

use cosmic::{app::context_drawer, theme};

use crate::{
    app::{AppModel, Message, State},
    fl,
};

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    /// About [`ContextPage`] of the application
    About,
    /// Settings [`ContextPage`] of the application
    Settings,
    /// Pokemon Details [`ContextPage`] of the application
    PokemonDetails,
    /// Pokémon Filtering Options [`ContextPage`] of the application
    FiltersPage,
}

impl ContextPage {
    /// Display the [`ContextPage`]
    pub fn display<'a>(
        &self,
        app_model: &'a AppModel,
    ) -> Option<context_drawer::ContextDrawer<'a, Message>> {
        let spacing = theme::active().cosmic().spacing;

        Some(match &self {
            ContextPage::About => context_drawer::about(
                &app_model.about,
                |s| Message::LaunchUrl(s.to_string()),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::Settings => context_drawer::context_drawer(
                app_model.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
            ContextPage::PokemonDetails => {
                let State::Ready {
                    selected_pokemon,
                    wants_pokemon_details,
                    ..
                } = &app_model.state
                else {
                    return None;
                };

                if let Some(pokemon) = selected_pokemon.as_ref().as_ref() {
                    context_drawer::context_drawer(
                        crate::app::pokemon_details(pokemon, wants_pokemon_details, &spacing),
                        Message::ToggleContextPage(ContextPage::PokemonDetails),
                    )
                    .title(fl!("pokemon-page"))
                } else {
                    return None;
                }
            }
            ContextPage::FiltersPage => {
                let State::Ready { filters, .. } = &app_model.state else {
                    return None;
                };

                context_drawer::context_drawer(
                    crate::app::filters_page(filters, &spacing),
                    Message::ToggleContextPage(ContextPage::FiltersPage),
                )
                .title(fl!("filters-page"))
            }
        })
    }
}
