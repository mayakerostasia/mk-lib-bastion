// use std::marker::PhantomData;

// use bytes::Bytes;
// use tower::{Layer, Service};
// use serde::{Serialize, Deserialize};

// use crate::{DbConfig, DbService, Storable};

// pub struct DbServiceLayer<S, Request>
// where
//     S: Service<Request>,
//     Request: Clone + From<Bytes> + Into<Bytes>,
//     // Request: Into<Bytes> + Clone + From<Bytes> + Send + Sync + 'static,
//     // T: std::fmt::Debug + Storable<T> + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
// {
//     pub db_config: DbConfig,
//     _phantom_service: PhantomData<S>,
//     _phantom_request: PhantomData<Request>,
//     // _phantom_storable: PhantomData<T>,
// }

// impl<S, Request> DbServiceLayer<S, Request>
// where
//     S: Service<Request>,
//     Request: Clone + From<Bytes> + Into<Bytes>,
// // where
// //     S: Service<Request>,
// //     Request: Into<Bytes> + Clone + From<Bytes> + Send + Sync + 'static,
// //     T: std::fmt::Debug + Storable<T> + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
// {
//     pub fn new(db_config: DbConfig) -> Self {
//         Self {
//             db_config,
//             _phantom_service: PhantomData,
//             _phantom_request: PhantomData,
//             // _phantom_storable: PhantomData
//         }
//     }
// }

// impl<S, Request> Layer<S> for DbServiceLayer<S, Request>
// where
//     S: Service<Request>,
//     Request: Clone + From<Bytes> + Into<Bytes>,
// // where
// //     S: Service<Request>,
// //     Request: Into<Bytes> + Clone + From<Bytes> + Send + Sync + 'static,
// //     T: std::fmt::Debug + Storable<T> + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
// {
//     type Service = DbService<S, Request>;

//     fn layer(&self, inner: S) -> Self::Service {
//         // inner.layer(DbService)
//         DbService::new(&self.db_config,  inner)
//     }
// }
