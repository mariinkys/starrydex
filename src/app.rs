// SPDX-License-Identifier: GPL-3.0

use std::collections::HashMap;

use cosmic::Application;
use cosmic::app::context_drawer;
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::core::keyboard::{Key, Modifiers};
use cosmic::iced::keyboard::key::Named;
use cosmic::iced::widget::operation::AbsoluteOffset;
use cosmic::iced::widget::{center, scrollable};
use cosmic::iced::{Event, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::menu::Action;
use cosmic::widget::{self, about::About, menu, segmented_button, text};
use starrydex_model::{Generation, PokemonType};

use crate::app::app_menu::MenuAction;
use crate::app::context_page::ContextPage;
use crate::app::core::{ListQuery, StarryCore};
use crate::app::pages::details::{DetailTab, Details};
use crate::app::utils::presentation::{Label, data_lang};
use crate::app::utils::{
    Category, Filters, PaginationAction, SortBy, play_audio_file, remove_dir_contents,
};
use crate::config::{AppTheme, ConfigInput, StarryConfig, TypeFilteringMode, ViewMode};
use crate::fl;
use crate::key_binds::key_binds;

pub mod app_menu;
mod context_page;
mod core;
mod pages;
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
    /// Application Themes
    app_themes: Vec<String>,
    /// Available Type Filter Modes
    type_filter_modes: Vec<String>,
    /// Available View Modes
    view_modes: Vec<String>,
    /// Labels of the sort options
    sort_options: Vec<String>,
    /// Tabs of the Pokémon details page
    detail_tabs: segmented_button::SingleSelectModel,
    /// Application State
    state: State,
}

pub enum State {
    Loading,
    Error(String),
    Ready(Box<Ready>),
}

/// Everything that exists once the Pokédex is loaded.
pub struct Ready {
    /// Memory-mapped Pokédex
    pub core: StarryCore,
    /// Indices (into the Pokédex) of the Pokémon matching the current search/filters/sort
    pub results: Vec<u32>,
    /// Current page of the list (0-based)
    pub page: usize,
    /// Search input value
    pub search: String,
    /// Currently applied filters
    pub filters: Filters,
    /// Curenntly applied sorting
    pub sort: SortBy,
    /// Is sort descending?
    pub descending: bool,
    /// The open Pokémon, if any
    pub details: Option<Details>,
    /// Scroll position of the Pokémon list, restored when coming back from a details page
    pub list_scroll: AbsoluteOffset,
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
    /// Callback after clicking something in the app menu
    MenuAction(app_menu::MenuAction),
    /// Executes the appropiate cosmic binding on keyboard shortcut
    Key(Modifiers, Key),
    /// Updates the current state of keyboard modifiers
    Modifiers(Modifiers),
    /// Callback after loading the application core
    CoreLoaded(Result<StarryCore, anywho::Error>),
    /// Open the details page of the Pokémon with the given id
    OpenPokemon(u32),
    /// Leave the details page (or go back to the previously viewed Pokémon)
    Back,
    /// A tab of the details page was selected
    DetailTab(segmented_button::Entity),
    /// Callback after input on the Config [`ContextPage`]
    ConfigInput(ConfigInput),
    /// Input on the details page
    Details(DetailsInput),
    /// Input on the Pokémon list
    List(ListInput),
    /// Input on the Filters [`ContextPage`]
    Filters(FiltersInput),
}

/// Some user interaction that happens on the Pokémon list
#[derive(Debug, Clone)]
pub enum ListInput {
    Page(PaginationAction),
    Search(String),
    Sort(usize),
    ToggleOrder,
    ClearAll,
    /// The list was scrolled (remembers where the user was)
    Scrolled(AbsoluteOffset),
    /// Scroll the list back to the remembered position
    RestoreScroll(AbsoluteOffset),
}

/// Some user interaction that happens on the details page
#[derive(Debug, Clone)]
pub enum DetailsInput {
    Step(PaginationAction),
    PlayAudio(String),
    ToggleShiny,
    ToggleBack,
    ToggleFemale,
    FlavorEntry(usize),
    MoveVersionGroup(usize),
    ToggleMove(u32),
    EncounterVersion(usize),
}

/// Some user interaction that happens on the Filters [`ContextPage`]
#[derive(Debug, Clone)]
pub enum FiltersInput {
    Type(bool, PokemonType),
    Generation(bool, Generation),
    Category(bool, Category),
    StatsEnabled(bool),
    MinTotal(u16),
    Reset,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = crate::flags::Flags;
    type Message = Message;
    const APP_ID: &'static str = "dev.mariinkys.StarryDex";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    fn init(core: cosmic::Core, flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
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

        let mut detail_tabs = segmented_button::SingleSelectModel::default();
        for tab in DetailTab::ALL {
            detail_tabs.insert().text(tab.label()).data(*tab);
        }
        detail_tabs.activate_position(0);

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
            sort_options: SortBy::ALL.iter().map(Label::label).collect(),
            detail_tabs,
            state: State::Loading,
        };

        let tasks = vec![
            app.update_title(),
            cosmic::command::set_theme(app.config.app_theme.theme()),
            Task::perform(StarryCore::initialize(), |res| {
                cosmic::action::app(Message::CoreLoaded(res))
            }),
        ];

        (app, Task::batch(tasks))
    }

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

    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }
        self.context_page.display(self)
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let content: Element<_> = match &self.state {
            State::Loading => center(
                widget::column::with_children(vec![
                    widget::progress_bar::indeterminate_circular().into(),
                    text::body(fl!("loading")).into(),
                ])
                .spacing(12)
                .align_x(cosmic::iced::Alignment::Center),
            )
            .into(),
            State::Error(error) => center(text::body(error)).into(),
            State::Ready(ready) => match &ready.details {
                Some(details) => pages::details::view(
                    &ready.core,
                    details,
                    &self.detail_tabs,
                    self.active_tab(),
                    &ready.results,
                ),
                None => pages::home::view(self, ready),
            },
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            cosmic::iced::event::listen_with(|event, status, _| match event {
                Event::Keyboard(cosmic::iced::keyboard::Event::KeyPressed {
                    key,
                    modifiers,
                    ..
                }) => {
                    // Keys typed into a text input (the search box) are not shortcuts.
                    if status == cosmic::iced::event::Status::Captured {
                        return None;
                    }
                    match &key {
                        Key::Named(Named::ArrowLeft) if modifiers.is_empty() => {
                            Some(Message::Hotkey(Hotkey::ArrowLeft))
                        }
                        Key::Named(Named::ArrowRight) if modifiers.is_empty() => {
                            Some(Message::Hotkey(Hotkey::ArrowRight))
                        }
                        Key::Named(Named::Escape) => Some(Message::Hotkey(Hotkey::Escape)),
                        _ => Some(Message::Key(modifiers, key)),
                    }
                }
                Event::Keyboard(cosmic::iced::keyboard::Event::ModifiersChanged(modifiers)) => {
                    Some(Message::Modifiers(modifiers))
                }
                _ => None,
            }),
            self.core()
                .watch_config::<StarryConfig>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::Hotkey(hotkey) => self.on_hotkey(hotkey),
            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
                Task::none()
            }
            Message::UpdateConfig(config) => {
                let theme_changed = config.app_theme != self.config.app_theme;
                self.config = config;
                self.refresh_results();
                if theme_changed {
                    cosmic::command::set_theme(self.config.app_theme.theme())
                } else {
                    Task::none()
                }
            }
            Message::LaunchUrl(url) => {
                if let Err(err) = open::that_detached(&url) {
                    eprintln!("failed to open {url:?}: {err}");
                }
                Task::none()
            }
            Message::MenuAction(action) => match action {
                MenuAction::About => self.update(Message::ToggleContextPage(ContextPage::About)),
                MenuAction::Settings => {
                    self.update(Message::ToggleContextPage(ContextPage::Settings))
                }
            },
            Message::Key(modifiers, key) => {
                for (key_bind, action) in &self.key_binds {
                    if key_bind.matches(modifiers, &key, None) {
                        return self.update(action.message());
                    }
                }
                Task::none()
            }
            Message::Modifiers(modifiers) => {
                self.modifiers = modifiers;
                Task::none()
            }
            Message::CoreLoaded(Err(e)) => {
                self.state = State::Error(e.to_string());
                Task::none()
            }
            Message::CoreLoaded(Ok(core)) => {
                self.state = State::Ready(Box::new(Ready {
                    core,
                    results: Vec::new(),
                    page: 0,
                    search: String::new(),
                    filters: Filters::default(),
                    sort: SortBy::default(),
                    descending: false,
                    details: None,
                    list_scroll: AbsoluteOffset::default(),
                }));
                self.refresh_results();
                Task::none()
            }
            Message::OpenPokemon(id) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let Some(index) = ready.core.dex().pokemon_index(id) else {
                    return Task::none();
                };
                match &mut ready.details {
                    Some(details) if details.index != index => {
                        details.history.push(details.index);
                        details.show(index, ready.core.dex());
                    }
                    Some(_) => {}
                    None => ready.details = Some(Details::new(index, ready.core.dex())),
                }
                if self.context_page == ContextPage::FiltersPage {
                    self.core.window.show_context = false;
                }
                Task::none()
            }
            Message::Back => {
                if let State::Ready(ready) = &mut self.state
                    && let Some(details) = &mut ready.details
                {
                    match details.history.pop() {
                        Some(previous) => details.show(previous, ready.core.dex()),
                        None => {
                            ready.details = None;
                            // Restore on the next update, once the list view exists again. The offset is
                            // captured now because the new scrollable reports 0 through `on_scroll` when it
                            // first draws, which would overwrite `list_scroll`.
                            return Task::done(cosmic::action::app(Message::List(
                                ListInput::RestoreScroll(ready.list_scroll),
                            )));
                        }
                    }
                }
                Task::none()
            }
            Message::DetailTab(entity) => {
                self.detail_tabs.activate(entity);
                Task::none()
            }
            Message::ConfigInput(input) => self.on_config_input(input),
            Message::Details(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let Some(details) = &mut ready.details else {
                    return Task::none();
                };
                let dex = ready.core.dex();
                match input {
                    DetailsInput::Step(direction) => {
                        let next =
                            step(&ready.results, dex.pokemon.len(), details.index, &direction);
                        details.history.clear();
                        details.show(next, dex);
                    }
                    DetailsInput::PlayAudio(path) => {
                        #[cfg(unix)]
                        play_audio_file(&path, f32::from(self.config.cry_volume) / 100.0);
                    }
                    DetailsInput::ToggleShiny => details.shiny = !details.shiny,
                    DetailsInput::ToggleBack => details.back = !details.back,
                    DetailsInput::ToggleFemale => details.female = !details.female,
                    DetailsInput::FlavorEntry(index) => details.flavor_entry = index,
                    DetailsInput::MoveVersionGroup(index) => {
                        details.move_version_group =
                            details.move_version_groups.get(index).copied();
                        details.expanded_moves.clear();
                    }
                    DetailsInput::ToggleMove(id) => {
                        if !details.expanded_moves.remove(&id) {
                            details.expanded_moves.insert(id);
                        }
                    }
                    DetailsInput::EncounterVersion(index) => {
                        // Index 0 is "all games".
                        details.encounter_version = index
                            .checked_sub(1)
                            .and_then(|i| details.encounter_versions.get(i).copied());
                    }
                }
                Task::none()
            }
            Message::List(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                match input {
                    ListInput::Page(action) => {
                        let pages = page_count(ready.results.len(), self.config.pokemon_per_page);
                        ready.page = match action {
                            PaginationAction::Next => (ready.page + 1).min(pages - 1),
                            PaginationAction::Back => ready.page.saturating_sub(1),
                        };

                        // go back to the top of the page when changing paage
                        ready.list_scroll = AbsoluteOffset::default();
                        return scrollable::scroll_to(
                            pages::home::LIST_SCROLL_ID.into(),
                            AbsoluteOffset {
                                x: None,
                                y: Some(0.0),
                            },
                        );
                    }
                    ListInput::Search(value) => ready.search = value,
                    ListInput::Sort(index) => {
                        ready.sort = SortBy::ALL.get(index).copied().unwrap_or_default();
                        ready.descending = false;
                    }
                    ListInput::ToggleOrder => ready.descending = !ready.descending,
                    ListInput::ClearAll => {
                        ready.search.clear();
                        ready.filters = Filters::default();
                        ready.sort = SortBy::default();
                        ready.descending = false;
                    }
                    ListInput::Scrolled(offset) => {
                        ready.list_scroll = offset;
                        return Task::none();
                    }
                    ListInput::RestoreScroll(offset) => {
                        ready.list_scroll = offset;
                        return scrollable::scroll_to(
                            pages::home::LIST_SCROLL_ID.into(),
                            offset.into(),
                        );
                    }
                }
                self.refresh_results();
                Task::none()
            }
            Message::Filters(input) => {
                let State::Ready(ready) = &mut self.state else {
                    return Task::none();
                };
                let filters = &mut ready.filters;
                match input {
                    FiltersInput::Type(on, t) => toggle(&mut filters.types, t, on),
                    FiltersInput::Generation(on, g) => toggle(&mut filters.generations, g, on),
                    FiltersInput::Category(on, c) => toggle(&mut filters.categories, c, on),
                    FiltersInput::StatsEnabled(on) => filters.stats_enabled = on,
                    FiltersInput::MinTotal(value) => filters.min_total = value,
                    FiltersInput::Reset => *filters = Filters::default(),
                }
                self.refresh_results();
                Task::none()
            }
        }
    }
}

impl AppModel {
    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(String::from("StarryDex"), id)
        } else {
            Task::none()
        }
    }

    fn active_tab(&self) -> DetailTab {
        self.detail_tabs
            .active_data::<DetailTab>()
            .copied()
            .unwrap_or_default()
    }

    /// Re-runs the list query after search, filters, sorting or settings changed.
    fn refresh_results(&mut self) {
        let State::Ready(ready) = &mut self.state else {
            return;
        };
        ready.results = ready.core.query(&ListQuery {
            search: &ready.search,
            filters: &ready.filters,
            type_mode: self.config.type_filtering_mode,
            show_forms: self.config.show_forms,
            sort: ready.sort,
            descending: ready.descending,
            lang: data_lang(),
        });
        ready.page = 0;
        ready.list_scroll = AbsoluteOffset::default();
    }

    fn on_hotkey(&mut self, hotkey: Hotkey) -> Task<cosmic::Action<Message>> {
        let State::Ready(ready) = &self.state else {
            return Task::none();
        };
        let in_details = ready.details.is_some();
        match hotkey {
            Hotkey::Escape if self.core.window.show_context => {
                self.core.window.show_context = false;
                Task::none()
            }
            Hotkey::Escape if in_details => self.update(Message::Back),
            Hotkey::Escape => Task::none(),
            Hotkey::ArrowLeft | Hotkey::ArrowRight => {
                let direction = if matches!(hotkey, Hotkey::ArrowLeft) {
                    PaginationAction::Back
                } else {
                    PaginationAction::Next
                };
                if in_details {
                    self.update(Message::Details(DetailsInput::Step(direction)))
                } else {
                    self.update(Message::List(ListInput::Page(direction)))
                }
            }
        }
    }

    fn save_config(
        &mut self,
        apply: impl Fn(
            &mut StarryConfig,
            &cosmic::cosmic_config::Config,
        ) -> Result<bool, cosmic::cosmic_config::Error>,
        fallback: impl Fn(&mut StarryConfig),
    ) {
        if let Some(handler) = &self.config_handler {
            if let Err(err) = apply(&mut self.config, handler) {
                eprintln!("Failed to save setting: {err}");
                // Keep the change for this session even if it can't be persisted.
                fallback(&mut self.config);
            }
        } else {
            fallback(&mut self.config);
        }
    }

    fn on_config_input(&mut self, input: ConfigInput) -> Task<cosmic::Action<Message>> {
        match input {
            ConfigInput::UpdateTheme(index) => {
                let app_theme = match index {
                    1 => AppTheme::Dark,
                    2 => AppTheme::Light,
                    _ => AppTheme::System,
                };
                self.save_config(
                    |c, h| c.set_app_theme(h, app_theme),
                    |c| c.app_theme = app_theme,
                );
                return cosmic::command::set_theme(self.config.app_theme.theme());
            }
            ConfigInput::UpdateViewMode(index) => {
                let per_row = match self.config.view_mode {
                    ViewMode::Manual { pokemon_per_row } => pokemon_per_row,
                    ViewMode::Responsive => 5,
                };
                let view_mode = if index == 1 {
                    ViewMode::Manual {
                        pokemon_per_row: per_row,
                    }
                } else {
                    ViewMode::Responsive
                };
                self.save_config(
                    |c, h| c.set_view_mode(h, view_mode),
                    |c| c.view_mode = view_mode,
                );
            }
            ConfigInput::UpdatePokemonPerRow(v) => {
                if let ViewMode::Manual { .. } = self.config.view_mode {
                    let view_mode = ViewMode::Manual {
                        pokemon_per_row: usize::from(v),
                    };
                    self.save_config(
                        |c, h| c.set_view_mode(h, view_mode),
                        |c| c.view_mode = view_mode,
                    );
                }
            }
            ConfigInput::UpdatePokemonPerPage(v) => {
                let value = usize::from(v);
                self.save_config(
                    |c, h| c.set_pokemon_per_page(h, value),
                    |c| c.pokemon_per_page = value,
                );
                self.refresh_results();
            }
            ConfigInput::UpdateTypeFilterMode(index) => {
                let mode = if index == 1 {
                    TypeFilteringMode::Inclusive
                } else {
                    TypeFilteringMode::Exclusive
                };
                self.save_config(
                    |c, h| c.set_type_filtering_mode(h, mode),
                    |c| c.type_filtering_mode = mode,
                );
                self.refresh_results();
            }
            ConfigInput::UpdateShowForms(value) => {
                self.save_config(|c, h| c.set_show_forms(h, value), |c| c.show_forms = value);
                self.refresh_results();
            }
            ConfigInput::DeleteCache => {
                self.core.window.show_context = false;
                self.state = State::Loading;
                let data_dir = dirs::data_dir().map(|d| d.join(Self::APP_ID));
                if let Some(dir) = data_dir
                    && let Err(e) = remove_dir_contents(&dir)
                {
                    eprintln!("Error deleting cache: {e}");
                }
                return Task::perform(StarryCore::initialize(), |core| {
                    cosmic::action::app(Message::CoreLoaded(core))
                });
            }
            ConfigInput::UpdateCryVolume(value) => {
                self.save_config(|c, h| c.set_cry_volume(h, value), |c| c.cry_volume = value);
            }
        }
        Task::none()
    }

    /// The settings context page for this app.
    pub fn settings(&self) -> Element<'_, Message> {
        pages::settings::view(self)
    }
}

fn toggle<T: Ord>(set: &mut std::collections::BTreeSet<T>, value: T, on: bool) {
    if on {
        set.insert(value);
    } else {
        set.remove(&value);
    }
}

/// Number of pages for `len` results (at least one).
pub fn page_count(len: usize, per_page: usize) -> usize {
    len.div_ceil(per_page.max(1)).max(1)
}

/// Index of the previous/next Pokémon: inside the current results when the open Pokémon is
/// part of them, otherwise in plain Pokédex order.
fn step(results: &[u32], total: usize, current: usize, direction: &PaginationAction) -> usize {
    let forward = matches!(direction, PaginationAction::Next);
    if let Some(position) = results.iter().position(|&i| i as usize == current) {
        let len = results.len();
        let next = if forward {
            (position + 1) % len
        } else {
            (position + len - 1) % len
        };
        return results[next] as usize;
    }
    if total == 0 {
        return current;
    }
    if forward {
        (current + 1) % total
    } else {
        (current + total - 1) % total
    }
}

#[derive(Debug, Clone)]
pub enum Hotkey {
    ArrowLeft,
    ArrowRight,
    Escape,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepping_wraps_around_results() {
        let results = [5, 9, 2];
        assert_eq!(step(&results, 100, 9, &PaginationAction::Next), 2);
        assert_eq!(step(&results, 100, 2, &PaginationAction::Next), 5);
        assert_eq!(step(&results, 100, 5, &PaginationAction::Back), 2);
        // Not in results → dex order
        assert_eq!(step(&results, 100, 50, &PaginationAction::Next), 51);
        assert_eq!(step(&results, 100, 0, &PaginationAction::Back), 99);
    }

    #[test]
    fn page_counts() {
        assert_eq!(page_count(0, 30), 1);
        assert_eq!(page_count(30, 30), 1);
        assert_eq!(page_count(31, 30), 2);
    }
}
