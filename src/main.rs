// SPDX-License-Identifier: GPL-3.0-only
#![allow(clippy::collapsible_if)]
#![allow(mismatched_lifetime_syntaxes)]
use icon_cache::{ICON_CACHE, IconCache};
use image_cache::{IMAGE_CACHE, ImageCache};

use crate::flags::flags;

mod app;
mod config;
mod core;
mod entities;
mod flags;
mod i18n;
mod icon_cache;
mod image_cache;
mod utils;
mod widgets;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Init the image cache
    IMAGE_CACHE.get_or_init(|| std::sync::Mutex::new(ImageCache::new()));

    // Init the icon cache
    ICON_CACHE.get_or_init(|| std::sync::Mutex::new(IconCache::new()));

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size(cosmic::iced::Size::new(1200.0, 800.0));

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::StarryDex>(settings, flags())
}
