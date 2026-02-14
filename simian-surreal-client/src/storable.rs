use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::DbConfig;
use crate::Error;
use crate::{create_record, delete_record, select, update_record};
use crate::{setup, Record};

// pub trait DBThings: Debug + Serialize + DeserializeOwned + Sized + Clone {}

lazy_static! {
    static ref CFG: DbConfig = setup();
}

/// A Master trait to define an object that can be stored in the database.
/// The object must implement the DBThings trait.
/// The functions that are required to be implemented are:
/// - id: Should return an Option<surrealdb::Id>
///        we do recommend using `Id::from([your_id])` from the surrealdb crate. in your implementation
///        in order to create a random Id we recommend using `Id::rand()` from the surrealdb crate.
/// - table: Should return an Option<String>
/// - thing: Should return a surrealdb::Thing with the table and id from the above two functions.
/// - data: Should return the data that you want to store in the database.
///
/// Example:
/// ```ignore
/// impl Storable<Person> for Person {
///     fn thing(&self) -> Thing {
///         Thing::from((self.table().unwrap(), self.id().unwrap()))
///     }
///     fn id(&self) -> Option<Id> {
///         Some(Id::Number(1))
///     }
///     
///     fn table(&self) -> Option<String> {
///         Some(TEST_TABLE.to_string())
///     }
///     
///     fn data(&self) -> Person {
///         self.clone()
///     }
/// }
/// ```
#[async_trait]
pub trait Storable<T>
where
    T: Debug + Serialize + Clone + for<'a> Deserialize<'a>,
    T: Send + Sync + 'static,
    Record<T>: Clone + From<T> + for<'a> From<&'a Self> + for<'a> From<&'a mut Self>,
{
    async fn save(&self) -> Result<Record<T>, Error> {
        create_record(self.into()).await
    }

    async fn select(&mut self) -> Result<Record<T>, Error> {
        let mut record: Record<T> = self.into();
        Ok(select(&mut record).await?)
    }

    async fn update(&self) -> Result<Record<T>, Error> {
        let record: Record<T> = self.into();
        let updated = update_record(record).await?;
        Ok(updated)
    }

    async fn delete(&self) -> Result<Option<Record<T>>, Error> {
        let record: Record<T> = self.into();
        Ok(delete_record(record).await?)
    }
}
