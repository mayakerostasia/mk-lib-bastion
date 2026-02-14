use std::future::Future;
use simian_nats_streams::{Proc, Frame};
use tower::{Service, BoxError};

use crate::{FrameFuture, MakoReactor};

pub struct ProcService<S,F,R,E> 
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>
{
    reactor: MakoReactor<S, F, R, E>
}

impl<S> Service<S> for ProcService<S, S::Future, S::Response, S::Error>  
where
    S: Service<S>,
    S::Future: Future<Output = Result<S::Response, S::Error>> + Send + Sync,
    S::Error: Into<BoxError> + Send + Sync,
{
    type Response = S::Response;
    type Future = S::Future;
    type Error = S::Error;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.reactor.service.poll_ready(cx)
    }

    fn call(&mut self, req: Proc) -> Self::Future {
        // self.call(req)
        Box::pin(async {
            Ok(self.reactor.call_registered_function(req).await?)
        })
    }
}
