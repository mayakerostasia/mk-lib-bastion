// pub use db_request::Cmd;
// pub use db_request::DbRequest;
// pub use service::DbService;
// use surrealdb::sql::{Thing, Value};

mod error;
mod future;
mod service;

pub use service::DbService;

// mod prelude {
//     pub use crate::schemas::Record;
//     pub use crate::surreal_tower::db_request::Cmd;
//     pub use crate::surreal_tower::db_request::DbRequest;
//     pub use crate::surreal_tower::service::DbService;
//     // pub use crate::surreal_tower::db_functions::DbFunctions;
// }

#[cfg(test)]
mod tests {
    use crate::Record;
    use crate::{setup, DbService, Error, prelude::Id};
    use bytes::Bytes;
    use std::task::Context;
    use std::task::Poll;
    use std::{future::Future, pin::Pin};
    use tower::{BoxError, Service, ServiceExt};
    use serde_json::{Value, json};
    use bb_lib_nats_streams::{Frame, Encoder};

    #[derive(Clone)]
    struct MockService;

    impl Service<Record<Value>> for MockService {
        type Response = Record<Value>;
        type Error = BoxError;
        type Future =
            Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Record<Value>) -> Self::Future {
            let fut = async move {
                // Process the request and return a response
                println!("Mock-> Req -> {:#?}", req);
                Ok::<_, BoxError>(req)
            };
            Box::pin(fut)
        }
    }

    #[tokio::test]
    async fn test_db_service() -> Result<(), BoxError> {
        let cfg = setup();
        let mut db = DbService::new(cfg, MockService).await?;
        let _db = db.ready().await?;
        let saved = _db.call(Record::new(
            "test",
            Some(Id::rand()),
            Some(Box::new(json!({"hello":"world"}))),
            None,
        ));
        let okee = saved.await?;
        assert_eq!(Frame::Fin.encode()?, okee.encode()?);

        Ok(())
    }
}
