// SPDX-License-Identifier: GPL-3.0-only

use crate::api::Api;
use crate::config::{AppTheme, Config};
use crate::fl;
use crate::image_cache::ImageCache;
use crate::utils::{capitalize_string, scale_numbers};
use cosmic::app::{Command, Core};
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Pixels, Subscription};
use cosmic::iced_core::text::LineHeight;
use cosmic::widget::{self, menu, Column};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Element};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

const REPOSITORY: &str = "https://github.com/mariinkys/starrydex";
const APP_ICON: &[u8] =
    include_bytes!("../res/icons/hicolor/256x256/apps/dev.mariinkys.StarryDex.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct StarryDex {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
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
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    UpdateTheme(usize),

    LoadPokemon(i64),
    TogglePokemonDetails(bool),
    Search(String),

    CompletedFirstRun(Config, BTreeMap<i64, StarryPokemon>),
    LoadedPokemonList(BTreeMap<i64, StarryPokemon>),
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

/// Identifies the status of a page in the application.
pub enum PageStatus {
    FirstRun,
    Loaded,
    Loading,
}

/// Create a COSMIC application from the app model
impl Application for StarryDex {
    /// The async executor that will be used to run your application's commands.
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

    /// Initializes the application with any given flags and startup commands.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // Commands that will get executed on the application init
        let mut commands = vec![];

        // Controls if it's the first time the application runs on a system
        let mut first_run_completed = false;

        // Construct the app model with the runtime's core.
        let mut app = StarryDex {
            core,
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
        };
        // Startup command that sets the window title.
        commands.push(app.update_title());

        // Create the directory where all of our application data will exist
        let app_data_dir = dirs::data_dir().unwrap().join(Self::APP_ID);
        std::fs::create_dir_all(&app_data_dir).expect("Failed to create the app data directory");

        // Clone the app api in order to use it.
        let api_clone = app.api.clone();

        if !first_run_completed {
            // First application run, construct cache, download sprites and update the config
            app.current_page_status = PageStatus::FirstRun;
            commands.push(cosmic::app::Command::perform(
                async move { api_clone.load_all_pokemon().await },
                |pokemon_list| {
                    cosmic::app::message::app(Message::CompletedFirstRun(
                        Config {
                            app_theme: crate::config::AppTheme::System,
                            first_run_completed: true,
                        },
                        pokemon_list,
                    ))
                },
            ));
        } else {
            // Load  the Pokémon List
            app.current_page_status = PageStatus::Loading;
            commands.push(cosmic::app::Command::perform(
                async move { api_clone.load_all_pokemon().await },
                |pokemon_list| cosmic::app::message::app(Message::LoadedPokemonList(pokemon_list)),
            ));
        }

        (app, Command::batch(commands))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("about"), MenuAction::About),
                    menu::Item::Button(String::from("Settings"), MenuAction::Settings),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::Settings => self.settings(),
            ContextPage::PokemonPage => self.single_pokemon_page(),
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
                .push(widget::text::text(fl!("downloading-sprites")))
                .push(widget::text::text(fl!("estimate")))
                .push(widget::text::text(fl!("once-message")))
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .spacing(space_s)
                .into(),
            PageStatus::Loaded => self.landing(),
            PageStatus::Loading => Column::new()
                .push(widget::text::text(fl!("loading")))
                .align_items(Alignment::Center)
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
    /// Commands may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
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

                // Set the title of the context drawer.
                self.set_context_title(context_page.title());
            }
            Message::UpdateConfig(config) => {
                self.config = config;
                return cosmic::app::command::set_theme(self.config.app_theme.theme());
            }
            Message::UpdateTheme(index) => {
                let app_theme = match index {
                    1 => AppTheme::Dark,
                    2 => AppTheme::Light,
                    _ => AppTheme::System,
                };
                self.config = Config {
                    first_run_completed: true,
                    app_theme,
                };
                return cosmic::app::command::set_theme(self.config.app_theme.theme());
            }
            Message::CompletedFirstRun(config, pokemon_list) => {
                self.config = config;

                //self.pokemon_list = pokemon_list; //TODO: This is to temporarly fix an error that makes a empty pokemon to appear on the first position of the btree
                let mut pokemon_list = pokemon_list;
                pokemon_list.pop_first();
                self.pokemon_list = pokemon_list;

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

                // Set the title of the context drawer.
                self.set_context_title(ContextPage::PokemonPage.title());
            }
            Message::TogglePokemonDetails(value) => self.wants_pokemon_details = value,
            Message::Search(value) => {
                // TODO: Improve search speed? Search by id...
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
        }
        Command::none()
    }
}

impl StarryDex {
    /// The about context page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));

        let title = widget::text::title3(fl!("app-title"));

        let app_info = widget::text::text(fl!("app-info"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        let pokeapi_text = widget::text::text(fl!("pokeapi-text"));

        let nintendo_text = widget::text::text(fl!("nintendo-text"));

        let version_link = widget::button::link(format!("v{}", env!("CARGO_PKG_VERSION")))
            .on_press(Message::LaunchUrl(
                "https://github.com/mariinkys/starrydex/releases".to_string(),
            ))
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(app_info)
            .push(link)
            .push(pokeapi_text)
            .push(nintendo_text)
            .push(version_link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    /// The settings context page for this app.
    pub fn settings(&self) -> Element<Message> {
        //let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let app_theme_selected = match self.config.app_theme {
            AppTheme::Dark => 1,
            AppTheme::Light => 2,
            AppTheme::System => 0,
        };

        widget::settings::view_column(vec![widget::settings::section()
            .title(fl!("appearance"))
            .add(
                widget::settings::item::builder(fl!("theme")).control(widget::dropdown(
                    &self.app_themes,
                    Some(app_theme_selected),
                    Message::UpdateTheme,
                )),
            )
            .into()])
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
                    .align_items(Alignment::Center),
            )
            .width(Length::Fixed(200.0))
            .height(Length::Fixed(135.0))
            .on_press_down(Message::LoadPokemon(pokemon.pokemon.id))
            .style(theme::Button::Image)
            .padding([spacing.space_none, spacing.space_s]);

            // Insert a new row before adding the first Pokémon of each row
            if index % 3 == 0 {
                pokemon_grid = pokemon_grid.insert_row();
            }

            pokemon_grid = pokemon_grid.push(pokemon_container);
        }

        let search = widget::search_input(fl!("search"), &self.search)
            .style(theme::TextInput::Search)
            .on_input(Message::Search)
            .line_height(LineHeight::Absolute(Pixels(35.0)))
            .width(Length::Fill);

        let search_row = widget::Row::new().push(search).width(Length::Fill);

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

    /// The about context page for this app.
    pub fn single_pokemon_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let content: widget::Column<_> = match &self.selected_pokemon {
            Some(starry_pokemon) => {
                let page_title =
                    widget::text::title1(capitalize_string(starry_pokemon.pokemon.name.as_str()))
                        .width(Length::Fill)
                        .horizontal_alignment(Horizontal::Center);

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
                        .align_items(Alignment::Center)
                        .width(Length::Fill),
                )
                .style(theme::Container::ContextDrawer)
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
                        .align_items(Alignment::Center)
                        .width(Length::Fill),
                )
                .style(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_types = widget::container::Container::new(Column::with_children(
                    starry_pokemon.pokemon.types.iter().map(|poke_type| {
                        widget::Row::new()
                            .push(
                                widget::text(poke_type.to_uppercase())
                                    .width(Length::Fill)
                                    .horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .into()
                    }),
                ))
                .style(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_abilities = widget::container::Container::new(Column::with_children(
                    starry_pokemon.pokemon.abilities.iter().map(|poke_ability| {
                        widget::Row::new()
                            .push(
                                widget::text(poke_ability.to_uppercase())
                                    .width(Length::Fill)
                                    .horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .into()
                    }),
                ))
                .style(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_stats = widget::container::Container::new(
                    Column::new()
                        .push(
                            widget::Row::new()
                                .push(widget::text("HP").width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.hp.to_string())
                                        .horizontal_alignment(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text("Attack").width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.attack.to_string())
                                        .horizontal_alignment(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text("Defense").width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.defense.to_string())
                                        .horizontal_alignment(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text("Special Attack").width(Length::Fill))
                                .push(
                                    widget::text(
                                        starry_pokemon.pokemon.stats.sp_attack.to_string(),
                                    )
                                    .horizontal_alignment(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text("Special Defense").width(Length::Fill))
                                .push(
                                    widget::text(
                                        starry_pokemon.pokemon.stats.sp_defense.to_string(),
                                    )
                                    .horizontal_alignment(Horizontal::Left),
                                ),
                        )
                        .push(
                            widget::Row::new()
                                .push(widget::text("Speed").width(Length::Fill))
                                .push(
                                    widget::text(starry_pokemon.pokemon.stats.speed.to_string())
                                        .horizontal_alignment(Horizontal::Left),
                                ),
                        ),
                )
                .style(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let pokemon_first_row = widget::Row::new()
                    .push(pokemon_weight)
                    .push(pokemon_height)
                    .push(pokemon_types)
                    .spacing(8.0)
                    .align_items(Alignment::Center);

                let mut result_col = widget::Column::new()
                    .push(page_title)
                    .push(pokemon_image)
                    .push(pokemon_first_row)
                    .push(pokemon_abilities)
                    .push(pokemon_stats)
                    .align_items(Alignment::Center)
                    .spacing(10.0);

                let show_details = widget::Checkbox::new(
                    fl!("show-encounter-details"),
                    self.wants_pokemon_details,
                    Message::TogglePokemonDetails,
                );

                let encounter_info = match &starry_pokemon.encounter_info {
                    Some(info) => {
                        let children = info.iter().map(|ef| {
                            let mut version_column = widget::Column::new().width(Length::Fill);
                            version_column = version_column.push(
                                widget::text(capitalize_string(&ef.city))
                                    .style(theme::Text::Accent)
                                    .size(Pixels::from(15)),
                            );

                            for method in &ef.games_method {
                                version_column = version_column.push(widget::text(method));
                            }

                            version_column.into()
                        });

                        widget::container::Container::new(Column::with_children(children))
                            .style(theme::Container::ContextDrawer)
                            .padding([spacing.space_none, spacing.space_xxs])
                    }
                    None => widget::Container::new(widget::Text::new(fl!("no-encounter-info")))
                        .style(theme::Container::ContextDrawer),
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

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Command<Message> {
        let window_title = fl!("app-title");

        // if let Some(page) = self.nav.text(self.nav.active()) {
        //     window_title.push_str(" — ");
        //     window_title.push_str(page);
        // }

        self.set_window_title(window_title)
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
    PokemonPage,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
            Self::PokemonPage => fl!("pokemon-page"),
        }
    }
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
