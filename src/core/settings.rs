use std::sync::Mutex;

use crate::app::Flags;
use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};

use super::{
    config::StarryDexConfig,
    icon_cache::{IconCache, ICON_CACHE},
    image_cache::{ImageCache, IMAGE_CACHE},
};

pub fn init() -> (Settings, Flags) {
    set_logger();
    set_image_cache();
    set_icon_cache();

    let settings = get_app_settings();
    let flags = get_flags();

    (settings, flags)
}

pub fn get_app_settings() -> Settings {
    let mut settings = Settings::default();

    settings = settings.size_limits(Limits::NONE.min_width(800.0).min_height(300.0));
    settings = settings.size(Size::new(1200.0, 800.0));
    settings = settings.debug(false);
    settings
}

pub fn set_logger() {
    tracing_subscriber::fmt().json().init();
}

pub fn set_image_cache() {
    IMAGE_CACHE.get_or_init(|| Mutex::new(ImageCache::new()));
}

pub fn set_icon_cache() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}

pub fn get_flags() -> Flags {
    let (config_handler, config) = (StarryDexConfig::config_handler(), StarryDexConfig::config());

    let flags = Flags {
        config_handler,
        config,
    };
    flags
}
