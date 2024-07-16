use std::collections::HashMap;

use bb_lib_surreal_client::{
    prelude::Id,
    query, Error, Record, Storable, SurrealId,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use tracing::{debug, error, info};

const TEST_TABLE: &str = "test_table";
const TEST_PERSON: &str = "test_person";

// Definition
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Person {
    #[serde(skip_serializing)]
    _id: Option<Id>,
    #[serde(skip_serializing)]
    _tb: Option<String>,
    name: String,
    age: u8,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

impl Into<Record<Person>> for Person {
    fn into(self) -> Record<Person> {
        Record::new(
            self._tb.clone().unwrap().as_str(),
            Some(self._id.clone().unwrap()),
            Some(Box::new(self)),
            None,
        )
    }
}

// impl DBThings for Person {}

// API Call or Factory
impl Person {}
fn person_factory(table: &str, id: &str, name: &str, age: u8) -> Person {
    Person {
        _id: Some(Id::from(id)),
        _tb: Some(table.to_string()),
        name: name.to_string(),
        age,
        _extra: HashMap::new(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _otel = bb_lib_tracing::initialize()?;

    // Build the record - This has nothing to do with the database atm
    // This is simply used to setup the Record<Person> that will be submitted to the database
    let john: Record<Person> = person_factory(TEST_TABLE, TEST_PERSON, "John", 32).into();
    debug!("Record John: {:?}", &john);

    // Pull out the data
    let mut john = *john.data();
    // Modify data
    john.age = 33;
    debug!("Person Age Updated Locally -> {:#?}", &john.age);

    // Return to Record
    let rec: Record<Person> = john.into();

    // Send update
    let resp_update = rec.update().await?;
    debug!("Person Update is  -> {:#?}", resp_update);

    Ok(())
}
