// SPDX-License-Identifier: GPL-3.0-only
use cosmic::iced_core::image;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

pub(crate) static IMAGE_CACHE: OnceLock<Mutex<ImageCache>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImageCacheKey {
    name: &'static str,
}

pub struct ImageCache {
    cache: HashMap<ImageCacheKey, image::Handle>,
}

impl ImageCache {
    pub fn new() -> Self {
        let mut cache = HashMap::new();

        macro_rules! bundle {
            ($name:expr) => {
                let data: &'static [u8] =
                    include_bytes!(concat!("../../res/images/", $name, ".png"));
                cache.insert(
                    ImageCacheKey { name: $name },
                    image::Handle::from_memory(data.to_vec()),
                );
            };
        }

        bundle!("fallback");

        Self { cache }
    }

    fn get_image(&mut self, name: &'static str) -> image::Handle {
        self.cache
            .entry(ImageCacheKey { name })
            .or_insert_with(|| image::Handle::from_path(name))
            .clone()
    }

    pub fn get(name: &'static str) -> image::Handle {
        let mut image_cache = IMAGE_CACHE.get().unwrap().lock().unwrap();
        image_cache.get_image(name)
    }
}
