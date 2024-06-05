use std::{future::Future, pin::Pin, task::Poll};
use tower::{BoxError, Service};

use bytes::Bytes;

pub struct DbService {}

impl DbService { 
    pub fn new() -> Self {
        DbService {  }
    }
}

impl<Request> Service<Request> for DbService
where
    Request: Into<Bytes> + From<Bytes> + Send + 'static,
{
    type Response = Request;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        // let db = self.db.read().unwrap();
        // let client = db.client;
        Box::pin(async {
            // TODO: Rewrite Me!!!!
            Ok::<_, BoxError>(request)
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
