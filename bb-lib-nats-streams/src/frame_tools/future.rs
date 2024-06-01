use futures::Future;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::Poll;

use tower::BoxError;

pub type PinnedFuture<R, E> = Pin<Box<dyn Future<Output = Result<R, E>> + Send + Sync>>;

pin_project! {
    pub struct FrameFuture<F, R, E>
    where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>
    {
        #[pin]
        inner: F
    }
}

impl<F, R, E> FrameFuture<F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
{
    pub fn new(fut: F) -> Self {
        FrameFuture { inner: fut }
    }
}

impl<F, R, E> Future for FrameFuture<F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
{
    type Output = Result<R, E>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(result) => Poll::Ready(result),
            Poll::Pending => Poll::Pending,
        }
    }
}
