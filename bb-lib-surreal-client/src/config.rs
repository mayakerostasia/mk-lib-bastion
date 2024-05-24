use bb_lib_config::{srql_config, SurrealCfg};
// use std::env;

#[derive(Clone)]
pub struct DbConfig {
    pub path: String,
    pub ns: String,
    pub db: String,
    pub user: String,
    pub pass: String,
}

impl std::fmt::Debug for DbConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbConfig")
            .field("path", &self.path)
            .field("ns", &self.ns)
            .field("db", &self.db)
            .field("user", &self.user)
            .finish()
    }
}

pub fn setup() -> DbConfig {
    let cfg: SurrealCfg = srql_config().expect("Failed to setup surreal from ENV");
    DbConfig {
        path: cfg.path,
        ns: cfg.ns,
        db: cfg.db,
        user: cfg.user,
        pass: cfg.pass,
    }
}
