use bb_lib_surreal_client::{connect, query, setup, Error, Record, Storable};
use serde::{Deserialize, Serialize};
use surrealdb::RecordIdKey;

const TEST_TABLE: &str = "test_table";
const TEST_PERSON: &str = "test_person";

// Definition
// We're going to pull the table name from _tb and the id from _id
#[derive(Debug, Deserialize, Serialize, Clone)]
struct Person {
    _tb: String,
    _id: String,
    name: String,
    age: u8,
}

// This is where you'll be creating the ID that Surreal will source
// The table MUST be present but you may use `None` as the id and Surreal
// will create a random ID
impl From<Person> for Record<Person> {
    fn from(value: Person) -> Record<Person> {
        Record::new(
            value._tb.clone().as_str(),
            Some(RecordIdKey::from(value._id.clone())),
            Some(value),
        )
    }
}

// These are necessary because we've got a borrow to Person, which is required
// for some of the Storable<Person> calls
// since the trait functions operate on a borrow we need to tell the compiler
// how to move from a &Person into a Record<Person>
impl<'a> From<&'a Person> for Record<Person> {
    fn from(value: &'a Person) -> Record<Person> {
        value.clone().into()
    }
}

// Exactly the same as above just fur a mutable borrow
impl<'a> From<&'a mut Person> for Record<Person> {
    fn from(value: &'a mut Person) -> Record<Person> {
        value.clone().into()
    }
}

// Implement the Storable trait
impl Storable<Person> for Person {}

// API Call or Factory
fn person_factory(table: &str, id: &str, name: &str, age: u8) -> Option<Person> {
    eprintln!("Creating person");
    let person = Person {
        _tb: table.to_string(),
        _id: id.to_string(),
        name: name.to_string(),
        age,
    };
    eprintln!("Person Created");
    Some(person)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cfg = setup();
    let _db_guard = connect(&cfg).await;
    // Clean the Test Table
    query(format!("DELETE {}", TEST_TABLE).as_str()).await?;

    // Initalize the Record
    // Record To Database
    let john = person_factory(TEST_TABLE, TEST_PERSON, "John", 32).unwrap();
    println!("Record John: {:?}", &john);

    // Inital Save of the record
    let mut saved_john: Record<Person> = john.save().await?;
    println!("Created -> Yes");
    println!(
        "SavedJohn : ({}:{}) -> {:?}",
        TEST_TABLE, TEST_PERSON, saved_john
    );

    // Attempt to Select the record
    let selected_john = saved_john.select().await?;
    println!(
        "SelectedJohn : ({}:{}) -> {:?}",
        TEST_TABLE, TEST_PERSON, selected_john
    );

    // Update the record
    let mut new_john = selected_john.data();
    new_john.age = 33;
    let updated_john = new_john.update().await?;
    println! {
        "UpdatedJohn : ({}:{}) -> {:?}",
        TEST_TABLE, TEST_PERSON, updated_john
    };

    // Delete the Record
    let deleted_john = updated_john.delete().await?;
    println!(
        "DeletedJohn : ({}:{}) -> {:?}",
        TEST_TABLE, TEST_PERSON, deleted_john
    );

    // We Succeeded so Ret 0
    Ok(())
}
