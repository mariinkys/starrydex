// SPDX-License-Identifier: GPL-3.0-only

use image_cache::{ImageCache, IMAGE_CACHE};

mod api;
mod app;
mod config;
mod i18n;
mod image_cache;
mod utils;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Init the image cache
    IMAGE_CACHE.get_or_init(|| std::sync::Mutex::new(ImageCache::new()));

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size(cosmic::iced::Size::new(1200.0, 800.0));

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::StarryDex>(settings, ())
}
