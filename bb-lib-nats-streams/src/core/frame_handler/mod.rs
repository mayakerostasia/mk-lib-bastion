use tower::{Layer, Service};
use std::task::Poll;
use tracing::debug;

pub mod future;

#[derive(Debug)]
pub struct FrameHandler<S> {
    service: S
}


impl<S, T> Service<T> for FrameHandler<S> 
where
    S: Service<T>,
    T: Into<T> + std::fmt::Debug,
{
    type Error = S::Error;
    type Future = S::Future;
    type Response = S::Response;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: T) -> Self::Future {
        debug!("Request is {req:#?}");
        let tee: T = req.into();
        self.service.call(tee)
    }
}

pub struct FrameHandlerLayer;

impl <S> Layer<S> for FrameHandlerLayer
{
    type Service = FrameHandler<S>;

    fn layer(&self, inner: S) -> Self::Service {
        FrameHandler { service: inner }
    }
}
