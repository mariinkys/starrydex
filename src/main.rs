// SPDX-License-Identifier: GPL-3.0-only

use core::localization;

use app::StarryDex;
use i18n_embed::DesktopLanguageRequester;
/// The `app` module is used by convention to indicate the main component of our application.
mod app;
mod core;
mod utils;

fn main() -> cosmic::iced::Result {
    init_localizer();

    let (settings, flags) = core::settings::init();
    cosmic::app::run::<StarryDex>(settings, flags)
}

fn init_localizer() {
    let localizer = localization::localizer();
    let requested_languages = DesktopLanguageRequester::requested_languages();

    if let Err(why) = localizer.select(&requested_languages) {
        panic!("can't load localizations: {}", why);
    }
}
