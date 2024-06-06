// pub use db_request::Cmd;
// pub use db_request::DbRequest;
// pub use service::DbService;
// use surrealdb::sql::{Thing, Value};

mod error;
mod future;
mod service;

// mod prelude {
//     pub use crate::schemas::Record;
//     pub use crate::surreal_tower::db_request::Cmd;
//     pub use crate::surreal_tower::db_request::DbRequest;
//     pub use crate::surreal_tower::service::DbService;
//     // pub use crate::surreal_tower::db_functions::DbFunctions;
// }

#[cfg(test)]
mod tests {
    // use super::prelude::*;
    use crate::Error;

    #[tokio::test]
    async fn test_get_database() -> Result<(), Error> {
        // TODO: Remake me
        // let db = DbService::new().await?;
        // let mut db_serv = db.ready().await?;

        // db_serv.call(DbRequest::Query("SELECT name,dns FROM fn::get_gc_info();"));
        Ok(())
    }
}
