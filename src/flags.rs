use cosmic::cosmic_config;

use crate::config::StarryConfig;

/// Flags given to our COSMIC application to use in it's "init" function.
#[derive(Clone, Debug)]
pub struct Flags {
    pub config_handler: Option<cosmic_config::Config>,
    pub config: StarryConfig,
}

pub fn flags() -> Flags {
    let (config_handler, config) = (StarryConfig::config_handler(), StarryConfig::config());

    Flags {
        config_handler,
        config,
    }
}
