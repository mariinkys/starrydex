// SPDX-License-Identifier: GPL-3.0

use crate::app::app_menu::MenuAction;
use crate::app::context_page::ContextPage;
use crate::app::core::StarryCore;
use crate::app::entities::{
    PokemonInfo, StarryPokemon, StarryPokemonGeneration, StarryPokemonType,
};
use crate::app::utils::presentation::{capitalize_string, scale_numbers};
use crate::app::utils::{Filters, PaginationAction, remove_dir_contents};
use crate::app::widgets::barchart::BarChart;
use crate::config::{AppTheme, ConfigInput, StarryConfig, TypeFilteringMode, ViewMode};
use crate::key_binds::key_binds;
use crate::{fl, icons, images};
use cosmic::app::context_drawer;
use cosmic::cosmic_theme::Spacing;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Event, Length, Pixels, Subscription};
use cosmic::iced_core::keyboard::{Key, Modifiers};
use cosmic::iced_core::text::LineHeight;
use cosmic::iced_widget::{center, column, row, scrollable};
use cosmic::widget::menu::Action;
use cosmic::widget::{self, about::About, menu};
use cosmic::widget::{
    Column, Grid, Image, JustifyContent, Row, button, checkbox, container, flex_row, search_input,
    text,
};
use cosmic::{prelude::*, theme};
use rkyv::rancor;
use std::collections::HashMap;

pub mod app_menu;
mod context_page;
mod core;
mod entities;
mod utils;
mod widgets;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// The about page for this app.
    about: About,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Application Keyboard Modifiers
    modifiers: Modifiers,
    /// Application configuration handler
    config_handler: Option<cosmic::cosmic_config::Config>,
    /// Configuration data that persists between application runs.
    config: StarryConfig,
    // Application Themes
    app_themes: Vec<String>,
    /// Available Type Filter Modes
    type_filter_modes: Vec<String>,
    /// Available View Modes
    view_modes: Vec<String>,
    /// Application State
    state: State,
}

enum State {
    Loading,
    Error(String),
    Ready {
        /// Application Core (allows us to get pokemons...)
        core: StarryCore,
        /// List of Pokémon to show on the main page
        pokemon_list: Vec<PokemonInfo>,
        /// Holds the data of the currently selected Pokémon to show it on the context page
        selected_pokemon: Box<Option<StarryPokemon>>,
        /// Controls the Pokémon Details Toggle of the Pokémon Context Page
        wants_pokemon_details: bool,
        /// Holds the search input value
        search: String,
        /// Holds the currently applied filters if there are any
        filters: Filters,
        /// Controls in which page are we currently (mainscreen pagination)
        current_page: usize,
    },
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    /// StarryDex HotKey Callback
    Hotkey(Hotkey),
    /// Opens the given URL in the browser
    LaunchUrl(String),
    /// Opens (or closes if already open) the given [`ContextPage`]
    ToggleContextPage(ContextPage),
    /// Update the application config
    UpdateConfig(StarryConfig),
    /// Callback after updating the config, needed in some cases to refresh the data
    ConfigUpdated,
    /// Callback after clicking something in the app menu
    MenuAction(app_menu::MenuAction),
    /// Executes the appropiate cosmic binding on keyboard shortcut
    Key(Modifiers, Key),
    /// Updates the current state of keyboard modifiers
    Modifiers(Modifiers),

    /// Callback after loading the application core
    CoreLoaded(Result<StarryCore, anywho::Error>),

    /// Load the Pokémon with the given id and show it in the Pokémon Details [`ContextPage`]
    LoadPokemon(i64),

    /// Callback after input on the Config [`ContextPage`]
    ConfigInput(ConfigInput),
    /// Callback after input on the Pokémon Details [`ContextPage`]
    PokemonDetailsInput(PokemonDetailsInput),
    /// Callback after input on the on the Pokémon List Page (HomePage)
    PokemonListInput(PokemonListInput),
    /// Callback after input on the Filters [`ContextPage`]
    FiltersInput(FiltersInput),
}

/// Some user interaction that happens on the Pokémon List Page (HomePage)
#[derive(Debug, Clone)]
pub enum PokemonListInput {
    /// Some pagination action has been requested
    PaginationAction(PaginationAction),
    /// Search Input
    SearchInput(String),
    /// Clear currently applied filters
    ClearFilters,
}

/// Some user interaction that happens on the Pokémon Details [`ContextPage`]
#[derive(Debug, Clone)]
pub enum PokemonDetailsInput {
    /// Some pagination action has been requested
    PaginationAction(PaginationAction),
    /// User wants to toggle the pokemon details view
    TogglePokemonDetails(bool),
}

/// Some user interaction that happens on the Filters [`ContextPage`]
#[derive(Debug, Clone)]
pub enum FiltersInput {
    /// User wants to toggle a specific type filter on/off
    TypeFilterToggled(bool, StarryPokemonType),
    /// Toggle stats filter on/off
    StatsFilterToggled(bool),
    /// Stats filter value changed
    StatsFilterChanged(i64),
    /// User wants to toggle a specific generation filter on/off
    GenerationFilterToggled(bool, StarryPokemonGeneration),
    /// Apply the currently selected filters
    ApplyCurrentFilters,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = crate::flags::Flags;

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.mariinkys.StarryDex";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create the about widget
        let about = About::default()
            .name("StarryDex")
            .icon(widget::icon::from_name(Self::APP_ID))
            .version(env!("CARGO_PKG_VERSION"))
            .links([
                (fl!("repository"), REPOSITORY),
                (
                    fl!("support"),
                    "https://github.com/mariinkys/starrydex/issues",
                ),
            ])
            .license(env!("CARGO_PKG_LICENSE"))
            .developers([("mariinkys", "kysdev.owjga@aleeas.com")])
            .comments(format!(
                "{}\n{}\n{}",
                fl!("app-info"),
                fl!("pokeapi-text"),
                fl!("nintendo-text")
            ));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            about,
            key_binds: key_binds(),
            modifiers: Modifiers::empty(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            type_filter_modes: vec![fl!("exclusive"), fl!("inclusive")],
            view_modes: vec![fl!("view-mode-responsive"), fl!("view-mode-manual")],
            state: State::Loading,
        };

        // Startup tasks.
        let tasks = vec![
            app.update_title(),
            cosmic::command::set_theme(app.config.app_theme.theme()),
            Task::perform(StarryCore::initialize(), |res| {
                cosmic::action::app(Message::CoreLoaded(res))
            }),
        ];

        (app, Task::batch(tasks))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")).apply(Element::from),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("about"), None, MenuAction::About),
                    menu::Item::Button(fl!("settings"), None, MenuAction::Settings),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        self.context_page.display(self)
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<_> = match &self.state {
            State::Loading => center(text(fl!("loading"))).into(),
            State::Error(error) => center(text(error)).into(),
            State::Ready {
                pokemon_list,
                search,
                filters,
                current_page,
                ..
            } => {
                let spacing = theme::active().cosmic().spacing;

                homepage(
                    &spacing,
                    pokemon_list,
                    &self.config.view_mode,
                    search,
                    current_page,
                    filters,
                )
            }
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .apply(widget::container)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They can be dynamically
    /// stopped and started conditionally based on application state, or persist
    /// indefinitely.
    fn subscription(&self) -> Subscription<Self::Message> {
        // Add subscriptions which are always active.
        let subscriptions = vec![
            // Watch for key_bind inputs (cosmic)
            cosmic::iced::event::listen_with(|event, status, _| match event {
                Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) => match status {
                    cosmic::iced::event::Status::Ignored => Some(Message::Key(modifiers, key)),
                    cosmic::iced::event::Status::Captured => None,
                },
                Event::Keyboard(cosmic::iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<StarryConfig>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
            // Application HoyKeys
            cosmic::iced::event::listen_with(handle_event),
        ];

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Hotkey(hotkey) => {
                let State::Ready { .. } = &mut self.state else {
                    return Task::none();
                };

                if self.context_page == ContextPage::PokemonDetails && self.core.window.show_context
                {
                    // We're on the Pokémon details page
                    match hotkey {
                        Hotkey::ArrowLeft => self.update(Message::PokemonDetailsInput(
                            PokemonDetailsInput::PaginationAction(PaginationAction::Back),
                        )),
                        Hotkey::ArrowRight => self.update(Message::PokemonDetailsInput(
                            PokemonDetailsInput::PaginationAction(PaginationAction::Next),
                        )),
                    }
                } else if !self.core.window.show_context {
                    // We're on the main page
                    match hotkey {
                        Hotkey::ArrowLeft => self.update(Message::PokemonListInput(
                            PokemonListInput::PaginationAction(PaginationAction::Back),
                        )),
                        Hotkey::ArrowRight => self.update(Message::PokemonListInput(
                            PokemonListInput::PaginationAction(PaginationAction::Next),
                        )),
                    }
                } else {
                    Task::none()
                }
            }
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                Task::none()
            }

            Message::UpdateConfig(config) => {
                self.config = config;
                let State::Ready {
                    core,
                    current_page,
                    pokemon_list,
                    ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                *current_page = 0;
                *pokemon_list = core.get_pokemon_page(0, self.config.pokemon_per_page);

                cosmic::command::set_theme(self.config.app_theme.theme())
            }
            Message::ConfigUpdated => {
                let State::Ready {
                    core,
                    current_page,
                    pokemon_list,
                    ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                *current_page = 0;
                *pokemon_list = core.get_pokemon_page(0, self.config.pokemon_per_page);
                Task::none()
            }
            Message::LaunchUrl(url) => {
                match open::that_detached(&url) {
                    Ok(()) => {}
                    Err(err) => {
                        eprintln!("failed to open {url:?}: {err}");
                    }
                }
                Task::none()
            }
            Message::MenuAction(action) => {
                let State::Ready { .. } = &mut self.state else {
                    return Task::none();
                };

                match action {
                    app_menu::MenuAction::About => {
                        self.update(Message::ToggleContextPage(ContextPage::About))
                    }
                    app_menu::MenuAction::Settings => {
                        self.update(Message::ToggleContextPage(ContextPage::Settings))
                    }
                }
            }
            Message::Key(modifiers, key) => {
                for (key_bind, action) in self.key_binds.iter() {
                    if key_bind.matches(modifiers, &key) {
                        return self.update(action.message());
                    }
                }
                Task::none()
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }

            Message::CoreLoaded(res) => {
                if let Err(e) = res {
                    self.state = State::Error(e.to_string());
                    return Task::none();
                }

                let core = res.unwrap();
                let pokemon_list = core.get_pokemon_page(0, self.config.pokemon_per_page);

                self.state = State::Ready {
                    core,
                    pokemon_list,
                    selected_pokemon: Box::from(None),
                    wants_pokemon_details: false,
                    search: String::new(),
                    filters: Filters::default(),
                    current_page: 0,
                };

                Task::none()
            }

            Message::LoadPokemon(pokemon_id) => {
                let State::Ready {
                    core,
                    selected_pokemon,
                    ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                let pokemon = core.get_pokemon_by_id(pokemon_id).map(|archived_pokemon| {
                    // It is theoretically safe to unwrap here
                    rkyv::deserialize::<StarryPokemon, rancor::Error>(archived_pokemon).unwrap()
                });

                *selected_pokemon = Box::from(pokemon);
                self.context_page = ContextPage::PokemonDetails;
                self.core.window.show_context = true;

                Task::none()
            }

            Message::ConfigInput(input) => {
                match input {
                    ConfigInput::UpdateTheme(index) => {
                        let app_theme = match index {
                            1 => AppTheme::Dark,
                            2 => AppTheme::Light,
                            _ => AppTheme::System,
                        };

                        if let Some(handler) = &self.config_handler {
                            if let Err(err) = self.config.set_app_theme(handler, app_theme) {
                                eprintln!("{err}");
                                // even if it fails we update the config (it won't get saved after restart)
                                let mut old_config = self.config.clone();
                                old_config.app_theme = app_theme;
                                self.config = old_config;
                            }

                            return cosmic::command::set_theme(self.config.app_theme.theme());
                        }
                        Task::none()
                    }
                    ConfigInput::UpdateViewMode(index) => {
                        let per_row_value =
                            if let ViewMode::Manual { pokemon_per_row } = &self.config.view_mode {
                                pokemon_per_row
                            } else {
                                &3
                            };

                        let view_mode = match index {
                            0 => ViewMode::Responsive,
                            1 => ViewMode::Manual {
                                pokemon_per_row: *per_row_value,
                            },
                            _ => ViewMode::Responsive,
                        };

                        #[allow(clippy::collapsible_if)]
                        if let Some(handler) = &self.config_handler {
                            if let Err(err) = self.config.set_view_mode(handler, view_mode) {
                                eprintln!("{err}");
                                // even if it fails we update the config (it won't get saved after restart)
                                let mut old_config = self.config.clone();
                                old_config.view_mode = view_mode;
                                self.config = old_config;
                            }
                        }
                        Task::none()
                    }
                    ConfigInput::UpdatePokemonPerRow(v) => {
                        let ViewMode::Manual { .. } = &mut self.config.view_mode else {
                            return Task::none();
                        };

                        if let Some(handler) = &self.config_handler {
                            let value = ViewMode::Manual {
                                pokemon_per_row: v as usize,
                            };
                            if let Err(err) = self.config.set_view_mode(handler, value) {
                                eprintln!("{err}");
                                // even if it fails we update the config (it won't get saved after restart)
                                let mut old_config = self.config.clone();
                                old_config.view_mode = value;
                                self.config = old_config;
                            }
                        }
                        Task::none()
                    }
                    ConfigInput::UpdatePokemonPerPage(v) => {
                        if let Some(handler) = &self.config_handler {
                            let value = v as usize;
                            if let Err(err) = self.config.set_pokemon_per_page(handler, value) {
                                eprintln!("{err}");
                                // even if it fails we update the config (it won't get saved after restart)
                                let mut old_config = self.config.clone();
                                old_config.pokemon_per_page = value;
                                self.config = old_config;
                            }
                            return self.update(Message::ConfigUpdated);
                        }
                        Task::none()
                    }
                    ConfigInput::UpdateTypeFilterMode(index) => {
                        let filter_mode = match index {
                            1 => TypeFilteringMode::Inclusive,
                            _ => TypeFilteringMode::Exclusive,
                        };

                        #[allow(clippy::collapsible_if)]
                        if let Some(handler) = &self.config_handler {
                            if let Err(err) =
                                self.config.set_type_filtering_mode(handler, filter_mode)
                            {
                                eprintln!("{err}");
                                // even if it fails we update the config (it won't get saved after restart)
                                let mut old_config = self.config.clone();
                                old_config.type_filtering_mode = filter_mode;
                                self.config = old_config;
                            }
                        }
                        Task::none()
                    }
                    ConfigInput::DeleteCache => {
                        let State::Ready { .. } = &mut self.state else {
                            return Task::none();
                        };
                        self.set_show_context(false);

                        let data_dir = dirs::data_dir().unwrap().join(Self::APP_ID);
                        if let Err(e) = remove_dir_contents(&data_dir) {
                            eprintln!("Error deleting cache: {e}");
                        }

                        Task::perform(StarryCore::initialize(), |core| {
                            cosmic::action::app(Message::CoreLoaded(core))
                        })
                    }
                }
            }

            Message::PokemonDetailsInput(input) => {
                let State::Ready {
                    core,
                    pokemon_list,
                    selected_pokemon,
                    wants_pokemon_details,
                    ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                match input {
                    PokemonDetailsInput::PaginationAction(action) => match action {
                        PaginationAction::Next => {
                            #[allow(clippy::collapsible_if)]
                            if !pokemon_list.is_empty() {
                                if let Some(current_pokemon) = selected_pokemon.as_ref() {
                                    if let Some(current_index) = &pokemon_list
                                        .iter()
                                        .position(|p| p.id == current_pokemon.pokemon.id)
                                    {
                                        let next_index = (current_index + 1) % pokemon_list.len();
                                        let next_pokemon_id = pokemon_list[next_index].id;
                                        let pokemon = core.get_pokemon_by_id(next_pokemon_id).map(
                                            |archived_pokemon| {
                                                // It is theoretically safe to unwrap here
                                                rkyv::deserialize::<StarryPokemon, rancor::Error>(
                                                    archived_pokemon,
                                                )
                                                .unwrap()
                                            },
                                        );

                                        *selected_pokemon = Box::from(pokemon)
                                    }
                                }
                            }
                        }
                        PaginationAction::Back => {
                            #[allow(clippy::collapsible_if)]
                            if !pokemon_list.is_empty() {
                                if let Some(current_pokemon) = selected_pokemon.as_ref() {
                                    if let Some(current_index) = &pokemon_list
                                        .iter()
                                        .position(|p| p.id == current_pokemon.pokemon.id)
                                    {
                                        let prev_index = if *current_index == 0 {
                                            pokemon_list.len() - 1
                                        } else {
                                            current_index - 1
                                        };
                                        let prev_pokemon_id = pokemon_list[prev_index].id;
                                        let pokemon = core.get_pokemon_by_id(prev_pokemon_id).map(
                                            |archived_pokemon| {
                                                // It is theoretically safe to unwrap here
                                                rkyv::deserialize::<StarryPokemon, rancor::Error>(
                                                    archived_pokemon,
                                                )
                                                .unwrap()
                                            },
                                        );

                                        *selected_pokemon = Box::from(pokemon)
                                    }
                                }
                            }
                        }
                    },
                    PokemonDetailsInput::TogglePokemonDetails(value) => {
                        *wants_pokemon_details = value;
                    }
                }

                Task::none()
            }

            Message::PokemonListInput(input) => {
                let State::Ready {
                    core,
                    pokemon_list,
                    filters,
                    current_page,
                    search,
                    ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                match input {
                    PokemonListInput::PaginationAction(action) => match action {
                        PaginationAction::Next => {
                            if !filters.any_applied() && search.is_empty() {
                                let new_list = core.get_pokemon_page(
                                    (*current_page + 1) * self.config.pokemon_per_page,
                                    self.config.pokemon_per_page,
                                );
                                if !new_list.is_empty() {
                                    *current_page += 1;
                                    *pokemon_list = new_list;
                                }
                            }
                        }
                        PaginationAction::Back =>
                        {
                            #[allow(clippy::collapsible_if)]
                            if *current_page >= 1 {
                                if !filters.any_applied() && search.is_empty() {
                                    let new_list = core.get_pokemon_page(
                                        (*current_page - 1) * self.config.pokemon_per_page,
                                        self.config.pokemon_per_page,
                                    );
                                    if !new_list.is_empty() {
                                        *current_page -= 1;
                                        *pokemon_list = new_list;
                                    }
                                }
                            }
                        }
                    },
                    PokemonListInput::SearchInput(value) => {
                        *search = value;
                        if search.is_empty() {
                            *pokemon_list = core.get_pokemon_page(
                                *current_page * self.config.pokemon_per_page,
                                self.config.pokemon_per_page,
                            );
                        } else {
                            *pokemon_list = core.search_pokemon(search);
                        }
                    }
                    PokemonListInput::ClearFilters => {
                        // TODO: Is this better than before, when we we're just restarting all fields except core manualy?
                        self.state = State::Loading;
                        return Task::perform(StarryCore::initialize(), |res| {
                            cosmic::action::app(Message::CoreLoaded(res))
                        });
                    }
                }

                Task::none()
            }

            Message::FiltersInput(input) => {
                let State::Ready {
                    core,
                    filters,
                    search,
                    current_page,
                    pokemon_list,
                    ..
                } = &mut self.state
                else {
                    return Task::none();
                };

                match input {
                    FiltersInput::TypeFilterToggled(value, pokemon_type) => {
                        if value {
                            filters.selected_types.insert(pokemon_type);
                        } else {
                            filters.selected_types.remove(&pokemon_type);
                        }
                        Task::none()
                    }
                    FiltersInput::StatsFilterToggled(value) => {
                        filters.total_stats.0 = value;
                        Task::none()
                    }
                    FiltersInput::StatsFilterChanged(value) => {
                        if filters.total_stats.0 {
                            filters.total_stats.1 = value;
                        }
                        Task::none()
                    }
                    FiltersInput::GenerationFilterToggled(value, pokemon_generation) => {
                        if value {
                            filters.selected_generations.insert(pokemon_generation);
                        } else {
                            filters.selected_generations.remove(&pokemon_generation);
                        }
                        Task::none()
                    }
                    FiltersInput::ApplyCurrentFilters => {
                        if filters.any_applied() {
                            *search = String::new();
                            *current_page = 0;

                            let mut all_pokemon = core.get_pokemon_list();

                            // Try to apply types filter if needed
                            if !filters.selected_types.is_empty() {
                                match self.config.type_filtering_mode {
                                    TypeFilteringMode::Inclusive => {
                                        // Ej: If fire and ice are selected it will show fire pokemons and ice pokemons
                                        all_pokemon =
                                            core.filter_pokemon_inclusive(&filters.selected_types);
                                    }
                                    TypeFilteringMode::Exclusive => {
                                        // Ej: If fire and ice are selected it will show pokemons that are both fire and ice types
                                        all_pokemon =
                                            core.filter_pokemon_exclusive(&filters.selected_types);
                                    }
                                }
                            }

                            // Try to apply stats filter if needed
                            if filters.total_stats.0 && filters.total_stats.1 > 0 {
                                all_pokemon = core.filter_pokemon_stats_with_list(
                                    &all_pokemon,
                                    filters.total_stats.1,
                                );
                            }

                            // Try to apply generations filter if needed
                            if !filters.selected_generations.is_empty() {
                                all_pokemon = core.filter_pokemon_by_generation(
                                    &all_pokemon,
                                    &filters.selected_generations,
                                );
                            }

                            *pokemon_list = all_pokemon;
                        }

                        self.core.window.show_context = false;

                        Task::none()
                    }
                }
            }
        }
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let window_title = String::from("StarryDex");

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    /// The settings context page for this app.
    pub fn settings(&self) -> Element<Message> {
        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };
        let type_filter_mode_selected = match self.config.type_filtering_mode {
            TypeFilteringMode::Inclusive => 1,
            TypeFilteringMode::Exclusive => 0,
        };
        let view_mode_selected = match self.config.view_mode {
            ViewMode::Responsive => 0,
            ViewMode::Manual { .. } => 1,
        };

        // Appearance Section
        let mut appearance_section = widget::settings::section()
            .title(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    |v| Message::ConfigInput(ConfigInput::UpdateTheme(v)),
                )),
            )
            .add(
                widget::settings::item::builder(fl!("view-mode")).control(widget::dropdown(
                    &self.view_modes,
                    Some(view_mode_selected),
                    |v| Message::ConfigInput(ConfigInput::UpdateViewMode(v)),
                )),
            );
        // Conditionally add pokemon-per-row slider if ViewMode::Manual is selected
        if let ViewMode::Manual { pokemon_per_row } = self.config.view_mode {
            appearance_section = appearance_section.add(
                widget::settings::item::builder(fl!("pokemon-per-row"))
                    .description(format!("{}", pokemon_per_row))
                    .control(
                        widget::slider(1..=10, pokemon_per_row as u16, |v| {
                            Message::ConfigInput(ConfigInput::UpdatePokemonPerRow(v))
                        })
                        .step(1u16),
                    ),
            );
        }
        // Add pokemon-per-page slider
        appearance_section = appearance_section.add(
            widget::settings::item::builder(fl!("pokemon-per-page"))
                .description(format!("{}", self.config.pokemon_per_page))
                .control(
                    widget::slider(10..=1500, self.config.pokemon_per_page as u16, |v| {
                        Message::ConfigInput(ConfigInput::UpdatePokemonPerPage(v))
                    })
                    .step(10u16),
                ),
        );

        widget::settings::view_column(vec![
            appearance_section.into(),
            widget::settings::section()
                .title(fl!("other"))
                .add(
                    widget::settings::item::builder(fl!("type-filter-mode")).control(
                        widget::dropdown(
                            &self.type_filter_modes,
                            Some(type_filter_mode_selected),
                            |v| Message::ConfigInput(ConfigInput::UpdateTypeFilterMode(v)),
                        ),
                    ),
                )
                .add(
                    widget::settings::item::builder(fl!("renew-cache")).control(
                        widget::button::destructive(fl!("renew-cache-button"))
                            .on_press(Message::ConfigInput(ConfigInput::DeleteCache)),
                    ),
                )
                .into(),
        ])
        .into()
    }
}

/// The pokemon details context page for this app.
pub fn homepage<'a>(
    spacing: &Spacing,
    pokemon_list: &'a [PokemonInfo],
    view_mode: &'a ViewMode,
    search: &'a str,
    current_page: &'a usize,
    current_filters: &'a Filters,
) -> Element<'a, Message> {
    let pokemon_content: Element<Message> = match view_mode {
        ViewMode::Manual { pokemon_per_row } => {
            let mut pokemon_grid = Grid::new().width(Length::Fill);

            for (index, pokemon) in pokemon_list.iter().enumerate() {
                let pokemon_image = match pokemon.sprite_path.as_ref() {
                    Some(path) => Image::new(path.as_str()),
                    None => Image::new(images::get("fallback")),
                }
                .content_fit(cosmic::iced::ContentFit::None)
                .width(Length::Fixed(100.0))
                .height(Length::Fixed(100.0));

                let pokemon_container = button::custom(
                    Column::new()
                        .push(pokemon_image.width(Length::Shrink))
                        .push(
                            text(capitalize_string(&pokemon.name))
                                .width(Length::Shrink)
                                .font(cosmic::iced::Font {
                                    weight: cosmic::iced::font::Weight::Bold,
                                    ..Default::default()
                                })
                                .line_height(LineHeight::Absolute(Pixels::from(15.0))),
                        )
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                )
                .width(Length::Fixed(200.0))
                .height(Length::Fixed(135.0))
                .on_press_down(Message::LoadPokemon(pokemon.id))
                .class(theme::Button::IconVertical);

                // Insert a new row before adding the first Pokémon of each row
                if index % pokemon_per_row == 0 {
                    pokemon_grid = pokemon_grid.insert_row();
                }

                pokemon_grid = pokemon_grid.push(pokemon_container);
            }

            pokemon_grid.into()
        }
        ViewMode::Responsive => {
            let pokemon_items: Vec<Element<Message>> = pokemon_list
                .iter()
                .map(|pokemon| {
                    let pokemon_image = match pokemon.sprite_path.as_ref() {
                        Some(path) => Image::new(path.as_str()),
                        None => Image::new(images::get("fallback")),
                    }
                    .content_fit(cosmic::iced::ContentFit::None)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0));

                    button::custom(
                        Column::new()
                            .push(pokemon_image.width(Length::Shrink))
                            .push(
                                text(capitalize_string(&pokemon.name))
                                    .width(Length::Shrink)
                                    .font(cosmic::iced::Font {
                                        weight: cosmic::iced::font::Weight::Bold,
                                        ..Default::default()
                                    })
                                    .line_height(LineHeight::Absolute(Pixels::from(15.0))),
                            )
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                    )
                    .width(Length::Fixed(200.0))
                    .height(Length::Fixed(135.0))
                    .on_press_down(Message::LoadPokemon(pokemon.id))
                    .class(theme::Button::IconVertical)
                    .into()
                })
                .collect();

            flex_row(pokemon_items)
                .row_spacing(spacing.space_xs)
                .column_spacing(spacing.space_xs)
                .width(Length::Fill)
                .into()
        }
    };

    column![
        // SEARCH ROW
        row![
            search_input(fl!("search"), search)
                .style(theme::TextInput::Search)
                .on_input(|v| Message::PokemonListInput(PokemonListInput::SearchInput(v)))
                .line_height(LineHeight::Absolute(Pixels(30.0)))
                .width(Length::Fill),
            button::icon(icons::get_handle("filter-symbolic", 18))
                .class(theme::Button::Suggested)
                .on_press(Message::ToggleContextPage(ContextPage::FiltersPage))
                .width(Length::Shrink),
            button::icon(icons::get_handle("edit-clear-all-symbolic", 18))
                .class(theme::Button::Destructive)
                .on_press(Message::PokemonListInput(PokemonListInput::ClearFilters))
                .width(Length::Shrink)
        ]
        .spacing(spacing.space_xxxs)
        .width(Length::Fill),
        // POKEMON LIST
        scrollable(container(pokemon_content).align_x(Alignment::Center))
            .height(Length::FillPortion(8))
            .width(Length::Fill),
        // PAGINATION
        container(
            row![
                widget::button::icon(icons::get_handle("go-previous-symbolic", 18)).on_press_maybe(
                    (!current_filters.any_applied()).then_some(Message::PokemonListInput(
                        PokemonListInput::PaginationAction(PaginationAction::Back),
                    )),
                ),
                text(format!("{} - {}", fl!("page"), (current_page + 1))),
                widget::button::icon(icons::get_handle("go-next-symbolic", 18)).on_press_maybe(
                    (!current_filters.any_applied()).then_some(Message::PokemonListInput(
                        PokemonListInput::PaginationAction(PaginationAction::Next),
                    )),
                )
            ]
            .spacing(spacing.space_xxl)
            .width(Length::Shrink)
            .align_y(Alignment::Center)
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
    ]
    .width(Length::Fill)
    .padding(5.)
    .spacing(spacing.space_xxs)
    .into()
}

/// The pokemon details context page for this app.
pub fn pokemon_details<'a>(
    starry_pokemon: &'a StarryPokemon,
    wants_pokemon_details: &'a bool,
    spacing: &Spacing,
) -> Element<'a, Message> {
    let show_details = checkbox(fl!("show-encounter-details"), *wants_pokemon_details)
        .on_toggle(|v| Message::PokemonDetailsInput(PokemonDetailsInput::TogglePokemonDetails(v)));

    let encounter_info = match &starry_pokemon.encounter_info {
        Some(info) => {
            let children = info.iter().map(|ef| {
                widget::Column::new()
                    .width(Length::Fill)
                    .push(
                        text(capitalize_string(&ef.city))
                            .class(theme::Text::Accent)
                            .size(15.),
                    )
                    .extend(ef.games_method.iter().map(|method| text(method).into()))
                    .into()
            });
            widget::container(Column::with_children(children))
                .class(theme::Container::Card)
                .padding([spacing.space_none, spacing.space_xxs])
        }
        None => widget::container(text(fl!("no-encounter-info"))).class(theme::Container::Card),
    };

    let has_encounters = starry_pokemon
        .encounter_info
        .as_ref()
        .is_some_and(|info| !info.is_empty());

    Column::new()
        // TITLE
        .push(
            container(
                column![
                    row![
                        button::icon(icons::get_handle("go-previous-symbolic", 18)).on_press(
                            Message::PokemonDetailsInput(PokemonDetailsInput::PaginationAction(
                                PaginationAction::Back
                            ))
                        ),
                        text::title1(capitalize_string(starry_pokemon.pokemon.name.as_str())),
                        button::icon(icons::get_handle("go-next-symbolic", 18)).on_press(
                            Message::PokemonDetailsInput(PokemonDetailsInput::PaginationAction(
                                PaginationAction::Next
                            ))
                        )
                    ]
                    .spacing(spacing.space_s)
                    .align_y(Alignment::Center),
                    text::title4(format!(
                        "#{} {}",
                        &starry_pokemon.pokemon.id,
                        &starry_pokemon
                            .specie
                            .as_ref()
                            .map(|s| format!("- {}", s.generation))
                            .unwrap_or_default()
                    ))
                ]
                .align_x(Alignment::Center)
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .align_x(Alignment::Center),
        )
        // IMAGE (SPRITE)
        .push(if let Some(path) = &starry_pokemon.sprite_path {
            Image::new(path).content_fit(cosmic::iced::ContentFit::Fill)
        } else {
            Image::new(images::get("fallback")).content_fit(cosmic::iced::ContentFit::Fill)
        })
        // POKÉMON TYPES
        .push(
            container(Row::new().spacing(spacing.space_s).extend(
                starry_pokemon.pokemon.types.iter().map(|poke_type| {
                    widget::tooltip(
                        widget::icon(icons::get_handle_owned(poke_type.icon_name(), 18)),
                        text(capitalize_string(&poke_type.to_string())),
                        widget::tooltip::Position::Bottom,
                    )
                    .into()
                }),
            ))
            .align_x(Alignment::Center),
        )
        // WEIGHT & HEIGHT
        .push(
            row![
                container(
                    column![
                        text::title3(fl!("weight")),
                        text(format!(
                            "{} Kg",
                            scale_numbers(starry_pokemon.pokemon.weight)
                        ))
                        .size(15.)
                    ]
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                )
                .class(theme::Container::Card)
                .padding([spacing.space_none, spacing.space_xxs]),
                container(
                    column![
                        text::title3(fl!("height")),
                        text(format!(
                            "{} m",
                            scale_numbers(starry_pokemon.pokemon.height)
                        ))
                        .size(15.)
                    ]
                    .align_x(Alignment::Center)
                    .width(Length::Fill),
                )
                .class(theme::Container::Card)
                .padding([spacing.space_none, spacing.space_xxs])
            ]
            .spacing(8.)
            .align_y(Alignment::Center),
        )
        // POKÉMON ABILITIES
        .push(
            widget::container(
                widget::Column::new()
                    .push(
                        widget::text::title3(fl!("pokemon-abilities"))
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                    )
                    .extend(starry_pokemon.pokemon.abilities.iter().map(|ability| {
                        text(capitalize_string(ability))
                            .width(Length::Fill)
                            .align_x(Alignment::Center)
                            .into()
                    })),
            )
            .width(Length::Fill)
            .class(theme::Container::Card)
            .padding([spacing.space_none, spacing.space_xxs]),
        )
        // POKÉMON STATS
        .push(
            container(column![
                widget::text::title3(fl!("poke-stats"))
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
                BarChart::new()
                    .column_spacing(3.)
                    .padding(3.)
                    .push(fl!("hp"), starry_pokemon.pokemon.stats.hp as f32)
                    .push(fl!("attack"), starry_pokemon.pokemon.stats.attack as f32)
                    .push(fl!("defense"), starry_pokemon.pokemon.stats.defense as f32)
                    .push(fl!("sp-a"), starry_pokemon.pokemon.stats.sp_attack as f32)
                    .push(fl!("sp-d"), starry_pokemon.pokemon.stats.sp_defense as f32)
                    .push(fl!("spd"), starry_pokemon.pokemon.stats.speed as f32),
                widget::text(format!(
                    "{}: {}",
                    fl!("total"),
                    starry_pokemon.get_total_stats()
                ))
                .width(Length::Fill)
                .align_x(Alignment::Center)
            ])
            .padding(10.)
            .class(theme::Container::Card),
        )
        // EVOLUTION DATA
        .push(
            container(evolution_data_view(starry_pokemon))
                .align_x(Alignment::Center)
                .padding(10.)
                .class(theme::Container::Card),
        )
        // ENCOUNTER DATA (IF ANY)
        .extend(
            [
                has_encounters.then(|| show_details.into()),
                (has_encounters && *wants_pokemon_details).then(|| encounter_info.into()),
                Some(
                    widget::button::link(format!("{} (Bulbapedia)", fl!("link-more-info")))
                        .on_press(Message::LaunchUrl(format!(
                            "https://bulbapedia.bulbagarden.net/w/index.php?search={}",
                            &starry_pokemon.pokemon.name
                        )))
                        .into(),
                ),
            ]
            .into_iter()
            .flatten(),
        )
        .align_x(Alignment::Center)
        .height(Length::Shrink)
        .spacing(10.0)
        .into()
}

/// The filters context page for this app.
pub fn filters_page<'a>(filters: &'a Filters, _spacing: &Spacing) -> Element<'a, Message> {
    let mut generations_column = Column::new()
        .push(widget::text::title3(fl!("generation-filters")))
        .spacing(5)
        .width(Length::Fill);

    for chunk in StarryPokemonGeneration::ALL.chunks(2) {
        let mut row = widget::Row::new();
        for generation in chunk {
            let is_checked = filters.selected_generations.contains(generation);
            let checkbox: Element<Message> = checkbox(generation.to_string(), is_checked)
                .on_toggle(move |v| {
                    Message::FiltersInput(FiltersInput::GenerationFilterToggled(
                        v,
                        generation.clone(),
                    ))
                })
                .into();

            row = row.push(widget::container(checkbox).width(Length::Fill));
        }
        generations_column = generations_column.push(row);
    }

    let mut types_column = Column::new()
        .push(widget::text::title3(fl!("type-filters")))
        .spacing(5)
        .width(Length::Fill);

    for chunk in StarryPokemonType::ALL.chunks(2) {
        let mut row = widget::Row::new();
        for pokemon_type in chunk {
            let is_checked = filters.selected_types.contains(pokemon_type);
            let checkbox: Element<Message> = checkbox(pokemon_type.to_string(), is_checked)
                .on_toggle(move |v| {
                    Message::FiltersInput(FiltersInput::TypeFilterToggled(v, pokemon_type.clone()))
                })
                .into();

            row = row.push(widget::container(checkbox).width(Length::Fill));
        }
        types_column = types_column.push(row);
    }

    let poke_stats_column = column![
        widget::text::title3(fl!("stats-filter")),
        widget::Row::new()
            .push(
                checkbox(fl!("enabled"), filters.total_stats.0)
                    .on_toggle(|v| Message::FiltersInput(FiltersInput::StatsFilterToggled(v)))
                    .width(Length::Fill),
            )
            .push(
                column![
                    text(format!(
                        "{}: {}",
                        fl!("minimum-poke-stats"),
                        &filters.total_stats.1
                    )),
                    widget::slider(
                        0.0..=800.0,
                        filters.total_stats.1 as f64,
                        move |new_value| Message::FiltersInput(FiltersInput::StatsFilterChanged(
                            new_value as i64
                        )),
                    )
                    .step(10.0)
                ]
                .spacing(2.),
            )
            .align_y(Alignment::Center)
            .width(Length::Fill)
    ];

    container(
        column![
            types_column,
            generations_column,
            poke_stats_column,
            container(
                button::suggested(fl!("apply-filters"))
                    .on_press(Message::FiltersInput(FiltersInput::ApplyCurrentFilters))
                    .width(Length::Shrink)
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center)
        ]
        .width(Length::Fill)
        .spacing(15.),
    )
    .into()
}

//
// VIEW HELPERS
//

fn evolution_data_view<'a>(starry_pokemon: &'a StarryPokemon) -> Element<'a, Message> {
    if let Some(specie) = &starry_pokemon.specie
        && !specie.evolution_data.is_empty()
    {
        let mut evo_items = Vec::new();

        for data in &specie.evolution_data {
            let pokemon_image = {
                let image = if let Some(path) = &data.sprite_path {
                    widget::Image::new(path).content_fit(cosmic::iced::ContentFit::Fill)
                } else {
                    widget::Image::new(images::get("fallback"))
                        .content_fit(cosmic::iced::ContentFit::Fill)
                };
                widget::tooltip(
                    widget::mouse_area(image).on_press(Message::LoadPokemon(data.id)),
                    widget::text(data.name.to_owned()),
                    widget::tooltip::Position::Top,
                )
            };

            // Group arrow and image together in a row
            let item = if let Some(n) = &data.needs_to_evolve {
                row![
                    container(widget::tooltip(
                        widget::icon(icons::get_handle("go-next-symbolic", 18)),
                        widget::text(n.to_owned()),
                        widget::tooltip::Position::Top,
                    ))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .height(Length::Fixed(96.0)),
                    pokemon_image
                ]
                .align_y(Alignment::Center)
                .spacing(5.)
                .into()
            } else {
                pokemon_image.into()
            };

            evo_items.push(item);
        }

        let evo_data_row = widget::flex_row(evo_items)
            .align_items(Alignment::Center)
            .justify_content(JustifyContent::SpaceEvenly);

        column![
            widget::text::title3(fl!("poke-evo-data"))
                .width(Length::Fill)
                .align_x(Alignment::Center),
            container(evo_data_row)
                .align_x(Alignment::Center)
                .width(Length::Fill),
        ]
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    } else {
        column![
            widget::text::title3(fl!("poke-evo-data"))
                .width(Length::Fill)
                .align_x(Alignment::Center),
            widget::text(fl!("no-evo-data")),
        ]
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .into()
    }
}

//
// SUBSCRIPTION HANDLING
//

#[derive(Debug, Clone)]
pub enum Hotkey {
    ArrowLeft,
    ArrowRight,
}

fn handle_event(
    event: cosmic::iced::event::Event,
    _: cosmic::iced::event::Status,
    _: cosmic::iced::window::Id,
) -> Option<Message> {
    match event {
        #[allow(clippy::collapsible_match)]
        cosmic::iced::event::Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
            key,
            ..
        }) => match key {
            cosmic::iced::keyboard::Key::Named(cosmic::iced::keyboard::key::Named::ArrowRight) => {
                Some(Message::Hotkey(Hotkey::ArrowRight))
            }
            cosmic::iced::keyboard::Key::Named(cosmic::iced::keyboard::key::Named::ArrowLeft) => {
                Some(Message::Hotkey(Hotkey::ArrowLeft))
            }
            _ => None,
        },
        _ => None,
    }
}
