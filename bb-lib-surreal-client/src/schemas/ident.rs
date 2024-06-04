use std::fmt::Display;

use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SurrealId(pub Thing);

impl SurrealId {
    pub fn new(tb: &str, id: &str) -> Self {
        SurrealId(Thing::from((tb.to_string(), id.to_string())))
    }

    pub fn random(tb: &str) -> Self {
        SurrealId(Thing {
            tb: tb.to_string(),
            id: surrealdb::sql::Id::rand(),
        })
    }

    pub fn get_thing(&self) -> surrealdb::sql::Thing {
        self.0.clone()
    }

    pub fn get_id(&self) -> surrealdb::sql::Id {
        self.0.id.clone()
    }

    pub fn get_tb(&self) -> String {
        self.0.tb.clone()
    }
}

impl Display for SurrealId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
