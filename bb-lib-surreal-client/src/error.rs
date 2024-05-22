pub type Error = anyhow::Error;

/// Error type for the SurrealDB Client
#[derive(thiserror::Error, Debug)]
pub enum SurrealClientError {
    #[error("No Record")]
    NoRecord,

    #[error("Update Failed")]
    UpdateFailed,

    #[error("No Id")]
    NoID,

    #[error("No Record w/ msg {0}")]
    NoRecordMsg(String),

    #[error("No Data stored in Record! {0}")]
    NoDataStored(String),

    #[error("Table name cannot be unset")]
    TableNameUnset,

    #[error("Surreal Error: {0}")]
    SurrealdbError(#[from] surrealdb::Error),

    #[error("DB Error: {0:?}")]
    SurrealErrorAPI(#[from] surrealdb::error::Api),

    #[error("Surreal Db Error: {0}")]
    SurrealErrorDB(#[from] surrealdb::error::Db),

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

// impl From<PoisonError<RwLockReadGuard<'_, DBClient>>> for Error {
//     fn from(e: PoisonError<RwLockReadGuard<'_, DBClient>>) -> Self {
//         Error::PoisonError(e.to_string())
//     }
// }
// `fn from(_: PoisonError<RwLockReadGuard<'_, DBClient>>) -> Self { todo!() }`:
// `fn from(_: PoisonError<RwLockReadGuard<'_, DBClient>>) -> Self { todo!() }
// `
