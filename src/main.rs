// SPDX-License-Identifier: GPL-3.0-only

mod api;
mod app;
mod config;
mod i18n;
mod image_cache;
mod settings;
mod utils;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings and flags for configuring the application window and iced runtime.
    let (settings, flags) = settings::init();

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::StarryDex>(settings, flags)
}
