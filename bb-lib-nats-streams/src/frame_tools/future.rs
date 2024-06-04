use futures::Future;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::Poll;

use tower::BoxError;

pub type PinnedFuture<R, E> = Pin<Box<dyn Future<Output = Result<R, E>> + Send + Sync>>;

pin_project! {
    #[derive(Debug)]
    pub struct FrameFuture<F, T> {
        #[pin]
        state: FrameFutureStatus<F, T>,
    }
}

pin_project! {
    #[project = FrameFutureProj]
    #[derive(Debug)]
    enum FrameFutureStatus<F, T>
    {
        Poll { #[pin] fut: F },
        Error { err: Option<BoxError> },
        Ok { res: T },
    }
}

impl<F, T> FrameFuture<F, T> {
    pub fn new(fut: F) -> Self {
        FrameFuture {
            state: FrameFutureStatus::Poll { fut },
        }
    }

    pub fn fail<E: Into<BoxError>>(err: E) -> Self {
        FrameFuture {
            state: FrameFutureStatus::Error {
                err: Some(err.into()),
            },
        }
    }
}

impl<F, T> Future for FrameFuture<F, T>
where
    F: Future<Output = Result<T, BoxError>>,
    T: Clone,
{
    type Output = Result<T, BoxError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();

        match this.state.as_mut().project() {
            // match fut.poll(cx) {
            FrameFutureProj::Error { err } => {
                Poll::Ready(Err::<T, BoxError>(err.take().unwrap()))
            }
            FrameFutureProj::Poll { fut } => fut.poll(cx).map_err(Into::into),
            FrameFutureProj::Ok { res } => {
                let result = res.clone();
                this.state.set(FrameFutureStatus::Ok { res: result.clone() });
                Poll::Ready(Ok(result))
            }
        }
    }
}
