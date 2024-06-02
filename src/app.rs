// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::fl;
use crate::utils::{
    capitalize_string, download_all_pokemon_sprites, load_all_pokemon, load_pokemon, scale_numbers,
};
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::Column;
use cosmic::widget::{self, menu};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Apply, Element};
use rustemon::model::pokemon::{Pokemon, PokemonStat, PokemonType};

const REPOSITORY: &str = "https://github.com/mariinkys/cosmicdex";

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct CosmicDex {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    /// Currently selected Page
    current_page: Page,
    /// Page Status
    page_status: PageStatus,
    /// Contains the list of all Pokémon
    pokemon_list: Vec<CustomPokemon>,
    /// Contains the list of pokemon after searching
    filtered_pokemon_list: Vec<CustomPokemon>,
    /// Currently viewing Pokémon
    selected_pokemon: Option<CustomPokemon>,
    /// Holds the search input value
    search: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    LoadedPokemonList(Vec<CustomPokemon>),
    LoadPokemon(String),
    LoadedPokemon(CustomPokemon),
    DownloadAllImages,
    DownloadedAllImages,
    Search(String),
}

/// Identifies a page in the application.
pub enum Page {
    LandingPage,
}

/// Identifies the status of a page in the application.
pub enum PageStatus {
    Loaded,
    Loading,
}

/// Identifies a context page to display in the context drawer.
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
            Self::PokemonPage => fl!("pokemon_page"),
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

#[derive(Debug, Clone)]
pub struct CustomPokemon {
    pub pokemon: Pokemon,
    pub sprite_path: Option<String>,
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for CosmicDex {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "dev.mariinkys.CosmicDex";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut app = CosmicDex {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            current_page: Page::LandingPage,
            pokemon_list: Vec::<CustomPokemon>::new(),
            filtered_pokemon_list: Vec::<CustomPokemon>::new(),
            selected_pokemon: None,
            page_status: PageStatus::Loading,
            search: String::new(),
        };

        let cmd = cosmic::app::Command::perform(load_all_pokemon(), |pokemon_list| {
            cosmic::app::message::app(Message::LoadedPokemonList(pokemon_list))
        });
        let commands = Command::batch(vec![app.update_titles(), cmd]);

        (app, commands)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![
                    menu::Item::Button(fl!("about"), MenuAction::About),
                    menu::Item::Button(fl!("settings"), MenuAction::Settings),
                ],
            ),
        )]);

        vec![menu_bar.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::LandingPage => self.landing(),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::LaunchUrl(url) => {
                let _result = open::that_detached(url);
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
            Message::LoadedPokemonList(pokemons) => {
                self.pokemon_list = pokemons.clone();
                self.filtered_pokemon_list = pokemons;
                self.page_status = PageStatus::Loaded;
            }
            Message::LoadedPokemon(pokemon) => {
                self.selected_pokemon = Some(pokemon);

                if self.context_page == ContextPage::PokemonPage {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = ContextPage::PokemonPage;
                    self.core.window.show_context = true;
                }

                // Set the title of the context drawer.
                self.set_context_title(ContextPage::PokemonPage.title());
            }
            Message::LoadPokemon(pokemon_name) => {
                return cosmic::app::Command::perform(load_pokemon(pokemon_name), |pokemon| {
                    cosmic::app::message::app(Message::LoadedPokemon(pokemon))
                });
            }
            Message::DownloadAllImages => {
                return cosmic::app::Command::perform(download_all_pokemon_sprites(), |_| {
                    cosmic::app::message::app(Message::DownloadedAllImages)
                });
            }
            //TODO:
            Message::DownloadedAllImages => todo!(),
            Message::Search(new_value) => {
                self.search = new_value;
                self.filtered_pokemon_list = self
                    .pokemon_list
                    .clone()
                    .into_iter()
                    .filter(|pokemon| pokemon.pokemon.name.contains(&self.search))
                    .collect();
            }
        }
        Command::none()
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => self.about(),
            ContextPage::Settings => self.settings(),
            ContextPage::PokemonPage => self.pokemon_page(),
        })
    }
}

impl CosmicDex {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(
            &include_bytes!("../res/icons/hicolor/128x128/apps/dev.mariinkys.CosmicDex.svg")[..],
        ));

        let title = widget::text::title3(fl!("app-title"));

        let app_info = widget::text::text(fl!("app_info"));

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::LaunchUrl(REPOSITORY.to_string()))
            .padding(0);

        let pokeapi_text = widget::text::text(fl!("pokeapi_text"));

        let nintendo_text = widget::text::text(fl!("nintendo_text"));

        widget::column()
            .push(icon)
            .push(title)
            .push(app_info)
            .push(link)
            .push(pokeapi_text)
            .push(nintendo_text)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn settings(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let download_row = widget::Row::new()
            .push(
                widget::column()
                    .push(widget::text::text(fl!("download_all_images")))
                    .push(widget::text::text(fl!("download_all_info")).size(10.0))
                    .width(Length::Fill),
            )
            .push(
                widget::button(widget::text::text(fl!("download")))
                    .on_press(Message::DownloadAllImages)
                    .style(theme::Button::Suggested)
                    .width(Length::Shrink),
            );

        widget::column()
            .push(download_row)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn landing(&self) -> Element<Message> {
        let space_s = theme::active().cosmic().spacing.space_s;
        let spacing = theme::active().cosmic().spacing;

        match self.page_status {
            PageStatus::Loaded => {
                let pokemon_children = self.filtered_pokemon_list.iter().map(|custom_pokemon| {
                    let pokemon_image = if let Some(path) = &custom_pokemon.sprite_path {
                        widget::Image::new(path).content_fit(cosmic::iced::ContentFit::Fill)
                    } else {
                        widget::Image::new("resources/fallback.png")
                            .content_fit(cosmic::iced::ContentFit::Fill)
                    };

                    let pokemon_container = widget::button(
                        widget::Column::new()
                            .push(pokemon_image)
                            .push(widget::text::text(capitalize_string(
                                &custom_pokemon.pokemon.name,
                            )))
                            .align_items(Alignment::Center),
                    )
                    .on_press_down(Message::LoadPokemon(
                        custom_pokemon.pokemon.name.to_string(),
                    ))
                    .style(theme::Button::Image)
                    .padding([spacing.space_none, spacing.space_s]);

                    pokemon_container.into()
                });

                let search = widget::search_input(fl!("search"), &self.search)
                    .on_input(Message::Search)
                    .on_clear(Message::Search(String::new()))
                    // .id(self.text_input_id.clone())
                    // .on_submit(Message::Enter)
                    .width(Length::Fill);

                let search_row = widget::Row::new()
                    .push(search)
                    .width(Length::Fill)
                    .padding(5.0);

                //TODO: This should not be a column, how canI have some kind of responsive grid?
                //The grid widget does not have ::with_children, how can I push my content?
                let pokemon_list = Column::with_children(pokemon_children)
                    .align_items(Alignment::Center)
                    .width(Length::Fill)
                    .spacing(space_s);

                //TODO: The searchbar should not scroll with the pokemon_list but if I try to put it outisde of the scrollable it disappears.
                let content = widget::Column::new()
                    .push(search_row)
                    .push(pokemon_list)
                    .width(Length::Fill)
                    .spacing(5.0);

                widget::scrollable(content).into()
            }
            PageStatus::Loading => Column::new()
                .push(widget::text::text("Loading..."))
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .spacing(space_s)
                .into(),
        }
    }

    pub fn pokemon_page(&self) -> Element<Message> {
        let spacing = theme::active().cosmic().spacing;

        let content: widget::Column<_> = match &self.selected_pokemon {
            Some(custom_pokemon) => {
                let page_title =
                    widget::text::title1(capitalize_string(custom_pokemon.pokemon.name.as_str()))
                        .width(Length::Fill)
                        .horizontal_alignment(Horizontal::Center);

                //TODO: Fallback image
                let pokemon_image = if let Some(path) = &custom_pokemon.sprite_path {
                    widget::Image::new(path).content_fit(cosmic::iced::ContentFit::Fill)
                } else {
                    widget::Image::new("resources/fallback.png")
                        .content_fit(cosmic::iced::ContentFit::Fill)
                };

                let pokemon_weight = widget::container::Container::new(
                    widget::Column::new()
                        .push(widget::text::title3("WEIGHT"))
                        .push(
                            widget::text::text(format!(
                                "{} Kg",
                                scale_numbers(custom_pokemon.pokemon.weight).to_string()
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
                        .push(widget::text::title3("HEIGHT"))
                        .push(
                            widget::text::text(format!(
                                "{} m",
                                scale_numbers(custom_pokemon.pokemon.height).to_string()
                            ))
                            .size(15.0),
                        )
                        .align_items(Alignment::Center)
                        .width(Length::Fill),
                )
                .style(theme::Container::ContextDrawer)
                .padding([spacing.space_none, spacing.space_xxs]);

                let parsed_pokemon_types =
                    self.parse_pokemon_types(&custom_pokemon.pokemon.types, &spacing);

                let pokemon_first_row = widget::Row::new()
                    .push(pokemon_weight)
                    .push(pokemon_height)
                    .push(parsed_pokemon_types)
                    .spacing(8.0)
                    .align_items(Alignment::Center);

                let parsed_pokemon_stats =
                    self.parse_pokemon_stats(&custom_pokemon.pokemon.stats, &spacing);

                widget::Column::new()
                    .push(page_title)
                    .push(pokemon_image)
                    .push(pokemon_first_row)
                    .push(parsed_pokemon_stats)
                    .align_items(Alignment::Center)
                    .spacing(10.0)
                    .into()
            }
            None => {
                let error = widget::text::title1(fl!("generic_error"))
                    .apply(widget::container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center);

                widget::Column::new().push(error).into()
            }
        };

        widget::container(content).into()
    }

    /// Updates the header and window titles.
    pub fn update_titles(&mut self) -> Command<Message> {
        let mut window_title = fl!("app-title");
        let mut header_title = String::new();

        match self.current_page {
            Page::LandingPage => {
                window_title.push_str(" — ");
                window_title.push_str("All Pokémon");
                header_title.push_str("All Pokémon");
            }
        }

        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }

    pub fn parse_pokemon_stats(
        &self,
        stats: &Vec<PokemonStat>,
        spacing: &cosmic_theme::Spacing,
    ) -> Element<Message> {
        //TODO: Missing card title

        let children = stats.iter().map(|pokemon_stats| {
            widget::Row::new()
                .push(widget::text(capitalize_string(&pokemon_stats.stat.name)).width(Length::Fill))
                .push(
                    widget::text(pokemon_stats.base_stat.to_string())
                        .horizontal_alignment(Horizontal::Left),
                )
                .into()
        });

        widget::container::Container::new(Column::with_children(children))
            .style(theme::Container::ContextDrawer)
            .padding([spacing.space_none, spacing.space_xxs])
            .into()
    }

    pub fn parse_pokemon_types(
        &self,
        types: &Vec<PokemonType>,
        spacing: &cosmic_theme::Spacing,
    ) -> Element<Message> {
        let children = types.iter().map(|pokemon_types| {
            widget::Row::new()
                .push(
                    widget::text(pokemon_types.type_.name.to_uppercase())
                        .width(Length::Fill)
                        .horizontal_alignment(Horizontal::Center),
                )
                .width(Length::Fill)
                .into()
        });

        widget::container::Container::new(Column::with_children(children))
            .style(theme::Container::ContextDrawer)
            .padding([spacing.space_none, spacing.space_xxs])
            .into()
    }
}
