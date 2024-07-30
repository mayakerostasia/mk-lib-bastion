// use nico_surreal_client::prelude::*;
use bb_lib_surreal_client::{Error, Record, Storable};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Id;

const TEST_TABLE: &str = "test_table";
const TEST_PERSON: &str = "test_person";

// Definition
#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Person {
    name: String,
    age: u8,
}

// API Call or Factory
fn person_factory(table: &str, id: Id, name: &str, age: u8) -> Option<Record<Person>> {
    let person = Person {
        name: name.to_string(),
        age,
    };
    Some(Record::new(table, Some(id), Some(Box::new(person)), None))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Record To Database
    let john = person_factory(TEST_TABLE, Id::from(TEST_PERSON), "John", 32).unwrap();
    println!("Record John: {:?}", &john);

    // let _ = john.delete().await?;
    let saved_john = john.save().await?;
    let selected_john = john.select().await?;
    // let john: Record<Person> = selected_john.take().expect("Couldn't select john");
    // let updated_john = john.update().await.await?;
    // let deleted_john = john.delete().await.await?;

    // Some Logging
    println!("Created -> Yes");
    println!(
        "SavedJohn : ({}:{}) -> {:?}",
        TEST_TABLE, TEST_PERSON, saved_john
    );
    println!(
        "SelectedJohn : ({}:{}) -> {:?}",
        TEST_TABLE, TEST_PERSON, selected_john
    );
    // println!{
    //     "UpdatedJohn : ({}:{}) -> {:?}",
    //     TEST_TABLE, TEST_PERSON, updated_john
    // };
    // println!(
    //     "DeletedJohn : ({}:{}) -> {:?}",
    //     TEST_TABLE, TEST_PERSON, deleted_john
    // );

    // We Succeeded so Ret 0
    Ok(())
}
