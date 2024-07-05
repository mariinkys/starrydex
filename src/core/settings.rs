use std::sync::Mutex;

use cosmic::{
    app::Settings,
    iced::{Limits, Size},
};

use super::{
    icon_cache::{IconCache, ICON_CACHE},
    image_cache::{ImageCache, IMAGE_CACHE},
};

pub fn init() -> Settings {
    set_image_cache();
    set_icon_cache();

    let settings = get_app_settings();
    settings
}

pub fn get_app_settings() -> Settings {
    let mut settings = Settings::default();

    settings = settings.size_limits(Limits::NONE.min_width(800.0).min_height(300.0));
    settings = settings.size(Size::new(1200.0, 800.0));
    settings = settings.debug(false);
    settings
}

pub fn set_image_cache() {
    IMAGE_CACHE.get_or_init(|| Mutex::new(ImageCache::new()));
}

pub fn set_icon_cache() {
    ICON_CACHE.get_or_init(|| Mutex::new(IconCache::new()));
}
