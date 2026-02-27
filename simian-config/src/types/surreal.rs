use crate::loader::{read_config, ConfigLoader};
use crate::configuration::Configuration;
use config::{ConfigError, FileFormat};
use serde::{Deserialize, Serialize};

/// SurrealDB Configuration
///
/// Environment variables (with SRQL prefix and __ separator):
/// - SRQL__PATH: Database connection path (e.g., "ws://localhost:8000", "memory")
/// - SRQL__NS: Namespace
/// - SRQL__DB: Database name
/// - SRQL__USER: Username
/// - SRQL__PASS: Password
/// - SRQL__AUTH_LEVEL: Authorization level (root, ns, db)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        Self {
            path: "memory".to_string(),
            ns: "test".to_string(),
            db: "test".to_string(),
            user: "root".to_string(),
            pass: "root".to_string(),
            auth_level: "root".to_string(),
        }
    }
}

impl SurrealCfg {
    /// Load from environment variables with SRQL prefix
    pub fn from_env() -> Result<Self, ConfigError> {
        ConfigLoader::new("SRQL").from_env().load()
    }

    /// Load from JSON file
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        ConfigLoader::new("SRQL")
            .from_file(path, FileFormat::Json)
            .load()
    }

    /// Load from environment with fallback to defaults
    pub fn load() -> Self {
        Self::from_env().unwrap_or_default()
    }
}

/// Backward-compatible function for existing code
pub fn srql_config() -> Result<SurrealCfg, ConfigError> {
    read_config::<SurrealCfg>("SRQL", "__")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = SurrealCfg::default();
        assert_eq!(cfg.path, "memory");
        assert_eq!(cfg.ns, "test");
        assert_eq!(cfg.db, "test");
        assert_eq!(cfg.user, "root");
        assert_eq!(cfg.pass, "root");
        assert_eq!(cfg.auth_level, "root");
    }

    #[test]
    fn test_from_env() {
        std::env::set_var("SRQL__PATH", "ws://localhost:8000");
        std::env::set_var("SRQL__NS", "production");
        std::env::set_var("SRQL__DB", "maindb");
        std::env::set_var("SRQL__USER", "admin");
        std::env::set_var("SRQL__PASS", "secret");
        std::env::set_var("SRQL__AUTH_LEVEL", "db");

        let cfg = SurrealCfg::from_env().unwrap();
        assert_eq!(cfg.path, "ws://localhost:8000");
        assert_eq!(cfg.ns, "production");
        assert_eq!(cfg.db, "maindb");

        std::env::remove_var("SRQL__PATH");
        std::env::remove_var("SRQL__NS");
        std::env::remove_var("SRQL__DB");
        std::env::remove_var("SRQL__USER");
        std::env::remove_var("SRQL__PASS");
        std::env::remove_var("SRQL__AUTH_LEVEL");
    }

    #[test]
    fn test_load_with_fallback() {
        // Clean up any leftover env vars
        std::env::remove_var("SRQL__PATH");
        std::env::remove_var("SRQL__NS");
        std::env::remove_var("SRQL__DB");
        std::env::remove_var("SRQL__USER");
        std::env::remove_var("SRQL__PASS");
        std::env::remove_var("SRQL__AUTH_LEVEL");
        
        // Should not panic, returns default
        let cfg = SurrealCfg::load();
        assert_eq!(cfg.path, "memory");
    }
}
