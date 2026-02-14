use futures::ready;
use pin_project_lite::pin_project;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pin_project! {
    /// Future that completes when the buffered service eventually services the submitted request.
    #[derive(Debug)]
    pub struct ResponseFuture<T> {
        #[pin]
        state: ResponseState<T>,
    }
}

pin_project! {
    #[project = ResponseStateProj]
    #[derive(Debug)]
    enum ResponseState<T> {
        Failed {
            error: Option<crate::BoxError>,
        },
        Frame {
            #[pin]
            frame: T,
        },
        Poll {
            #[pin]
            fut: T
        }
    }
}

impl<T> ResponseFuture<T> {
    pub fn new(fut: T) -> Self {
        ResponseFuture {
            state: ResponseState::Poll { fut },
        }
    }

    pub fn failed(err: crate::BoxError) -> Self {
        ResponseFuture {
            state: ResponseState::Failed { error: Some(err) },
        }
    }
}

impl<F, T, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: Into<crate::BoxError>,
{
    type Output = Result<T, crate::BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();
        match this.state.as_mut().project() {
            ResponseStateProj::Failed { error } => {
                Poll::Ready(Err(error.take().expect("polled after error")))
            }
            ResponseStateProj::Frame { frame } => match ready!(frame.poll(cx)) {
                Ok(fut) => Poll::Ready(Ok(fut)),
                Err(e) => Poll::Ready(Err(e.into())),
            },
            ResponseStateProj::Poll { fut } => fut.poll(cx).map_err(Into::into),
        }
    }
}
