use config::{Config, ConfigError, Environment};
use serde::de::Deserialize;

pub trait Configuration {}

#[allow(unused)]
pub fn read_config<'de, T: Configuration + Deserialize<'de>>(
    prefix: &str,
    list_separator: &str,
) -> Result<T, ConfigError> {
    let config = Config::builder()
        .add_source(
            Environment::default()
                .prefix(prefix)
                .list_separator(list_separator),
        )
        .build()
        .expect("Failed to Initialize Configuration Builder");

    let config: Result<T, _> = config.try_deserialize();

    config
}
