use crate::{connect, DbConfig, DbGuard, Error, Record};
use anyhow::anyhow;
use bb_lib_nats_streams::Frame;
// use bytes::Bytes;
use futures::executor;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::{future::Future, marker::PhantomData, pin::Pin, task::Poll};
use tower::{BoxError, Service};
use tracing::debug;

pin_project! {
    #[derive(Clone)]
    pub struct DbService<T>
    // where
    //     T: Debug + Serialize + DeserializeOwned + Sized + Clone,
    //     T: Send + Sync + 'static,
    {
        #[pin]
        db_guard: Option<DbGuard>,
        db_cfg: DbConfig,
        _phantom_request: PhantomData<T>,
    }
}
impl<T> Display for DbService<T>
// where
//     S: Service<Request>,
//     Request: Clone + From<Bytes> + Into<Bytes>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.db_guard {
            Some(_) => f.write_str("SurrealDBService::Connected"),
            None => f.write_str("SurrealDBService::Disconnected"),
        }
    }
}

impl<T> std::fmt::Debug for DbService<T>
// where
//     S: Service<Request>,
//     Request: Clone + From<Bytes> + Into<Bytes>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.db_guard {
            Some(_) => f.write_str("SurrealDBService::Connected"),
            None => f.write_str("SurrealDBService::Disconnected"),
        }
    }
}

#[allow(unused)]
impl<T> DbService<T>
// where
//     S: Service<Request>,
//     Request: Clone + From<Bytes> + Into<Bytes>,
{
    pub fn new(cfg: &DbConfig) -> Self {
        DbService {
            db_guard: None,
            db_cfg: cfg.clone(),
            _phantom_request: PhantomData::<T>,
        }
    }
}

impl<T> Service<Record<T>> for DbService<T>
where
    // S: Service<Request>,
    // Request: Into<Frame> + From<Frame> + Into<Bytes>,
    T: Debug + Serialize + Clone + for<'a> Deserialize<'a>,
    T: Send + Sync + 'static,
    Record<T>: From<T> + for<'a> From<&'a Self>,
    // T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + Storable<T> + Clone,
    // T: Send + Sync + 'static,
    // T: From<Bytes> + Into<Record<T>>,
    // Record<T>: Send + Sync,
{
    type Response = Frame;
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync + 'static>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let mut guard_future = Box::pin(connect(&self.db_cfg));
        while self.db_guard.is_none() {
            match guard_future.as_mut().poll(cx) {
                Poll::Pending => {}
                Poll::Ready(guard) => {
                    debug!("Db Ready");
                    match guard {
                        Ok(gua) => {
                            let _ = self.db_guard.insert(gua);
                            return Poll::Ready(Ok(()));
                        }
                        Err(e) => return Poll::Ready(Err(e.into())),
                    }
                }
            }
        }
        if self.db_guard.is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(Error::msg(anyhow!("Couldn't connect to DB")).into()))
        }
    }

    fn call(&mut self, mut request: Record<T>) -> Self::Future {
        Box::pin(async move {
            let okee = request.update();
            let resp = executor::block_on(okee);
            debug!(record_debug = ?resp, "Record Updated");
            Ok::<_, BoxError>(Frame::Fin)
        })
    }
}

// impl<S, T> Service<Record<T>> for DbService<S, Record<T>>
// where
//     S: Service<Record<T>> + Clone,
//     T: /* Into<Bytes> */ Clone
//         + Serialize
//         + for<'de> Deserialize<'de>
//         + std::fmt::Debug
//         + Send
//         + Sync
//         + 'static,
//     Record<T>: Storable<T>,
// {
//     type Response = Frame;
//     type Error = BoxError;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//
//         let mut guard_future = Box::pin(connect(&self.db_cfg));
//         while self.db_guard.is_none() {
//             match guard_future.as_mut().poll(cx) {
//                 Poll::Pending => {},
//                 Poll::Ready(guard) => {
//                     debug!("Db Ready");
//                     match guard {
//                         Ok(gua) => {
//                             self.db_guard.insert(gua);
//                             return Poll::Ready(Ok(()))
//                         },
//                         Err(e) => return Poll::Ready(Err(e.into()))
//                     }
//                     // let _ = self.db_guard.insert(guard)
//                 }
//             }
//         };
//         if self.db_guard.is_some() {
//             Poll::Ready(Ok(()))
//         } else {
//             Poll::Ready(Err(Error::msg(anyhow!("Couldn't connect to DB")).into()))
//         }
//     }

//     fn call(&mut self, request: Record<T>) -> Self::Future {
//         Box::pin(async move {
//             let okee = request.save().await?;
//             debug!("Record Saved! {}", okee.id()?);
//             Ok::<_, BoxError>(Frame::Fin)
//         })
//     }
// }

#[cfg(test)]
mod tests {
    use crate::Error;

    #[tokio::test]
    async fn test_get_database() -> Result<(), Error> {
        Ok(())
    }
}
