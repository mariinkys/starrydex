use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};

use crate::app::Flags;

use super::{
    config::StarryConfig,
    image_cache::{ImageCache, IMAGE_CACHE},
};

pub fn init() -> (Settings, Flags) {
    set_image_cache();

    let settings = get_app_settings();
    let flags = get_flags();

    (settings, flags)
}

pub fn get_app_settings() -> Settings {
    let mut settings = Settings::default();

    settings = settings.size_limits(Limits::NONE.min_width(500.0).min_height(180.0));
    settings = settings.size(Size::new(1200.0, 800.0));
    settings = settings.debug(false);
    settings
}

pub fn get_flags() -> Flags {
    let (config_handler, config) = (StarryConfig::config_handler(), StarryConfig::config());

    Flags {
        config_handler,
        config,
    }
}

pub fn set_image_cache() {
    // Init the image cache
    IMAGE_CACHE.get_or_init(|| std::sync::Mutex::new(ImageCache::new()));
}
