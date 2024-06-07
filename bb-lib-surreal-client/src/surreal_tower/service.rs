use crate::Record;
use crate::Storable;
use crate::{connect, DbConfig, DbGuard};
use crate::{Error, DB};
use anyhow::anyhow;
use bb_lib_nats_streams::Frame;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use std::{future::Future, marker::PhantomData, pin::Pin, task::Poll};
use tower::{BoxError, Service};
use tracing::debug;
use bytes::Bytes;

pin_project! {
    #[derive(Debug, Clone)]
    pub struct DbService<S, Request>
    where
        S: Service<Request>,
        // Request: Into<Bytes>,
    {
        #[pin]
        db_connect: Option<DbGuard>,
        inner_service: S,
        _phantom_request: PhantomData<Request>,
    }
}

#[allow(unused)]
impl<S, Request> DbService<S, Request>
where
    S: Service<Request>,
    Request: Send + 'static,
{
    pub async fn new(cfg: DbConfig, inner_service: S) -> Result<Self, Error> {
        let guard = match connect(&cfg).await {
            Ok(gu) => gu,
            Err(e) => return Err(Error::msg(e)),
        };
        Ok(DbService {
            db_connect: Some(guard),
            inner_service,
            _phantom_request: PhantomData::<Request>,
        })
    }
}

impl<S, T> Service<Record<T>> for DbService<S, Record<T>>
where
    S: Service<Record<T>>,
    T: /* Into<Bytes> */ Clone
        + Serialize
        + for<'de> Deserialize<'de>
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
    Record<T>: Storable<T>,
{
    type Response = Frame;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        if self.db_connect.is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(Error::msg(anyhow!("Couldn't connect to DB")).into()))
        }
    }

    fn call(&mut self, request: Record<T>) -> Self::Future {
        Box::pin(async move {
            let okee = request.save().await?;
            debug!("Record Saved! {}", okee.id()?);
            Ok::<_, BoxError>(Frame::Fin)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[tokio::test]
    async fn test_get_database() -> Result<(), Error> {
        Ok(())
    }
}
