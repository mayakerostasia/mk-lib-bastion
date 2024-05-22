use crate::configuration::{read_config, Configuration};
use serde::{Deserialize, Serialize};

/// # Autotask Configuration
/// ! Prepend the configuration with the prefix `SRQL`
/// See [SurrealDB Documentation](https://docs.rs/surrealdb/latest/surrealdb/) for more information
///
/// ```text
/// SRQL_NS: Namespace  
/// SRQL_DB: Database  
/// SRQL_USER: Username  
/// SRQL_PASS: Password  
/// SRQL_AUTH_LEVEL: Authorization Level  
/// ```
#[allow(unused)]
#[derive(Debug, Deserialize, Serialize)]
pub struct SurrealCfg {
    pub path: String,
    pub ns: String,
    pub db: String,
    pub user: String,
    pub pass: String,
    pub auth_level: String,
}

impl Configuration for SurrealCfg {}

impl Default for SurrealCfg {
    fn default() -> Self {
        SurrealCfg {
            path: "path".to_string(),
            user: "root".to_string(),
            pass: "root".to_string(),
            auth_level: "root".to_string(),
            ns: "test".to_string(),
            db: "test".to_string(),
        }
    }
}

pub fn srql_config() -> Result<SurrealCfg, config::ConfigError> {
    read_config::<SurrealCfg>("srql", ",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srql_config() {
        let cfg = SurrealCfg::default();
        assert_eq!(cfg.path, "path");
        assert_eq!(cfg.ns, "test");
        assert_eq!(cfg.db, "test");
        assert_eq!(cfg.user, "root");
        assert_eq!(cfg.pass, "root");
        assert_eq!(cfg.auth_level, "root");
    }
}
