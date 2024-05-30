use pin_project_lite::pin_project;
// use crate::Frame;

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll}
};

pin_project! {
    #[derive(Debug)]
    pub struct FrameFuture<T> {
        #[pin]
        response: T
    }
}

impl<T> FrameFuture<T> {
    pub fn new(response: T) -> Self {
        FrameFuture { response }
    }
}

impl<F, T, E> Future for FrameFuture<F> 
where
    F: Future<Output = Result<T, E>>,
    E: Into<anyhow::Error>,
{
    type Output = Result<T, anyhow::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.response.poll(cx) {
            Poll::Ready(v) => return Poll::Ready(v.map_err(Into::into)),
            Poll::Pending => Poll::Pending
        }
    }
}

