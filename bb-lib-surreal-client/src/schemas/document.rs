// #![cfg(feature = "serde_json")]

use crate::{SurrealId};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Document {
    pub id: SurrealId,
    pub meta: Option<Box<Value>>,
    pub data: Option<Box<Value>>,
}

impl Document {
    #[allow(unused)]
    pub fn new(table: &str, id: Option<&str>, value: &Value) -> Self {
        let _id = match id {
            Some(ident) => SurrealId::new(table, ident),
            None => SurrealId::random(table),
        };
        Document {
            id: _id,
            meta: Some(Box::new(json!({"created": SystemTime::now()}))),
            data: Some(Box::new(value.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn save_document_test() -> Result<(), Error> {
        let json = json!({"hello":"world"});
        let doc = Document::new("test", Some("test_doc"), &json);
        println!("{:#?}", doc);
        Ok(())
    }
}
