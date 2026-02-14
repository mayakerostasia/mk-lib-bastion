use crate::configuration::{read_config, Configuration};
use serde::{Deserialize, Serialize};

///
/// # Swimlane Configuration
/// Environment Variables:
/// - JIRA_URL
/// - JIRA_USER
/// - JIRA_API_KEY
///
#[allow(unused)]
#[derive(Debug, Deserialize, Serialize)]
pub struct JiraCfg<'a> {
    pub url: &'a str,
    pub user: &'a str,
    pub api_key: &'a str,
}

impl Configuration for JiraCfg<'_> {}

impl Default for JiraCfg<'_> {
    fn default() -> Self {
        JiraCfg {
            url: "https://jira.com",
            user: "user",
            api_key: "api_key",
        }
    }
}

/// # Autotask Configuration
/// ! Prepend the configuration with the prefix `JIRA`
///
/// ## Example
///
#[allow(unused)]
pub fn jira_config() -> Result<JiraCfg<'static>, config::ConfigError> {
    read_config::<JiraCfg>("jira", ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_at_config() {
        let cfg = JiraCfg::default();
        assert_eq!(cfg.url, "https://jira.com");
        assert_eq!(cfg.user, "user");
        assert_eq!(cfg.api_key, "api_key");
    }
}
