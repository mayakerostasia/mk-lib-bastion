use crate::FrameFuture;
use crate::protocol::Frame;

use super::BoxError;
use super::MakoReactor;
use bytes::Bytes;
use std::convert::Infallible;
use std::{future::Future, pin::Pin};
use futures::FutureExt;
use tower::Service;
// use tracing::error;

type Error = tower::BoxError;

async fn ret_frame(frame: Frame) -> Result<Frame, Error> {
    Ok(frame)
}

async fn match_future<F, R, E>(ff: impl Future<Output = Result<Frame, Error>>) -> Result<Frame, Error> {
    match ff.await {
        Err(e) => {
            eprintln!("Whoops! {}", e);
            Err(Error::from(e))
        },
        Ok(frame) => Ok(frame)
    }
}

async fn ret_error(e: &str) -> Result<Infallible, Error> {
    Err(Error::from(format!("Whoops! -> {}", e)))
}


impl<S, T> Service<T> for MakoReactor<S, S::Future, S::Response, S::Error>
where
    S: Service<
        T, 
        Future = Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync>>,
        Response = Frame,
        Error = BoxError,
    > + Sync + Send + Clone + 'static,
    T: Into<Bytes>,
    // F: S::Future,
    // E: Into<BoxError> + Send + Sync,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn call(&mut self, req: T) -> Self::Future {
        let byt: Bytes = req.into();
        let frame: Frame = Frame::from(byt);
        let reactor = self.clone();
        
        Box::pin(async move {
            let response_frame = match frame {
                Frame::Exec(proc) => reactor.call_registered_function(proc).await?,
                Frame::Error(e) => {
                    // Decode error already converted to Frame::Error
                    Frame::Error(e)
                }
                _ => Frame::Error("Expected Frame::Exec".to_string()),
            };
            
            Ok::<_, BoxError>(response_frame)
        })
    }

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }
}
