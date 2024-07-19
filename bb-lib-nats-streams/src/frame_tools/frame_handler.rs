use super::proc::Proc;
use super::frame::Frame;
use std::future::Future;
use std::pin::Pin;
use tower::BoxError;
use tracing::{debug, error};
use async_trait::async_trait;

type ProcFunk = fn(Proc) -> Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync + 'static>>; 

#[async_trait]
pub trait HandlesFrames<P>
where
    P: Into<Frame> + Send + Sync + 'static,
{
    fn frame_handler(
        &self,
        frame: P,
        funk: ProcFunk
    ) -> Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync + 'static>> ;
}

#[derive(Clone)]
pub struct FrameHandler;

impl<P> HandlesFrames<P> for FrameHandler
where
    P: Into<Frame> + Clone + Send + Sync + 'static,
{
    fn frame_handler(
        &self,
        frame: P,
        funk: fn(Proc) -> Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync + 'static>>
    ) -> Pin<Box<dyn Future<Output = Result<Frame, BoxError>> + Send + Sync + 'static>> {
        Box::pin(async move {
            debug!("Frame Handler Started");
            let fr: Frame = frame.into();
            Ok(match fr.clone() {
                Frame::Ping => Frame::Pong,
                Frame::Pong => Frame::Fin,
                Frame::Exec(proc) => funk(proc).await?,
                Frame::Json(val) => {
                    debug!("Frame Handler received JSON");
                    Frame::json(val)
                }
                Frame::Error(stri) => {
                    error!(stri);
                    Frame::Error(stri)
                }
                Frame::Close => {
                    error!("Received Close Frame");
                    Frame::Fin
                }
                _ => {
                    error!("Error: {}", "Failed to decode frame");
                    Frame::ping()
                }
            })
        })
    }
}
