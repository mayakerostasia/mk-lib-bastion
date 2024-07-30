//! # SurrealDB Client for BlueBastion
//!
//! This crate is a client for the SurrealDB database.
//! If you use this crait to implement the [Storable] trait for your struct,
//!     you can store and retrieve your struct from the SurrealDB database.
//!
//! ## Example
//! // Define your object that you want to store in the database
//! // We recommend avoiding fields called "id" and "table" in your struct
//! // You can use serde::skip() to skip these fields in serialization
//! ```ignore
//! #[derive(Debug, Deserialize, Serialize, Clone)]
//! struct Person {
//!     name: String,
//!     age: u8,
//! }
//!
//! impl DBThings for Person {}
//!
//! impl Storable<Person> for Person {
//!     fn thing(&self) -> Thing {
//!         Thing::from((self.table().unwrap(), self.id().unwrap()))
//!     }
//!
//!     fn id(&self) -> Option<Id> {
//!         // We recommend using `Id::from([your_id])`
//!         // in your implementation
//!         Some( Id::from(1) )
//!     }
//!     
//!     fn table(&self) -> Option<String> {
//!         Some( "some_table".to_string() )
//!     }
//!     
//!     fn data(&self) -> Person {
//!         self.clone()
//!     }
//! }
//! ```

pub use config::{setup, DbConfig};
pub use error::Error;
pub use schemas::Document;
pub use schemas::Record;
pub use schemas::SurrealId;
pub use storable::Storable;

#[cfg(feature = "tower")]
pub use surreal_tower::DbService;
// use surrealdb::method::QueryStream;
// use surrealdb::sql::statements::LiveStatement;
// #[cfg(feature = "tower")]
// pub use surreal_tower::DbServiceLayer;

use core::panic;
use error::SurrealClientError;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use surrealdb::{
    engine::any::Any,
    opt::{auth::Root, PatchOp},
    sql::{Id, Thing},
    Notification, Response, Surreal,
};
use tracing::{debug, instrument, warn};

mod config;
mod creds;
mod error;
mod schemas;
mod storable;
#[cfg(feature = "tower")]
mod surreal_tower;

pub mod prelude {
    pub use surrealdb::sql::Id;
    pub use surrealdb::sql::Thing;
    pub use surrealdb::sql::Value;
    pub use surrealdb::Error as SDBError;
    pub use surrealdb::Notification;
    pub use surrealdb::Response;

    // pub use super::live::subscribe;
    // live_select
    // pub use super::live_select;
    pub use super::{
        connect,
        create_record,
        delete_record,
        // deserialize_id,
        get_record,
        query,
        update_record,
        Error,
        Record,
        Storable,
        SurrealId,
    };
}
static DB: Lazy<Surreal<Any>> = Lazy::new(Surreal::init);

/// Processing a Result<Option<T>> into a Result<T>
/// Process Response Option
/// PResPopTee
pub fn prespopt<T: Send>(rpo: Result<Option<T>, Error>) -> Result<T, Error> {
    match rpo {
        Ok(Some(rec)) => Ok(rec),
        Ok(None) => Err(SurrealClientError::NoRecord.into()),
        Err(e) => Err(SurrealClientError::NoRecordMsg(e.to_string()).into()),
    }
}

/// needs `connect` to be called first
///
///
/// # Examples
///
#[instrument]
pub async fn create_record<'a, T>(record: Record<T>) -> Result<Record<T>, Error>
where
    T: Debug + Serialize + DeserializeOwned + Sized + Clone,
    T: Send + Sync + 'static,
{
    // let _id = record.thing();
    let data = record.data();
    let id = match record.id() {
        Ok(_id) => _id,
        Err(e) => {
            if let SurrealClientError::NoID = e {
                warn!("No ID set, creating one");
                surrealdb::sql::Id::rand()
            } else {
                panic!("Fuck!");
            }
        }
    };
    let created: Option<Record<T>> = DB.create((record.tb(), id)).content(data).await?;

    match created {
        Some(record) => Ok(record),
        None => {
            Err(SurrealClientError::NoDataStored("No data stored in record!".to_string()).into())
        }
    }
}

/// Static function to update a record
/// This function requires you to call the `connect` function before calling
///
/// Uses the following query
/// `select * from tb:id;`
// #[instrument]
#[instrument(skip(record), fields(db_id = %record.id().unwrap_or(Id::from(666)), db_tb = &record.tb()))]
pub async fn update_record<'a, T>(record: Record<T>) -> Result<Record<Value>, Error>
where
    T: Debug + Serialize + DeserializeOwned + Sized + Clone,
    T: Send + Sync + 'static,
{
    let data = record.data();
    let updated: Option<Record<Value>> =
        DB.update((record.tb(), record.id()?)).content(data).await?;
    // let updated = None;

    match updated {
        Some(record) => Ok(record),
        None => Err(SurrealClientError::UpdateFailed.into()),
    }
}

pub async fn select<T: Send + Clone>(record: &mut Record<T>) -> Result<Record<Value>, Error> {
    let selected: Option<Record<Value>> = DB.select((record.tb(), record.id()?)).await?;
    // let bux = Box::new(selected);
    // record.set_data(bux);
    match selected {
        Some(rec) => Ok(rec),
        None => Err(SurrealClientError::NoRecord.into()),
    }
}
/// Static function to get a record
/// Returns only the ID as a Thing, does not return any Data or Meta (TODO)
/// This function requires you to call the `connect` function before calling
#[instrument(skip(record), fields(db_id = %record.id().unwrap_or(Id::from(666)), db_tb = &record.tb()))]
pub async fn get_record<T>(record: Record<T>) -> Result<Value, Error>
where
    T: Debug + Serialize + DeserializeOwned + Sized + Clone,
    T: Send + Sync + 'static,
{
    let q_str = format!(
        "select * from {};",
        Thing::from((record.tb(), record.id()?))
    );
    debug!(q_str = %q_str);
    let mut q = query(q_str.as_str()).await?;
    let ret: Option<Value> = q.take(0)?;
    match ret {
        Some(val) => Ok(val),
        None => Err(SurrealClientError::NoRecord.into()),
    }
}

/// Static function to delete a record
/// This function requires you to call the `connect` function before calling
// #[instrument]
#[instrument(skip(record), fields(db_id = %record.id().unwrap_or(Id::from(666)), db_tb = &record.tb()))]
pub async fn delete_record<T>(record: Record<T>) -> Result<Option<T>, Error>
where
    T: Debug + Serialize + DeserializeOwned + Sized + Clone,
    T: Send + Sync + 'static,
{
    let table = record.tb();

    if table == "_" {
        return Err(SurrealClientError::TableNameUnset.into());
    }

    let id = record.id()?;

    Ok(DB.delete((table, id)).await?)
}

/// Static function to update a record
/// This function is used automatically in the `Storable` trait
// #[instrument]
#[instrument(skip(record), fields(db_id = %record.id().unwrap_or(Id::from(666)), db_tb = &record.tb()))]
pub async fn patch_record<T>(record: Record<T>, patch: PatchOp) -> Result<Option<T>, Error>
where
    T: Debug + Serialize + DeserializeOwned + Sized + Clone,
    T: Send + Sync + 'static,
{
    let table = record.tb();
    if table == "_" {
        return Err(SurrealClientError::TableNameUnset.into());
    }

    let id = record.id()?;

    let ret = DB.update((table, id.clone())).patch(patch).await?;
    // .map_err(|_e| Error::UpdateFailed {
    //     id: id.to_string(),
    //     id_raw: id.to_raw(),
    //     table,
    // })?;

    Ok(ret)
}

/// Static function to query the database
/// This function takes in a SurrealQL query string
/// e.g. `SELECT * FROM table;`
/// This function requires you to call the `connect` function before calling
// #[instrument]
#[instrument]
pub async fn query(query: &str) -> Result<Response, Error> {
    let results: Response = DB.query(query).await?;
    Ok(results)
}

/// Static function to connect to the database
/// This function is used automatically in the `Storable` trait
pub async fn connect(config: &config::DbConfig) -> Result<DbGuard, Error> {
    DB.connect(&config.path).await?;
    let _result = DB
        .signin(Root {
            username: &config.user,
            password: &config.pass,
        })
        .await?;

    DB.use_ns(&config.ns).use_db(&config.db).await?;
    Ok(DbGuard)
}

pub async fn relate(edge_table: &str, from: Thing, to: Thing) -> Result<Response, Error> {
    let relate_query = format!("RELATE {} ->{} -> {};", from, edge_table, to);
    let resp = query(relate_query.as_str()).await?;
    Ok(resp)
}
/// Static function to start a live select stream
/// This function requires you to call the `connect` function before calling
/// Unimplemented
pub async fn live_select<'a, T>(
    table: &str,
    id: &str,
) -> Result<surrealdb::method::Stream<'a, Any, Option<T>>, Error>
where
    T: Debug + Serialize + DeserializeOwned + Sized + Clone,
    T: Send + Sync + 'static,
{
    let stream = DB.select((table, id)).live().await?;
    Ok(stream)
}

pub async fn live_query<T>(
    query: &str,
) -> Result<surrealdb::method::QueryStream<Notification<T>>, Error>
where
    T: Debug + Serialize + DeserializeOwned + Sized + Clone + Unpin,
    T: Send + Sync + 'static,
{
    let mut resp = DB.query(query).await?;
    resp.stream(0).map_err(|e| e.into())
}

// DBGuard Implementation
/// Currently Unimplemented
#[derive(Debug, Clone)]
pub struct DbGuard;

impl Drop for DbGuard {
    fn drop(&mut self) {
        let _closed = DB.invalidate();
    }
}

pub async fn close() -> Result<(), Error> {
    DB.invalidate().await?;
    Ok(())
}
