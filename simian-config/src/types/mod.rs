mod surreal;
pub use surreal::{SurrealCfg, srql_config};

// Re-export other config types from old location for compatibility
#[cfg(feature = "autotask")]
pub use crate::configs::autotask::{AutotaskCfg, autotask_config};

#[cfg(feature = "jira")]
pub use crate::configs::jira::{JiraCfg, jira_config};

#[cfg(feature = "swimlane")]
pub use crate::configs::swimlane::{SwimlaneCfg, swimlane_config};
