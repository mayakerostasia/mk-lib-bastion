use crate::configuration::{read_config, Configuration};
use serde::{Deserialize, Serialize};

///
/// # Swimlane Configuration
/// Environment Variables:
/// - SL_HOST
/// - SL_PAT
/// - SL_USER
/// - SL_PASSConfiguration
///
#[allow(unused)]
#[derive(Debug, Deserialize, Serialize)]
pub struct SwimlaneCfg {
    pub host: String,
    pub pat: String,
    pub user: String,
    pub pass: String,
}

impl Configuration for SwimlaneCfg {}

impl Default for SwimlaneCfg {
    fn default() -> Self {
        SwimlaneCfg {
            host: "https://swimlane.com".to_string(),
            pat: "pat".to_string(),
            user: "user".to_string(),
            pass: "pass".to_string(),
        }
    }
}

/// # Autotask Configuration
/// ! Prepend the configuration with the prefix `SL`
///
/// ## Example
///
#[allow(unused)]
pub fn swimlane_config() -> Result<SwimlaneCfg, config::ConfigError> {
    read_config::<SwimlaneCfg>("sl", ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_config() {
        let cfg = SwimlaneCfg::default();
        assert_eq!(cfg.host, "https://swimlane.com");
        assert_eq!(cfg.pat, "pat");
        assert_eq!(cfg.user, "user");
        assert_eq!(cfg.pass, "pass");
    }
}
