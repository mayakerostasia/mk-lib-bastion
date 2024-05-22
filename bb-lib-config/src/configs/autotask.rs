use crate::configuration::{read_config, Configuration};
use serde::{Deserialize, Serialize};

///
/// # Autotask Configuration
///
/// ENV vars:
/// - AT_URL
/// - AT_INTEGRATION_CODE
/// - AT_USERNAME
/// - AT_PASSWORD
#[allow(unused)]
#[derive(Debug, Deserialize, Serialize)]
pub struct AutotaskCfg<'a> {
    pub url: &'a str,
    pub integration_code: &'a str,
    pub username: &'a str,
    pub password: &'a str,
}

impl Configuration for AutotaskCfg<'_> {}

impl Default for AutotaskCfg<'_> {
    fn default() -> Self {
        AutotaskCfg {
            url: "https://autotask.net",
            integration_code: "integration_code",
            username: "username",
            password: "password",
        }
    }
}

/// # Autotask Configuration
/// ! Prepend the configuration with the prefix `AT`
///
/// ## Example
///
#[allow(unused)]
pub fn autotask_config() -> Result<AutotaskCfg<'static>, config::ConfigError> {
    read_config::<AutotaskCfg>("at", ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_config() {
        let cfg = AutotaskCfg::default();
        assert_eq!(cfg.url, "https://autotask.net");
        assert_eq!(cfg.integration_code, "integration_code");
        assert_eq!(cfg.username, "username");
        assert_eq!(cfg.password, "password");
    }
}
