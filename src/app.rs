// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::fl;
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
    /// Contains the list of all Pokémon
    pokemon_list: Vec<CustomPokemon>,
    /// Currently viewing Pokémon
    selected_pokemon: Option<CustomPokemon>,
}

#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    LoadedPokemonList(Vec<CustomPokemon>),
    LoadPokemon(String),
    LoadedPokemon(CustomPokemon),
    ReturnToLandingPage,
    DownloadAllImages,
    DownloadedAllImages,
}

/// Identifies a page in the application.
pub enum Page {
    LandingPage,
    PokemonPage,
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
    Settings,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
            Self::Settings => fl!("settings"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
    Settings,
    Back,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
            MenuAction::Settings => Message::ToggleContextPage(ContextPage::Settings),
            MenuAction::Back => Message::ReturnToLandingPage,
        }
    }
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
            //rustemon_client: rustemon::client::RustemonClient::default(),
            pokemon_list: Vec::<CustomPokemon>::new(),
            selected_pokemon: None,
        };

        let cmd = cosmic::app::Command::perform(load_all_pokemon(), |pokemon_list| {
            cosmic::app::message::app(Message::LoadedPokemonList(pokemon_list))
        });
        let commands = Command::batch(vec![app.update_titles(), cmd]);

        (app, commands)
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        let menu_bar = menu::bar(vec![
            menu::Tree::with_children(
                menu::root(fl!("view")),
                menu::items(
                    &self.key_binds,
                    vec![
                        menu::Item::Button(fl!("about"), MenuAction::About),
                        menu::Item::Button(fl!("settings"), MenuAction::Settings),
                    ],
                ),
            ),
            //TODO: This should be a button that allows to go back?
            menu::Tree::with_children(
                menu::root(fl!("back")),
                menu::items(
                    &self.key_binds,
                    vec![menu::Item::Button(fl!("back"), MenuAction::Back)],
                ),
            ),
        ]);

        vec![menu_bar.into()]
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match self.current_page {
            Page::LandingPage => self.landing(),
            Page::PokemonPage => self.pokemon_page(),
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
                self.pokemon_list = pokemons;
            }
            Message::LoadedPokemon(pokemon) => {
                self.selected_pokemon = Some(pokemon);
                self.current_page = Page::PokemonPage;
                let _ = self.update_titles();
            }
            Message::LoadPokemon(pokemon_name) => {
                return cosmic::app::Command::perform(load_pokemon(pokemon_name), |pokemon| {
                    cosmic::app::message::app(Message::LoadedPokemon(pokemon))
                });
            }
            Message::ReturnToLandingPage => self.current_page = Page::LandingPage,
            Message::DownloadAllImages => {
                return cosmic::app::Command::perform(download_all_pokemon_sprites(), |_| {
                    cosmic::app::message::app(Message::DownloadedAllImages)
                });
            }
            //TODO:
            Message::DownloadedAllImages => todo!(),
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
                    .push(widget::text::text(fl!("download_all_info")).size(10.0)),
            )
            .push(
                widget::button(widget::text::text(fl!("download")))
                    .on_press(Message::DownloadAllImages)
                    .style(theme::Button::Suggested),
            );

        widget::column()
            .push(download_row)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn landing(&self) -> Element<Message> {
        let space_xxs = theme::active().cosmic().spacing.space_xxs;

        let children = self.pokemon_list.iter().map(|custom_pokemon| {
            //TODO: This is temporal to reduce lag while not on release mode.
            // let pokemon_image = if let Some(path) = &custom_pokemon.sprite_path {
            //     widget::Image::new(path).content_fit(cosmic::iced::ContentFit::Fill)
            // } else {
            //     widget::Image::new("resources/fallback.png")
            //         .content_fit(cosmic::iced::ContentFit::Fill)
            // };
            let pokemon_image = widget::Image::new("resources/fallback.png")
                .content_fit(cosmic::iced::ContentFit::Fill);

            let pokemon_column = widget::Column::new().push(pokemon_image).push(
                widget::button(
                    widget::text::text(&custom_pokemon.pokemon.name)
                        .width(Length::Shrink)
                        .height(Length::Shrink)
                        .horizontal_alignment(Horizontal::Center),
                )
                .on_press_down(Message::LoadPokemon(
                    custom_pokemon.pokemon.name.to_string(),
                )),
            );

            widget::container(pokemon_column).into()
        });

        widget::scrollable(
            Column::with_children(children)
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .spacing(space_xxs),
        )
        .into()
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
                        .align_items(Alignment::Center),
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
                        .align_items(Alignment::Center),
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
            Page::PokemonPage => {
                window_title.push_str(" — ");
                window_title.push_str("Pokémon");
                header_title.push_str("Pokémon");
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
                .push(widget::text(capitalize_string(&pokemon_stats.stat.name)))
                .push(widget::text(pokemon_stats.base_stat.to_string()))
                .spacing(10.0)
                .into()
        });

        widget::container::Container::new(
            Column::with_children(children).align_items(Alignment::Center),
        )
        .style(theme::Container::ContextDrawer)
        .padding([spacing.space_none, spacing.space_xxs])
        .into()
    }

    pub fn parse_pokemon_types(
        &self,
        types: &Vec<PokemonType>,
        spacing: &cosmic_theme::Spacing,
    ) -> Element<Message> {
        //TODO: Missing card title

        let children = types.iter().map(|pokemon_types| {
            widget::Row::new()
                .push(widget::text(pokemon_types.type_.name.to_uppercase()))
                .into()
        });

        widget::container::Container::new(
            Column::with_children(children).align_items(Alignment::Center),
        )
        .style(theme::Container::ContextDrawer)
        .padding([spacing.space_none, spacing.space_xxs])
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct CustomPokemon {
    pokemon: Pokemon,
    sprite_path: Option<String>,
}

async fn load_all_pokemon() -> Vec<CustomPokemon> {
    let client = rustemon::client::RustemonClient::default();
    let all_entries = rustemon::pokemon::pokemon::get_all_entries(&client)
        .await
        .unwrap_or_default();

    let mut result = Vec::<CustomPokemon>::new();

    for entry in all_entries {
        let image_filename = format!("{}_front.png", entry.name);
        let image_path = format!("resources/sprites/{}/{}", entry.name, image_filename);

        result.push(CustomPokemon {
            pokemon: Pokemon {
                name: entry.name,
                ..Default::default()
            },
            sprite_path: Some(image_path),
        })
    }

    result
}

async fn load_pokemon(pokemon_name: String) -> CustomPokemon {
    let client = rustemon::client::RustemonClient::default();
    let pokemon = rustemon::pokemon::pokemon::get_by_name(pokemon_name.as_str(), &client)
        .await
        .unwrap_or_default();

    let image_path = if let Some(front_default_sprite) = &pokemon.sprites.front_default {
        // Only download the image if front_default sprite is available
        Some(
            download_image(front_default_sprite.to_string(), pokemon.name.to_string())
                .await
                .unwrap_or_else(|_| String::new()),
        )
    } else {
        None
    };

    CustomPokemon {
        pokemon: pokemon,
        sprite_path: image_path, // Set default if image_path is None
    }
}

async fn download_all_pokemon_sprites() {
    let client = rustemon::client::RustemonClient::default();
    let all_entries = rustemon::pokemon::pokemon::get_all_entries(&client)
        .await
        .unwrap_or_default();

    for entry in all_entries {
        let pokemon = rustemon::pokemon::pokemon::get_by_name(&entry.name, &client)
            .await
            .unwrap_or_default();

        let _ = download_image(
            pokemon.sprites.front_default.unwrap_or_default(),
            pokemon.name.to_string(),
        )
        .await;
    }
}

fn capitalize_string(input: &str) -> String {
    let words: Vec<&str> = input.split('-').collect();

    let capitalized_words: Vec<String> = words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            if let Some(first_char) = chars.next() {
                first_char.to_uppercase().collect::<String>() + chars.as_str()
            } else {
                String::new()
            }
        })
        .collect();

    capitalized_words.join(" ")
}

fn scale_numbers(num: i64) -> f64 {
    (num as f64) / 10.0
}

async fn download_image(
    image_url: String,
    pokemon_name: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let image_filename = format!("{}_front.png", pokemon_name);
    let image_path = format!("resources/sprites/{}/{}", pokemon_name, image_filename);

    // file already downloaded?
    if tokio::fs::metadata(&image_path).await.is_ok() {
        return Ok(image_path);
    }

    let response = reqwest::get(&image_url).await?;

    if response.status().is_success() {
        let bytes = response.bytes().await?;

        let path = std::path::PathBuf::from(&image_path);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        save_image(&image_path, &bytes).await?;
        Ok(image_path)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to download image. Status: {}", response.status()),
        )))
    }
}

async fn save_image(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = tokio::fs::File::create(path).await?;
    tokio::io::AsyncWriteExt::write_all(&mut file, bytes).await?;
    Ok(())
}
