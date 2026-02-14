#![doc(
    html_root_url = "https://glowing-adventure-8p43ek1.pages.github.io/DEV-simian-Config/index.html"
)]
//! # Simian-Config
//! A simple configuration library for Rust applications.
//!
//! ## Usage
//!
//! You must select a feature to use the library. The available features are:
//! - `dev`: [configuration::Configuration]
//! - `surreal`: [SurrealCfg]
//! - `autotask`: [AutotaskCfg]
//! - `jira`: [JiraCfg]
//! - `swimlane`: [SwimlaneCfg]
//!
//!
//! ## Example
//!
//! ```ignore
//! use simian_config::configuration::{ read_config, Configuration };
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct MyConfig {
//!    key: String,
//!    value: String,
//! }
//!
//! impl Configuration for MyConfig {}
//!
//! impl Default for MyConfig {
//!    fn default() -> Self {
//!        MyConfig {
//!            key: "key".to_string(),
//!            value: "value".to_string(),
//!        }
//!    }
//! }
//!
//! fn my_config() -> anyhow::Result<MyConfig> {
//!    let cfg = read_config::<MyConfig>("my", "config").map_err(|e| anyhow::Error::from(e))?;
//!    Ok(cfg)
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!    let cfg = my_config()?;
//!    println!("{:?}", cfg);
//!    Ok(())
//! }
//! ```
//!
//! *This project is inspired by philosphical debate*
//!
//! consider use: pub trait Source: Debug

mod configs;

#[cfg(not(feature = "dev"))]
pub mod configuration;

#[cfg(feature = "dev")]
pub mod configuration;

#[cfg(feature = "surreal")]
pub use configs::surreal::{srql_config, SurrealCfg};

#[cfg(feature = "autotask")]
pub use configs::autotask::{autotask_config, AutotaskCfg};

#[cfg(feature = "jira")]
pub use configs::jira::{jira_config, JiraCfg};

#[cfg(feature = "swimlane")]
pub use configs::swimlane::{swimlane_config, SwimlaneCfg};

// Add cfg generator here
