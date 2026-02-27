use config::{Config, ConfigError, Environment, File, FileFormat};
use serde::de::DeserializeOwned;

/// Flexible configuration loader supporting multiple sources
pub struct ConfigLoader {
    prefix: String,
    separator: String,
    sources: Vec<ConfigSource>,
}

enum ConfigSource {
    Environment { prefix: String, separator: String },
    File { path: String, format: FileFormat },
}

impl ConfigLoader {
    /// Create a new config loader with the given environment variable prefix
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            separator: "__".to_string(),
            sources: vec![],
        }
    }

    /// Add environment variables as a config source
    pub fn from_env(mut self) -> Self {
        self.sources.push(ConfigSource::Environment {
            prefix: self.prefix.clone(),
            separator: self.separator.clone(),
        });
        self
    }

    /// Add a config file as a source
    pub fn from_file(mut self, path: &str, format: FileFormat) -> Self {
        self.sources.push(ConfigSource::File {
            path: path.to_string(),
            format,
        });
        self
    }

    /// Set a custom separator for environment variables (default: "__")
    pub fn separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }

    /// Load the configuration by combining all sources
    pub fn load<T: DeserializeOwned>(self) -> Result<T, ConfigError> {
        let mut builder = Config::builder();

        for source in self.sources {
            builder = match source {
                ConfigSource::Environment { prefix, separator } => builder.add_source(
                    Environment::default()
                        .prefix(&prefix)
                        .separator(&separator),
                ),
                ConfigSource::File { path, format } => builder.add_source(File::new(&path, format)),
            };
        }

        builder.build()?.try_deserialize()
    }
}

/// Backward-compatible helper function
pub fn read_config<T: DeserializeOwned>(
    prefix: &str,
    separator: &str,
) -> Result<T, ConfigError> {
    ConfigLoader::new(prefix)
        .separator(separator)
        .from_env()
        .load()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        value: String,
        number: i32,
    }

    #[test]
    fn test_config_loader_env() {
        std::env::set_var("TEST__VALUE", "hello");
        std::env::set_var("TEST__NUMBER", "42");

        let config: TestConfig = ConfigLoader::new("TEST")
            .from_env()
            .load()
            .unwrap();

        assert_eq!(config.value, "hello");
        assert_eq!(config.number, 42);

        std::env::remove_var("TEST__VALUE");
        std::env::remove_var("TEST__NUMBER");
    }

    #[test]
    fn test_read_config_backward_compat() {
        std::env::set_var("COMPAT__VALUE", "world");
        std::env::set_var("COMPAT__NUMBER", "99");

        let config: TestConfig = read_config("COMPAT", "__").unwrap();

        assert_eq!(config.value, "world");
        assert_eq!(config.number, 99);

        std::env::remove_var("COMPAT__VALUE");
        std::env::remove_var("COMPAT__NUMBER");
    }
}
