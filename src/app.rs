// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::fl;
use cosmic::app::{Command, Core};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length};
use cosmic::iced_widget::Column;
use cosmic::widget::{self, icon, menu, nav_bar};
use cosmic::{cosmic_theme, theme, Application, ApplicationExt, Apply, Element};
use rustemon::model::pokemon::Pokemon;
use rustemon::model::resource::NamedApiResource;

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
    /// A model that contains all of the pages assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// The rustemon client for interacting with PokeApi
    //rustemon_client: rustemon::client::RustemonClient,
    /// Contains the list of all Pokémon
    pokemon_list: Vec<NamedApiResource<Pokemon>>,
    /// Currently viewing Pokémon
    selected_pokemon: Option<NamedApiResource<Pokemon>>,
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    LaunchUrl(String),
    ToggleContextPage(ContextPage),
    LoadedPokemonList(Vec<NamedApiResource<Pokemon>>),
}

/// Identifies a page in the application.
pub enum Page {
    Page1,
    Page2,
}

/// Identifies a context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

impl ContextPage {
    fn title(&self) -> String {
        match self {
            Self::About => fl!("about"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
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

    /// Instructs the cosmic runtime to use this model as the nav bar model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Command` type is used to send messages to your application. `Command::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text("Page 1")
            .data::<Page>(Page::Page1)
            .icon(icon::from_name("applications-science-symbolic"))
            .activate();

        nav.insert()
            .text("Page 2")
            .data::<Page>(Page::Page2)
            .icon(icon::from_name("applications-system-symbolic"));

        let mut app = CosmicDex {
            core,
            context_page: ContextPage::default(),
            key_binds: HashMap::new(),
            nav,
            //rustemon_client: rustemon::client::RustemonClient::default(),
            pokemon_list: Vec::<NamedApiResource<Pokemon>>::new(),
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
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<Self::Message> {
        let content = match self.nav.active_data::<Page>() {
            Some(page) => match page {
                Page::Page1 => self.landing(),
                Page::Page2 => self.testing_error_page(),
            },
            None => todo!(),
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Commands may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
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
        })
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        // Activate the page in the model.
        self.nav.activate(id);

        self.update_titles()
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

        widget::column()
            .push(icon)
            .push(title)
            .push(app_info)
            .push(link)
            .align_items(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    pub fn landing(&self) -> Element<Message> {
        let space_xxs = theme::active().cosmic().spacing.space_xxs;

        let children = self.pokemon_list.iter().map(|pokemon| {
            widget::text::text(pokemon.name.to_string())
                .width(Length::Fill)
                .height(Length::Shrink)
                .horizontal_alignment(Horizontal::Center)
                // .vertical_alignment(Vertical::Center)
                .into()
        });

        widget::scrollable(
            Column::with_children(children)
                .align_items(Alignment::Center)
                .spacing(space_xxs),
        )
        .into()
    }

    pub fn testing_error_page(&self) -> Element<Message> {
        widget::text::title1(fl!("generic_error"))
            .apply(widget::container)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    /// Updates the header and window titles.
    pub fn update_titles(&mut self) -> Command<Message> {
        let mut window_title = fl!("app-title");
        let mut header_title = String::new();

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
            header_title.push_str(page);
        }

        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }
}

async fn load_all_pokemon() -> Vec<NamedApiResource<Pokemon>> {
    let client = rustemon::client::RustemonClient::default();
    rustemon::pokemon::pokemon::get_all_entries(&client)
        .await
        .unwrap_or_default()
    // let pokemon: Vec<Pokemon> = api_all_pokemon
    //     .into_iter()
    //     .map(|pokemon_api_res| pokemon_api_res.name)
    //     .collect();
}
