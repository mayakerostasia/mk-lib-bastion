use pin_project_lite::pin_project;
use crate::Frame;
use futures::Future;
use bytes::Bytes;
use std::{task::Poll};
use std::pin::Pin;


use tower::BoxError;

// pin_project! {
//     pub struct FutureFrame<T, E> where T: Into<Bytes> {
//         #[pin]
//         frame: T,
//         err: Option<E>
//     }
// }
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

// impl<T, E> Future for FutureFrame<T, E> 
// where
//     T: Into<Bytes>,
//     E: Into<BoxError>,
// {
//     type Output = Result<T, E>;

//     fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
//         let this = self.project();

//         match this.frame.poll(cx) {
//             Poll::Ready(result) => {
//                 return Poll::Ready(result)
//             },
//             Poll::Pending => Poll::Pending,
//         }
//         
//     }
// }

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
            Poll::Ready(result) => {
                // let result = result.map_err(Into::into);
                return Poll::Ready(result);
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
