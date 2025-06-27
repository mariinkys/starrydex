// SPDX-License-Identifier: GPL-3.0-only

use crate::config::{AppTheme, StarryConfig, TypeFilteringMode};
use crate::core::StarryCore;
use crate::entities::{PokemonInfo, PokemonType, StarryPokemon};
use crate::image_cache::ImageCache;
use crate::utils::{capitalize_string, remove_dir_contents, scale_numbers};
use crate::widgets::barchart::BarChart;
use crate::{fl, icon_cache};
use anywho::Error;
use cosmic::app::context_drawer;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Pixels, Subscription};
use cosmic::iced_core::text::LineHeight;
use cosmic::iced_widget::{Row, column, text};
use cosmic::prelude::*;
use cosmic::theme;
use cosmic::widget::about::About;
use cosmic::widget::{self, Column, container, menu};
use rkyv::rancor;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

const REPOSITORY: &str = "https://github.com/mariinkys/starrydex";

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct StarryDex {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Application about page
    about: About,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Application configuration handler
    config_handler: Option<cosmic::cosmic_config::Config>,
    /// Configuration data that persists between application runs.
    config: StarryConfig,
    /// Application Themes
    app_themes: Vec<String>,
    // Core StarryDex Client
    starry_core: Option<StarryCore>,
    /// List of Pokémon to show on the main page
    pokemon_list: Vec<PokemonInfo>,
    /// Holds the data of the currently selected Pokémon to show it on the context page
    selected_pokemon: Option<StarryPokemon>,
    /// Status of the main application page
    current_page_status: PageStatus,
    /// Controls the Pokémon Details Toggle of the Pokémon Context Page
    wants_pokemon_details: bool,
    /// Holds the search input value
    search: String,
    /// Holds the currently applied filters if there are any
    filters: Filters,
    /// Type Filter Modes
    type_filter_mode: Vec<String>,
    /// Controls in which page are we currently
    current_page: usize,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(StarryConfig),
    UpdateTheme(usize),
    UpdateTypeFilterMode(usize),

    LoadPokemon(i64),
    TogglePokemonDetails(bool),
    SearchInput(String),
    ApplyCurrentFilters,
    ClearFilters,
    DeleteCache,
    PaginationActionRequested(PaginationAction),
    SinglePokemonPagination(PaginationAction),

    InitializedCore(Result<StarryCore, Error>),
    TypeFilterToggled(bool, PokemonType),
    StatsFilterToggled(bool),
    StatsFilterUpdated(i64),
}

/// Identifies an action related to Pagination
#[derive(Debug, Clone)]
pub enum PaginationAction {
    Next,
    Back,
}

/// Different filters you can apply to the Pokémon List
pub struct Filters {
    pub selected_types: HashSet<PokemonType>,
    pub total_stats: (bool, i64),
}

/// Identifies the status of a page in the application.
#[derive(PartialEq)]
pub enum PageStatus {
    FirstRun,
    Loaded,
    Loading,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for StarryDex {
    /// The async executor that will be used to run your application's tasks.
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

    /// Initializes the application with any given flags and startup tasks.
    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Tasks that will get executed on the application init
        let mut tasks = vec![];

        // Application about page
        let about = About::default()
            .name(fl!("app-title"))
            .icon(Self::APP_ID)
            .version(env!("CARGO_PKG_VERSION"))
            .author("mariinkys")
            .license("GPL-3.0-only")
            .links([
                (fl!("repository"), REPOSITORY),
                (
                    fl!("support"),
                    "https://github.com/mariinkys/starrydex/issues",
                ),
            ])
            .developers([("mariinkys", "kysdev.owjga@aleeas.com")])
            .comments(format!(
                "{}\n{}\n{}",
                fl!("app-info"),
                fl!("pokeapi-text"),
                fl!("nintendo-text")
            ));

        // Construct the app model with the runtime's core.
        let mut app = StarryDex {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            config_handler: flags.config_handler,
            config: flags.config,
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            starry_core: None,
            selected_pokemon: None,
            pokemon_list: Vec::new(),
            current_page_status: PageStatus::Loading,
            wants_pokemon_details: false,
            search: String::new(),
            filters: Filters {
                selected_types: HashSet::new(),
                total_stats: (false, 50),
            },
            type_filter_mode: vec![fl!("exclusive"), fl!("inclusive")],
            current_page: 0,
        };
        // Startup task that sets the window title.
        tasks.push(app.update_title());
        // Set correct theme on startup
        tasks.push(cosmic::command::set_theme(app.config.app_theme.theme()));

        // Create the directory where all of our application data will exist
        let app_data_dir = dirs::data_dir().unwrap().join(Self::APP_ID);
        std::fs::create_dir_all(&app_data_dir).expect("Failed to create the app data directory");

        // Init application core
        tasks.push(cosmic::app::Task::perform(
            async move { StarryCore::initialize().await },
            |starry_core| cosmic::action::app(Message::InitializedCore(starry_core)),
        ));

        if !app.config.first_run_completed {
            app.current_page_status = PageStatus::FirstRun;
            if let Some(handler) = &app.config_handler {
                if let Err(err) = app.config.set_first_run_completed(handler, true) {
                    eprintln!("{err}")
                }
            }
        } else {
            app.current_page_status = PageStatus::Loading;
        }

        (app, Task::batch(tasks))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            Element::from(menu::root(fl!("view"))),
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
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::about(
                &self.about,
                Message::LaunchUrl,
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
            ContextPage::Settings => context_drawer::context_drawer(
                self.settings(),
                Message::ToggleContextPage(ContextPage::Settings),
            )
            .title(fl!("settings")),
            ContextPage::PokemonPage => context_drawer::context_drawer(
                self.single_pokemon_page(),
                Message::ToggleContextPage(ContextPage::PokemonPage),
            )
            .title(fl!("pokemon-page")),
            ContextPage::FiltersPage => context_drawer::context_drawer(
                self.filters_page(),
                Message::ToggleContextPage(ContextPage::FiltersPage),
            )
            .title(fl!("filters-page")),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        let space_s = theme::active().cosmic().spacing.space_s;

        let content = match self.current_page_status {
            PageStatus::FirstRun => Column::new()
                .push(text(fl!("downloading-sprites")))
                .push(text(fl!("estimate")))
                .push(text(fl!("once-message")))
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .spacing(space_s)
                .into(),
            PageStatus::Loaded => self.landing(),
            PageStatus::Loading => Column::new()
                .push(text(fl!("loading")))
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .spacing(space_s)
                .into(),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            // Watch for application configuration changes.
            self.core()
                .watch_config::<StarryConfig>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ])
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::LaunchUrl(url) => {
                _ = open::that_detached(url);
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
            }
            Message::UpdateConfig(new_config) => {
                self.update_all_config_fields(&new_config);
                if let Some(core) = &self.starry_core {
                    self.current_page = 0;
                    self.pokemon_list = core.get_pokemon_page(0, self.config.items_per_page);
                }
                return cosmic::command::set_theme(self.config.app_theme.theme());
            }
            Message::UpdateTheme(index) => {
                let app_theme = match index {
                    1 => AppTheme::Dark,
                    2 => AppTheme::Light,
                    _ => AppTheme::System,
                };

                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_app_theme(handler, app_theme) {
                        eprintln!("{err}")
                    } else {
                        return cosmic::command::set_theme(self.config.app_theme.theme());
                    }
                }
            }
            Message::InitializedCore(core_res) => {
                match core_res {
                    Ok(core) => {
                        self.current_page = 0;
                        self.pokemon_list = core.get_pokemon_page(0, self.config.items_per_page);
                        self.starry_core = Some(core);
                        println!("Loaded StarryCore");
                    }
                    Err(err) => {
                        eprintln!("Failed to initialize StarryCore: {err}")
                    }
                }

                self.current_page_status = PageStatus::Loaded;
            }
            Message::LoadPokemon(pokemon_id) => {
                if let Some(core) = &self.starry_core {
                    let pokemon = core.get_pokemon_by_id(pokemon_id).map(|archived_pokemon| {
                        // TODO: Is unwrap safe here?
                        rkyv::deserialize::<StarryPokemon, rancor::Error>(archived_pokemon).unwrap()
                    });
                    self.selected_pokemon = pokemon;

                    // Open Context Page
                    self.context_page = ContextPage::PokemonPage;
                    self.core.window.show_context = true;
                }
            }
            Message::TogglePokemonDetails(value) => self.wants_pokemon_details = value,
            Message::SearchInput(value) => {
                self.search = value;
                if let Some(core) = &self.starry_core {
                    if self.search.is_empty() {
                        self.pokemon_list = core.get_pokemon_page(
                            self.current_page * self.config.items_per_page,
                            self.config.items_per_page,
                        );
                    } else {
                        self.pokemon_list = core.search_pokemon(&self.search);
                    }
                }
            }
            Message::TypeFilterToggled(value, type_name) => {
                if value {
                    // Add the selected type to the filter
                    self.filters.selected_types.insert(type_name);
                } else {
                    // Remove the deselected type from the filter
                    self.filters.selected_types.remove(&type_name);
                }
            }
            Message::ApplyCurrentFilters => {
                if let Some(core) = &self.starry_core {
                    if !self.filters.selected_types.is_empty() {
                        self.search = String::new();
                        self.current_page = 0;

                        let selected_types_lowercase: HashSet<String> = self
                            .filters
                            .selected_types
                            .iter()
                            .map(|t| t.name.to_lowercase())
                            .collect();

                        match self.config.type_filtering_mode {
                            TypeFilteringMode::Inclusive => {
                                // Ej: If fire and ice are selected it will show fire pokemons and ice pokemons
                                self.pokemon_list =
                                    core.filter_pokemon_inclusive(&selected_types_lowercase);
                            }
                            TypeFilteringMode::Exclusive => {
                                // Ej: If fire and ice are selected it will show pokemons that are both fire and ice types
                                self.pokemon_list =
                                    core.filter_pokemon_exclusive(&selected_types_lowercase);
                            }
                        }
                    }

                    // Apply total stats filter
                    if self.filters.total_stats.0 && self.filters.total_stats.1 > 0 {
                        if self.filters.selected_types.is_empty() {
                            self.pokemon_list =
                                core.filter_pokemon_stats(self.filters.total_stats.1);
                        } else {
                            self.pokemon_list = core.filter_pokemon_stats_with_list(
                                &self.pokemon_list,
                                self.filters.total_stats.1,
                            );
                        }
                    }

                    self.core.window.show_context = false;
                }
            }
            Message::ClearFilters => {
                if let Some(core) = &self.starry_core {
                    self.search = String::new();
                    self.current_page = 0;
                    self.pokemon_list = core.get_pokemon_page(0, self.config.items_per_page);
                    self.filters = Filters {
                        selected_types: HashSet::new(),
                        total_stats: (false, 50),
                    };
                    self.current_page_status = PageStatus::Loaded;
                }
            }
            Message::UpdateTypeFilterMode(index) => {
                let filter_mode = match index {
                    1 => TypeFilteringMode::Inclusive,
                    _ => TypeFilteringMode::Exclusive,
                };

                if let Some(handler) = &self.config_handler {
                    if let Err(err) = self.config.set_type_filtering_mode(handler, filter_mode) {
                        eprintln!("{err}")
                    }
                }
            }
            Message::DeleteCache => {
                if self.current_page_status == PageStatus::Loaded {
                    self.current_page_status = PageStatus::FirstRun;
                    self.set_show_context(false);

                    let data_dir = dirs::data_dir().unwrap().join(Self::APP_ID);
                    if let Err(e) = remove_dir_contents(&data_dir) {
                        eprintln!("Error deleting cache: {e}");
                    }

                    return cosmic::app::Task::perform(
                        async move { StarryCore::initialize().await },
                        |starry_core| cosmic::action::app(Message::InitializedCore(starry_core)),
                    );
                }
            }
            Message::PaginationActionRequested(action) => match &action {
                PaginationAction::Next => {
                    if let Some(core) = &self.starry_core {
                        if self.search.is_empty()
                            && self.filters.selected_types.is_empty()
                            && !self.filters.total_stats.0
                        {
                            let new_list = core.get_pokemon_page(
                                (self.current_page + 1) * self.config.items_per_page,
                                self.config.items_per_page,
                            );
                            if !new_list.is_empty() {
                                self.current_page += 1;
                                self.pokemon_list = new_list;
                            }
                        }
                    }
                }
                PaginationAction::Back => {
                    if self.current_page >= 1 {
                        if let Some(core) = &self.starry_core {
                            if self.search.is_empty()
                                && self.filters.selected_types.is_empty()
                                && !self.filters.total_stats.0
                            {
                                let new_list = core.get_pokemon_page(
                                    (self.current_page - 1) * self.config.items_per_page,
                                    self.config.items_per_page,
                                );
                                if !new_list.is_empty() {
                                    self.current_page -= 1;
                                    self.pokemon_list = new_list;
                                }
                            }
                        }
                    }
                }
            },
            Message::SinglePokemonPagination(action) => match &action {
                PaginationAction::Next => {
                    if let Some(core) = &self.starry_core {
                        if !self.pokemon_list.is_empty() {
                            if let Some(current_pokemon) = &self.selected_pokemon {
                                let current_id = current_pokemon.pokemon.id;
                                if let Some(current_index) =
                                    self.pokemon_list.iter().position(|p| p.id == current_id)
                                {
                                    let next_index = (current_index + 1) % self.pokemon_list.len();
                                    let next_pokemon_id = self.pokemon_list[next_index].id;
                                    let pokemon = core.get_pokemon_by_id(next_pokemon_id).map(
                                        |archived_pokemon| {
                                            // TODO: Is unwrap safe here?
                                            rkyv::deserialize::<StarryPokemon, rancor::Error>(
                                                archived_pokemon,
                                            )
                                            .unwrap()
                                        },
                                    );

                                    self.selected_pokemon = pokemon;
                                }
                            }
                        }
                    }
                }
                PaginationAction::Back => {
                    if let Some(core) = &self.starry_core {
                        if !self.pokemon_list.is_empty() {
                            if let Some(current_pokemon) = &self.selected_pokemon {
                                let current_id = current_pokemon.pokemon.id;
                                if let Some(current_index) =
                                    self.pokemon_list.iter().position(|p| p.id == current_id)
                                {
                                    let prev_index = if current_index == 0 {
                                        self.pokemon_list.len() - 1
                                    } else {
                                        current_index - 1
                                    };
                                    let prev_pokemon_id = self.pokemon_list[prev_index].id;
                                    let pokemon = core.get_pokemon_by_id(prev_pokemon_id).map(
                                        |archived_pokemon| {
                                            // TODO: Is unwrap safe here?
                                            rkyv::deserialize::<StarryPokemon, rancor::Error>(
                                                archived_pokemon,
                                            )
                                            .unwrap()
                                        },
                                    );

                                    self.selected_pokemon = pokemon;
                                }
                            }
                        }
                    }
                }
            },
            Message::StatsFilterToggled(new_value) => {
                self.filters.total_stats.0 = new_value;
            }
            Message::StatsFilterUpdated(new_value) => {
                if self.filters.total_stats.0 {
                    self.filters.total_stats.1 = new_value;
                }
            }
        }
        Task::none()
    }
}

impl StarryDex {
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

        let old_config = self.config.clone();

        widget::settings::view_column(vec![
            widget::settings::section()
                .title(fl!("appearance"))
                .add(
                    widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                        &self.app_themes,
                        Some(app_theme_selected),
                        Message::UpdateTheme,
                    )),
                )
                .add(
                    widget::settings::item::builder(fl!("pokemon-per-row"))
                        .description(format!("{}", self.config.pokemon_per_row))
                        .control(
                            widget::slider(
                                1..=10,
                                self.config.pokemon_per_row as u16,
                                move |new_value| {
                                    Message::UpdateConfig(StarryConfig {
                                        app_theme: old_config.app_theme,
                                        first_run_completed: old_config.first_run_completed,
                                        pokemon_per_row: new_value as usize,
                                        items_per_page: old_config.items_per_page,
                                        type_filtering_mode: old_config.type_filtering_mode,
                                    })
                                },
                            )
                            .step(1u16),
                        ),
                )
                .add(
                    widget::settings::item::builder(fl!("pokemon-per-page"))
                        .description(format!("{}", self.config.items_per_page))
                        .control(
                            widget::slider(
                                10..=1500,
                                self.config.items_per_page as u16,
                                move |new_value| {
                                    Message::UpdateConfig(StarryConfig {
                                        app_theme: old_config.app_theme,
                                        first_run_completed: old_config.first_run_completed,
                                        pokemon_per_row: old_config.pokemon_per_row,
                                        items_per_page: new_value as usize,
                                        type_filtering_mode: old_config.type_filtering_mode,
                                    })
                                },
                            )
                            .step(10u16),
                        ),
                )
                .into(),
            widget::settings::section()
                .title(fl!("other"))
                .add(
                    widget::settings::item::builder(fl!("type-filter-mode")).control(
                        widget::dropdown(
                            &self.type_filter_mode,
                            Some(type_filter_mode_selected),
                            Message::UpdateTypeFilterMode,
                        ),
                    ),
                )
                .add(
                    widget::settings::item::builder(fl!("renew-cache")).control(
                        widget::button::destructive(fl!("renew-cache-button"))
                            .on_press(Message::DeleteCache),
                    ),
                )
                .into(),
        ])
        .into()
    }

    /// The main page for this app.
    pub fn landing(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;
        let mut pokemon_grid = widget::Grid::new().width(Length::Fill);

        for (index, pokemon) in self.pokemon_list.iter().enumerate() {
            let pokemon_image = if let Some(path) = &pokemon.sprite_path.as_ref() {
                widget::Image::new(path.as_str())
                    .content_fit(cosmic::iced::ContentFit::None)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
            } else {
                widget::Image::new(ImageCache::get("fallback"))
                    .content_fit(cosmic::iced::ContentFit::None)
                    .width(Length::Fixed(100.0))
                    .height(Length::Fixed(100.0))
            };

            let pokemon_container = widget::button::custom(
                widget::Column::new()
                    .push(pokemon_image.width(Length::Shrink))
                    .push(
                        text(capitalize_string(&pokemon.name))
                            .width(Length::Shrink)
                            .line_height(LineHeight::Absolute(Pixels::from(15.0))),
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(135.0))
            .on_press_down(Message::LoadPokemon(pokemon.id))
            .class(theme::Button::Image)
            .padding([spacing.space_none, spacing.space_s]);

            // Insert a new row before adding the first Pokémon of each row
            if index % self.config.pokemon_per_row == 0 {
                pokemon_grid = pokemon_grid.insert_row();
            }

            pokemon_grid = pokemon_grid.push(pokemon_container);
        }

        let search = widget::search_input(fl!("search"), &self.search)
            .style(theme::TextInput::Search)
            .on_input(Message::SearchInput)
            .line_height(LineHeight::Absolute(Pixels(30.0)))
            .width(Length::Fill);

        let filters = widget::button::icon(icon_cache::get_handle("filter-symbolic", 18))
            .class(theme::Button::Suggested)
            .on_press(Message::ToggleContextPage(ContextPage::FiltersPage))
            .width(Length::Shrink);

        let clear_filters =
            widget::button::icon(icon_cache::get_handle("edit-clear-all-symbolic", 18))
                .class(theme::Button::Destructive)
                .on_press(Message::ClearFilters)
                .width(Length::Shrink);

        let search_row = widget::Row::new()
            .push(search)
            .push(filters)
            .push(clear_filters)
            .spacing(Pixels::from(spacing.space_xxxs))
            .width(Length::Fill);

        let pagination_row = widget::container(
            widget::Row::new()
                .push(
                    widget::button::icon(icon_cache::get_handle("go-previous-symbolic", 18))
                        .on_press(Message::PaginationActionRequested(PaginationAction::Back)),
                )
                .push(text(format!(
                    "{} - {}",
                    fl!("page"),
                    (&self.current_page + 1)
                )))
                .push(
                    widget::button::icon(icon_cache::get_handle("go-next-symbolic", 18))
                        .on_press(Message::PaginationActionRequested(PaginationAction::Next)),
                )
                .spacing(Pixels::from(spacing.space_xxl))
                .width(Length::Shrink)
                .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::Center);

        widget::Column::new()
            .push(search_row)
            .push(
                widget::scrollable(widget::container(pokemon_grid).align_x(Horizontal::Center))
                    .height(Length::FillPortion(8))
                    .width(Length::Fill),
            )
            .push(pagination_row)
            .width(Length::Fill)
            .padding(5.)
            .spacing(spacing.space_xxs)
            .into()
    }

    /// The pokemon details context page for this app.
    pub fn single_pokemon_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let content: widget::Column<_> = match &self.selected_pokemon {
            Some(starry_pokemon) => {
                let page_title = widget::container(
                    widget::Row::new()
                        .push(
                            widget::button::icon(icon_cache::get_handle(
                                "go-previous-symbolic",
                                18,
                            ))
                            .on_press(Message::SinglePokemonPagination(PaginationAction::Back)),
                        )
                        .push(widget::text::title1(capitalize_string(
                            starry_pokemon.pokemon.name.as_str(),
                        )))
                        .push(
                            widget::button::icon(icon_cache::get_handle("go-next-symbolic", 18))
                                .on_press(Message::SinglePokemonPagination(PaginationAction::Next)),
                        )
                        .spacing(spacing.space_s)
                        .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .align_y(Alignment::Center)
                .align_x(Alignment::Center);

                let pokemon_image = if let Some(path) = &starry_pokemon.sprite_path {
                    widget::Image::new(path).content_fit(cosmic::iced::ContentFit::Fill)
                } else {
                    widget::Image::new(ImageCache::get("fallback"))
                        .content_fit(cosmic::iced::ContentFit::Fill)
                };

                let pokemon_weight = widget::container(
                    widget::Column::new()
                        .push(widget::text::title3(fl!("weight")))
                        .push(
                            text(format!(
                                "{} Kg",
                                scale_numbers(starry_pokemon.pokemon.weight)
                            ))
                            .size(15.0),
                        )
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .class(theme::Container::Card)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_height = widget::container(
                    widget::Column::new()
                        .push(widget::text::title3(fl!("height")))
                        .push(
                            text(format!(
                                "{} m",
                                scale_numbers(starry_pokemon.pokemon.height)
                            ))
                            .size(15.0),
                        )
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .class(theme::Container::Card)
                .padding([spacing.space_none, spacing.space_xxs]);

                let mut pokemon_types_row = Row::new().spacing(spacing.space_s);
                for poke_type in &starry_pokemon.pokemon.types {
                    pokemon_types_row = pokemon_types_row.push(widget::tooltip(
                        widget::icon(icon_cache::get_handle_owned(
                            format!("type-{poke_type}"),
                            18,
                        )),
                        text(capitalize_string(poke_type)),
                        widget::tooltip::Position::Bottom,
                    ));
                }
                let pokemon_types = container(pokemon_types_row).align_x(Alignment::Center);

                let mut pokemon_abilities_column = widget::Column::new().push(
                    widget::text::title3(fl!("pokemon-abilities"))
                        .width(Length::Fill)
                        .align_x(Alignment::Center),
                );
                for ability in &starry_pokemon.pokemon.abilities {
                    pokemon_abilities_column = pokemon_abilities_column.push(
                        text(capitalize_string(ability))
                            .width(Length::Fill)
                            .align_x(Alignment::Center),
                    );
                }
                let pokemon_abilities = widget::container(pokemon_abilities_column)
                    .width(Length::Fill)
                    .class(theme::Container::Card)
                    .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_stats = widget::container(column![
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
                .class(theme::Container::Card);

                let pokemon_first_row = widget::Row::new()
                    .push(pokemon_weight)
                    .push(pokemon_height)
                    .spacing(8.0)
                    .align_y(Alignment::Center);

                let mut result_col = widget::Column::new()
                    .push(page_title)
                    .push(pokemon_image)
                    .push(pokemon_types)
                    .push(pokemon_first_row)
                    .push(pokemon_abilities)
                    .push(pokemon_stats)
                    .align_x(Alignment::Center)
                    .spacing(10.0);

                let show_details = widget::Checkbox::new(
                    fl!("show-encounter-details"),
                    self.wants_pokemon_details,
                )
                .on_toggle(Message::TogglePokemonDetails);

                let encounter_info = match &starry_pokemon.encounter_info {
                    Some(info) => {
                        let children = info.iter().map(|ef| {
                            let mut version_column = widget::Column::new().width(Length::Fill);
                            version_column = version_column.push(
                                text(capitalize_string(&ef.city))
                                    .class(theme::Text::Accent)
                                    .size(Pixels::from(15)),
                            );

                            for method in &ef.games_method {
                                version_column = version_column.push(text(method));
                            }

                            version_column.into()
                        });

                        widget::container(Column::with_children(children))
                            .class(theme::Container::Card)
                            .padding([spacing.space_none, spacing.space_xxs])
                    }
                    None => widget::container(text(fl!("no-encounter-info")))
                        .class(theme::Container::Card),
                };

                let link = widget::button::link(format!("{} (Bulbapedia)", fl!("link-more-info")))
                    .on_press(Message::LaunchUrl(format!(
                        "https://bulbapedia.bulbagarden.net/w/index.php?search={}",
                        &starry_pokemon.pokemon.name
                    )));

                if starry_pokemon.encounter_info.is_some()
                    && !starry_pokemon.encounter_info.clone().unwrap().is_empty()
                {
                    result_col = result_col.push(show_details);
                    if self.wants_pokemon_details {
                        result_col = result_col.push(encounter_info);
                    }
                }

                result_col = result_col.push(link);
                return result_col.into();
            }
            None => {
                let error = cosmic::Apply::apply(
                    widget::text::title1(fl!("generic-error")),
                    widget::container,
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center);

                widget::Column::new().push(error)
            }
        };

        widget::container(content).into()
    }

    /// The filters context page for this app.
    pub fn filters_page(&self) -> Element<Message> {
        let all_pokemon_types = PokemonType::get_all();

        let type_checkboxes: Vec<Element<Message>> = all_pokemon_types
            .into_iter()
            .map(|pokemon_type| {
                let is_checked = self.filters.selected_types.contains(&pokemon_type);
                let checkbox: Element<Message> =
                    widget::checkbox::Checkbox::new(pokemon_type.display_name.clone(), is_checked)
                        .on_toggle(move |value| {
                            Message::TypeFilterToggled(value, pokemon_type.clone())
                        })
                        .into();

                widget::container(checkbox).width(Length::Fill).into()
            })
            .collect();

        let mut types_column = widget::Column::new()
            .push(widget::text::title3(fl!("type-filters")))
            .spacing(5)
            .width(Length::Fill);
        let mut current_row = widget::Row::new();
        let mut count = 0;

        for t_checkbox in type_checkboxes {
            current_row = current_row.push(t_checkbox);
            count += 1;

            if count % 2 == 0 {
                types_column = types_column.push(current_row);
                current_row = widget::Row::new();
            }
        }

        // If there's an odd number of checkboxes, add the last row
        if count % 2 != 0 {
            types_column = types_column.push(current_row);
        }

        let poke_stats_column = column![
            widget::text::title3(fl!("stats-filter")),
            widget::Row::new()
                .push(
                    widget::Checkbox::new(fl!("enabled"), self.filters.total_stats.0)
                        .on_toggle(Message::StatsFilterToggled)
                        .width(Length::Fill),
                )
                .push(
                    column![
                        text(format!(
                            "{}: {}",
                            fl!("minimum-poke-stats"),
                            &self.filters.total_stats.1
                        )),
                        widget::slider(
                            0.0..=800.0,
                            self.filters.total_stats.1 as f64,
                            move |new_value| Message::StatsFilterUpdated(new_value as i64),
                        )
                        .step(10.0)
                    ]
                    .spacing(2.),
                )
                .align_y(Alignment::Center)
                .width(Length::Fill)
        ];

        let result_column = widget::Column::new()
            .width(Length::Fill)
            .push(types_column)
            .push(poke_stats_column)
            .push(
                widget::container(
                    widget::button::suggested(fl!("apply-filters"))
                        .on_press(Message::ApplyCurrentFilters)
                        .width(Length::Shrink),
                )
                .width(Length::Fill)
                .align_x(Horizontal::Center),
            )
            .spacing(15.);

        widget::container(result_column).into()
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let window_title = fl!("app-title");

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }

    fn update_all_config_fields(&mut self, config: &StarryConfig) {
        if let Some(handler) = &self.config_handler {
            if let Err(err) = self.config.set_app_theme(handler, config.app_theme) {
                eprintln!("{err}")
            }
            if let Err(err) = self
                .config
                .set_first_run_completed(handler, config.first_run_completed)
            {
                eprintln!("{err}")
            }
            if let Err(err) = self
                .config
                .set_items_per_page(handler, config.items_per_page)
            {
                eprintln!("{err}")
            }
            if let Err(err) = self
                .config
                .set_pokemon_per_row(handler, config.pokemon_per_row)
            {
                eprintln!("{err}")
            }
            if let Err(err) = self
                .config
                .set_type_filtering_mode(handler, config.type_filtering_mode)
            {
                eprintln!("{err}")
            }
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
    PokemonPage,
    FiltersPage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
        }
    }
}
