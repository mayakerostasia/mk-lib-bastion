use bb_lib_surreal_client::{Error, Record};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use surrealdb::{RecordId, RecordIdKey};
use tracing::debug;

const TEST_TABLE: &str = "test_table";
const TEST_PERSON: &str = "test_person";

// Definition
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Person {
    _id: RecordId,
    name: String,
    age: u8,
    #[serde(flatten)]
    _extra: HashMap<String, Value>,
}

impl From<Person> for Record<Person> {
    fn from(value: Person) -> Self {
        Record::new(
            TEST_TABLE,
            Some(RecordIdKey::from(TEST_PERSON)),
            Some(value.clone()),
            // None,
        )
    }
}

// API Call or Factory
impl Person {}
fn person_factory(table: &str, id: &str, name: &str, age: u8) -> Person {
    Person {
        _id: RecordId::from_table_key(table, id),
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
    let mut john = john.data();
    // Modify data
    john.age = 33;
    debug!("Person Age Updated Locally -> {:#?}", &john.age);

    // Return to Record
    let mut rec: Record<Person> = john.into();

    // Send update
    let resp_update = rec.update().await?;
    debug!("Person Update is  -> {:#?}", resp_update);

    Ok(())
}
