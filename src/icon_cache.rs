// SPDX-License-Identifier: GPL-3.0-only

use cosmic::widget::icon;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(crate) static ICON_CACHE: OnceLock<Mutex<IconCache>> = OnceLock::new();

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct IconCacheKey {
    name: Cow<'static, str>,
    size: u16,
}

pub struct IconCache {
    cache: HashMap<IconCacheKey, icon::Handle>,
}

impl IconCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        macro_rules! bundle {
            ($name:expr, $size:expr) => {
                let data: &'static [u8] =
                    include_bytes!(concat!("../res/icons/bundled/", $name, ".svg"));
                cache.insert(
                    IconCacheKey {
                        name: Cow::Borrowed($name),
                        size: $size,
                    },
                    icon::from_svg_bytes(data).symbolic(true),
                );
            };
        }
        bundle!("edit-clear-all-symbolic", 18);
        bundle!("filter-symbolic", 18);
        bundle!("go-next-symbolic", 18);
        bundle!("go-previous-symbolic", 18);
        bundle!("type-bug", 18);
        bundle!("type-dark", 18);
        bundle!("type-dragon", 18);
        bundle!("type-electric", 18);
        bundle!("type-fairy", 18);
        bundle!("type-fighting", 18);
        bundle!("type-fire", 18);
        bundle!("type-flying", 18);
        bundle!("type-ghost", 18);
        bundle!("type-grass", 18);
        bundle!("type-ground", 18);
        bundle!("type-ice", 18);
        bundle!("type-normal", 18);
        bundle!("type-poison", 18);
        bundle!("type-psychic", 18);
        bundle!("type-rock", 18);
        bundle!("type-steel", 18);
        bundle!("type-water", 18);
        Self { cache }
    }

    fn get_icon(&mut self, name: &'static str, size: u16) -> icon::Icon {
        let handle = self
            .cache
            .entry(IconCacheKey {
                name: Cow::Borrowed(name),
                size,
            })
            .or_insert_with(|| icon::from_name(name).size(size).handle())
            .clone();
        icon::icon(handle).size(size)
    }
}

#[allow(dead_code)]
pub fn get_icon(name: &'static str, size: u16) -> icon::Icon {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache.get_icon(name, size)
}

pub fn get_handle(name: &'static str, size: u16) -> icon::Handle {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache
        .cache
        .entry(IconCacheKey {
            name: Cow::Borrowed(name),
            size,
        })
        .or_insert_with(|| icon::from_name(name).size(size).handle())
        .clone()
}

pub fn get_handle_owned(name: String, size: u16) -> icon::Handle {
    let mut icon_cache = ICON_CACHE.get().unwrap().lock().unwrap();
    icon_cache
        .cache
        .entry(IconCacheKey {
            name: Cow::Owned(name.clone()),
            size,
        })
        .or_insert_with(|| icon::from_name(name).size(size).handle())
        .clone()
}
