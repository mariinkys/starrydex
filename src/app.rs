// SPDX-License-Identifier: GPL-3.0-only

use crate::api::Api;
use crate::config::{AppTheme, Config, TypeFilteringMode};
use crate::fl;
use crate::image_cache::ImageCache;
use crate::utils::{capitalize_string, remove_dir_contents, scale_numbers};
use cosmic::app::{context_drawer, Core, Task};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Pixels, Subscription};
use cosmic::iced_core::text::LineHeight;
use cosmic::widget::about::About;
use cosmic::widget::{self, menu, Column};
use cosmic::{theme, Application, ApplicationExt, Element};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;

const REPOSITORY: &str = "https://github.com/mariinkys/starrydex";
//const APP_ICON: &[u8] = include_bytes!("../res/icons/hicolor/256x256/apps/dev.mariinkys.StarryDex.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct StarryDex {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Application about page
    about: About,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config: Config,
    // Application Themes
    app_themes: Vec<String>,
    // API Client
    api: Api,
    // Status of the main application page
    current_page_status: PageStatus,
    // Holds the list of Pokémon
    pokemon_list: BTreeMap<i64, StarryPokemon>,
    // Holds the shown list of Pokémon
    filtered_pokemon_list: Vec<StarryPokemon>,
    // Holds the data of the currently selected Pokémon to show it on the context page
    selected_pokemon: Option<StarryPokemon>,
    // Controls the Pokémon Details Toggle of the Pokémon Context Page
    wants_pokemon_details: bool,
    // Holds the search input value
    search: String,
    // Holds the currently applied filters if there are any
    filters: Filters,
    // Type Filter Modes
    type_filter_mode: Vec<String>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    UpdateTheme(usize),
    UpdateTypeFilterMode(usize),

    LoadPokemon(i64),
    TogglePokemonDetails(bool),
    Search(String),
    ApplyCurrentFilters,
    ClearFilters,
    DeleteCache,

    CompletedFirstRun(Config, BTreeMap<i64, StarryPokemon>),
    LoadedPokemonList(BTreeMap<i64, StarryPokemon>),
    TypeFilterToggled(bool, String),
}

/// Represents a Pokémon in the application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarryPokemon {
    pub pokemon: StarryPokemonData,
    pub sprite_path: Option<String>,
    pub encounter_info: Option<Vec<StarryPokemonEncounterInfo>>,
}

/// Data of a Pokémon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarryPokemonData {
    pub id: i64,
    pub name: String,
    pub weight: i64,
    pub height: i64,
    pub types: Vec<String>,
    pub abilities: Vec<String>,
    pub stats: StarryPokemonStats,
}

/// Represents a Pokémon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarryPokemonStats {
    pub hp: i64,
    pub attack: i64,
    pub defense: i64,
    pub sp_attack: i64,
    pub sp_defense: i64,
    pub speed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarryPokemonEncounterInfo {
    pub city: String,
    pub games_method: Vec<String>,
}

pub struct Filters {
    pub selected_types: HashSet<String>,
}

/// Identifies the status of a page in the application.
pub enum PageStatus {
    FirstRun,
    Loaded,
    Loading,
}

/// Create a COSMIC application from the app model
impl Application for StarryDex {
    /// The async executor that will be used to run your application's tasks.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "dev.mariinkys.StarryDex";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup tasks.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Tasks that will get executed on the application init
        let mut tasks = vec![];

        // Controls if it's the first time the application runs on a system
        let mut first_run_completed = false;

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
            .developers([("mariinkys", "kysdev.owjga@aleeas.com")]);

        // Construct the app model with the runtime's core.
        let mut app = StarryDex {
            core,
            about,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => {
                        first_run_completed = config.first_run_completed;
                        config
                    }
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            app_themes: vec![fl!("match-desktop"), fl!("dark"), fl!("light")],
            api: Api::new(Self::APP_ID),
            current_page_status: PageStatus::Loading,
            pokemon_list: BTreeMap::new(),
            filtered_pokemon_list: Vec::new(),
            selected_pokemon: None,
            wants_pokemon_details: false,
            search: String::new(),
            filters: Filters {
                selected_types: HashSet::new(),
            },
            type_filter_mode: vec![fl!("exclusive"), fl!("inclusive")],
        };
        // Startup task that sets the window title.
        tasks.push(app.update_title());

        // Create the directory where all of our application data will exist
        let app_data_dir = dirs::data_dir().unwrap().join(Self::APP_ID);
        std::fs::create_dir_all(&app_data_dir).expect("Failed to create the app data directory");

        // Clone the app api in order to use it.
        let api_clone = app.api.clone();

        if !first_run_completed {
            // First application run, construct cache, download sprites and update the config
            app.current_page_status = PageStatus::FirstRun;
            tasks.push(cosmic::app::Task::perform(
                async move { api_clone.load_all_pokemon().await },
                |pokemon_list| {
                    cosmic::app::message::app(Message::CompletedFirstRun(
                        Config {
                            app_theme: crate::config::AppTheme::System,
                            first_run_completed: true,
                            pokemon_per_row: 3,
                            type_filtering_mode: crate::config::TypeFilteringMode::Exclusive,
                        },
                        pokemon_list,
                    ))
                },
            ));
        } else {
            // Load  the Pokémon List
            app.current_page_status = PageStatus::Loading;
            tasks.push(cosmic::app::Task::perform(
                async move { api_clone.load_all_pokemon().await },
                |pokemon_list| cosmic::app::message::app(Message::LoadedPokemonList(pokemon_list)),
            ));
        }

        (app, Task::batch(tasks))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
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
                //.push(widget::text::text(fl!("downloading-sprites")))
                //.push(widget::text::text(fl!("estimate")))
                //.push(widget::text::text(fl!("once-message")))
                // TODO: This is temporal because settings do not get saved and are lost upon app restart.
                .push(widget::text::text("Loading..."))
                .push(widget::text::text("First load may take a minute"))
                .push(widget::text::text("It will go faster after the first load"))
                .align_x(Alignment::Center)
                .width(Length::Fill)
                .spacing(space_s)
                .into(),
            PageStatus::Loaded => self.landing(),
            PageStatus::Loading => Column::new()
                .push(widget::text::text(fl!("loading")))
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
                .watch_config::<Config>(Self::APP_ID)
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
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
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
            Message::UpdateConfig(config) => {
                self.config = config;
                return cosmic::app::command::set_theme(self.config.app_theme.theme());
            }
            Message::UpdateTheme(index) => {
                let old_config = self.config.clone();

                let app_theme = match index {
                    1 => AppTheme::Dark,
                    2 => AppTheme::Light,
                    _ => AppTheme::System,
                };
                self.config = Config {
                    first_run_completed: old_config.first_run_completed,
                    pokemon_per_row: old_config.pokemon_per_row,
                    type_filtering_mode: old_config.type_filtering_mode,
                    app_theme,
                };
                return cosmic::app::command::set_theme(self.config.app_theme.theme());
            }
            Message::CompletedFirstRun(config, pokemon_list) => {
                self.config = config;

                self.pokemon_list = pokemon_list;
                //TODO: Remove this. This is to temporarly fixed an error that makes a empty pokemon to appear on the first position of the btree
                //let mut pokemon_list = pokemon_list;
                //pokemon_list.pop_first();
                //self.pokemon_list = pokemon_list;

                self.filtered_pokemon_list = self.pokemon_list.values().cloned().collect();
                self.current_page_status = PageStatus::Loaded;

                return cosmic::app::command::set_theme(self.config.app_theme.theme());
            }
            Message::LoadedPokemonList(pokemon_list) => {
                //self.pokemon_list = pokemon_list; //TODO: This is to temporarly fix an error that makes a empty pokemon to appear on the first position of the btree
                let mut pokemon_list = pokemon_list;
                pokemon_list.pop_first();
                self.pokemon_list = pokemon_list;

                self.filtered_pokemon_list = self.pokemon_list.values().cloned().collect();
                self.current_page_status = PageStatus::Loaded;
            }
            Message::LoadPokemon(pokemon_id) => {
                self.selected_pokemon = self.pokemon_list.get(&pokemon_id).cloned();

                // Open Context Page
                self.context_page = ContextPage::PokemonPage;
                self.core.window.show_context = true;
            }
            Message::TogglePokemonDetails(value) => self.wants_pokemon_details = value,
            Message::Search(value) => {
                // TODO: Improve search speed? Search by id...Search shouldn't erase filters
                self.search = value;
                self.filtered_pokemon_list = self
                    .pokemon_list
                    .iter()
                    .filter(|(&_id, pokemon)| {
                        pokemon
                            .pokemon
                            .name
                            .to_lowercase()
                            .contains(&self.search.to_lowercase())
                    })
                    .map(|(_, pokemon)| pokemon.clone())
                    .collect();
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
                //TODO: Revisit how to do this without this being necessary, search does not need to be lost?
                self.search = String::new();

                match self.config.type_filtering_mode {
                    TypeFilteringMode::Inclusive => {
                        // Ej: If fire and ice are selected it will show fire pokemons and ice pokemons
                        let selected_types_lowercase: HashSet<String> = self
                            .filters
                            .selected_types
                            .iter()
                            .map(|t| t.to_lowercase())
                            .collect();

                        self.filtered_pokemon_list = self
                            .pokemon_list
                            .values()
                            .filter(|pokemon| {
                                selected_types_lowercase.is_empty()
                                    || pokemon.pokemon.types.iter().any(|t| {
                                        selected_types_lowercase.contains(&t.to_lowercase())
                                    })
                            })
                            .cloned()
                            .collect();
                    }
                    TypeFilteringMode::Exclusive => {
                        // Ej: If fire and ice are selected it will show pokemons that are both fire and ice types
                        let selected_types_lowercase: HashSet<String> = self
                            .filters
                            .selected_types
                            .iter()
                            .map(|t| t.to_lowercase())
                            .collect();

                        self.filtered_pokemon_list = self
                            .pokemon_list
                            .values()
                            .filter(|pokemon| {
                                selected_types_lowercase.is_empty()
                                    || selected_types_lowercase.iter().all(|selected_type| {
                                        pokemon
                                            .pokemon
                                            .types
                                            .iter()
                                            .any(|t| t.to_lowercase() == *selected_type)
                                    })
                            })
                            .cloned()
                            .collect();
                    }
                }

                self.core.window.show_context = false;
            }
            Message::ClearFilters => {
                self.filtered_pokemon_list = self.pokemon_list.values().cloned().collect();
                self.filters = Filters {
                    selected_types: HashSet::new(),
                };
                self.current_page_status = PageStatus::Loaded;
            }
            Message::UpdateTypeFilterMode(index) => {
                let old_config = self.config.clone();

                let filter_mode = match index {
                    1 => TypeFilteringMode::Inclusive,
                    _ => TypeFilteringMode::Exclusive,
                };
                self.config = Config {
                    first_run_completed: old_config.first_run_completed,
                    pokemon_per_row: old_config.pokemon_per_row,
                    type_filtering_mode: filter_mode,
                    app_theme: old_config.app_theme,
                };
            }
            Message::DeleteCache => {
                self.current_page_status = PageStatus::FirstRun;
                self.set_show_context(false);

                let data_dir = dirs::data_dir().unwrap().join(Self::APP_ID);
                if let Err(e) = remove_dir_contents(&data_dir) {
                    eprintln!("Error deleting cache: {}", e);
                }

                // Reset the API
                self.api = Api::new(Self::APP_ID);
                let api_clone = self.api.clone();
                return cosmic::app::Task::perform(
                    async move { api_clone.load_all_pokemon().await },
                    |pokemon_list| {
                        cosmic::app::message::app(Message::LoadedPokemonList(pokemon_list))
                    },
                );
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

        let current_value = self.config.pokemon_per_row as u16;
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
                        .description(format!("{}", current_value))
                        .control(
                            widget::slider(1..=10, current_value, move |new_value| {
                                Message::UpdateConfig(Config {
                                    app_theme: old_config.app_theme,
                                    first_run_completed: old_config.first_run_completed,
                                    pokemon_per_row: new_value as usize,
                                    type_filtering_mode: old_config.type_filtering_mode,
                                })
                            })
                            .step(1u16),
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

        for (index, pokemon) in self.filtered_pokemon_list.iter().enumerate() {
            let pokemon_image = if let Some(path) = &pokemon.sprite_path {
                widget::Image::new(path)
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
                        widget::text::text(capitalize_string(&pokemon.pokemon.name))
                            .width(Length::Shrink)
                            .line_height(LineHeight::Absolute(Pixels::from(15.0))),
                    )
                    .width(Length::Fill)
                    .align_x(Alignment::Center),
            )
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(135.0))
            .on_press_down(Message::LoadPokemon(pokemon.pokemon.id))
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
            .on_input(Message::Search)
            .line_height(LineHeight::Absolute(Pixels(30.0)))
            .width(Length::Fill);

        let filters = widget::button::standard(fl!("filter"))
            .class(theme::Button::Suggested)
            .on_press(Message::ToggleContextPage(ContextPage::FiltersPage))
            .width(Length::Shrink);

        let clear_filters = widget::button::standard(fl!("clear-filters"))
            .class(theme::Button::Destructive)
            .on_press(Message::ClearFilters)
            .width(Length::Shrink);

        let search_row = widget::Row::new()
            .push(search)
            .push(filters)
            .push(clear_filters)
            .spacing(Pixels::from(spacing.space_xxxs))
            .width(Length::Fill);

        widget::Column::new()
            .push(search_row)
            .push(
                widget::scrollable(
                    widget::Container::new(pokemon_grid).align_x(Horizontal::Center),
                )
                .width(Length::Fill),
            )
            .width(Length::Fill)
            .spacing(spacing.space_s)
            .into()
    }

    /// The pokemon details context page for this app.
    pub fn single_pokemon_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let content: widget::Column<_> = match &self.selected_pokemon {
            Some(starry_pokemon) => {
                let page_title =
                    widget::text::title1(capitalize_string(starry_pokemon.pokemon.name.as_str()))
                        .width(Length::Fill)
                        .align_x(Horizontal::Center);

                let pokemon_image = if let Some(path) = &starry_pokemon.sprite_path {
                    widget::Image::new(path).content_fit(cosmic::iced::ContentFit::Fill)
                } else {
                    widget::Image::new(ImageCache::get("fallback"))
                        .content_fit(cosmic::iced::ContentFit::Fill)
                };

                let pokemon_weight = widget::container::Container::new(
                    widget::Column::new()
                        .push(widget::text::title3(fl!("weight")))
                        .push(
                            widget::text::text(format!(
                                "{} Kg",
                                scale_numbers(starry_pokemon.pokemon.weight)
                            ))
                            .size(15.0),
                        )
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .class(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_height = widget::container::Container::new(
                    widget::Column::new()
                        .push(widget::text::title3(fl!("height")))
                        .push(
                            widget::text::text(format!(
                                "{} m",
                                scale_numbers(starry_pokemon.pokemon.height)
                            ))
                            .size(15.0),
                        )
                        .align_x(Alignment::Center)
                        .width(Length::Fill),
                )
                .class(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_types = widget::container::Container::new(Column::with_children(
                    starry_pokemon.pokemon.types.iter().map(|poke_type| {
                        widget::Row::new()
                            .push(
                                widget::text(poke_type.to_uppercase())
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .into()
                    }),
                ))
                .class(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_abilities = widget::container::Container::new(Column::with_children(
                    starry_pokemon.pokemon.abilities.iter().map(|poke_ability| {
                        widget::Row::new()
                            .push(
                                widget::text(poke_ability.to_uppercase())
                                    .width(Length::Fill)
                                    .align_x(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .into()
                    }),
                ))
                .class(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_stats = widget::container::Container::new(
                    Column::new()
                        .push(
                            widget::Row::new()
                                .push(widget::text(fl!("hp")).width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.hp.to_string())
                                        .align_x(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text(fl!("attack")).width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.attack.to_string())
                                        .align_x(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text(fl!("defense")).width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.defense.to_string())
                                        .align_x(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text(fl!("sp-a")).width(Length::Fill))
                                .push(
                                    widget::text(
                                        starry_pokemon.pokemon.stats.sp_attack.to_string(),
                                    )
                                    .align_x(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text(fl!("sp-d")).width(Length::Fill))
                                .push(
                                    widget::text(
                                        starry_pokemon.pokemon.stats.sp_defense.to_string(),
                                    )
                                    .align_x(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text(fl!("spd")).width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.speed.to_string())
                                        .align_x(Horizontal::Left),
                                ),
                        ),
                )
                .class(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_first_row = widget::Row::new()
                    .push(pokemon_weight)
                    .push(pokemon_height)
                    .push(pokemon_types)
                    .spacing(8.0)
                    .align_y(Alignment::Center);

                let mut result_col = widget::Column::new()
                    .push(page_title)
                    .push(pokemon_image)
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
                                widget::text(capitalize_string(&ef.city))
                                    .class(theme::Text::Accent)
                                    .size(Pixels::from(15)),
                            );

                            for method in &ef.games_method {
                                version_column = version_column.push(widget::text(method));
                            }

                            version_column.into()
                        });

                        widget::container::Container::new(Column::with_children(children))
                            .class(theme::Container::ContextDrawer)
                            .padding([spacing.space_none, spacing.space_xxs])
                    }
                    None => widget::Container::new(widget::Text::new(fl!("no-encounter-info")))
                        .class(theme::Container::ContextDrawer),
                };

                let link = widget::button::link(fl!("link-more-info"))
                    .on_press(Message::LaunchUrl(format!(
                        "https://bulbapedia.bulbagarden.net/w/index.php?search={}",
                        &starry_pokemon.pokemon.name
                    )))
                    .padding(0);

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
        // TODO: Pokémon Types can't be transated because they need to match so the filtering works.
        //let all_pokemon_types = vec![
        //    fl!("normal"),
        //    fl!("fire"),
        //    fl!("water"),
        //    fl!("electric"),
        //    fl!("grass"),
        //    fl!("ice"),
        //    fl!("fighting"),
        //    fl!("poison"),
        //    fl!("ground"),
        //    fl!("flying"),
        //    fl!("psychic"),
        //    fl!("bug"),
        //    fl!("rock"),
        //    fl!("ghost"),
        //    fl!("dragon"),
        //    fl!("dark"),
        //    fl!("steel"),
        //    fl!("fairy"),
        //];
        let all_pokemon_types = vec![
            "Normal", "Fire", "Water", "Electric", "Grass", "Ice", "Fighting", "Poison", "Ground",
            "Flying", "Psychic", "Bug", "Rock", "Ghost", "Dragon", "Dark", "Steel", "Fairy",
        ];

        let type_checkboxes: Vec<Element<Message>> = all_pokemon_types
            .into_iter()
            .map(|pokemon_type| {
                let is_checked = self.filters.selected_types.contains(pokemon_type);
                let checkbox: Element<Message> =
                    widget::checkbox::Checkbox::new(pokemon_type, is_checked)
                        .on_toggle(move |value| {
                            Message::TypeFilterToggled(value, pokemon_type.to_string())
                        })
                        .into();

                widget::Container::new(checkbox).width(Length::Fill).into()
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

        let result_column = widget::Column::new()
            .width(Length::Fill)
            .push(types_column)
            .push(
                widget::Container::new(
                    widget::button::suggested(fl!("apply-filters"))
                        .on_press(Message::ApplyCurrentFilters)
                        .width(Length::Shrink),
                )
                .width(Length::Fill)
                .align_x(Horizontal::Center),
            )
            .spacing(Pixels::from(30.0));

        widget::Container::new(result_column).into()
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<Message> {
        let window_title = fl!("app-title");

        // if let Some(page) = self.nav.text(self.nav.active()) {
        //     window_title.push_str(" — ");
        //     window_title.push_str(page);
        // }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
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
