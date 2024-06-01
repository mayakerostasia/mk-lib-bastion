use core::future::Future;
use std::pin::Pin;

pub type PinnedFuture<F> = Pin<Box<dyn Future<Output = F> + Send>>;
pub type BoxedFutureFn<B> = Box<dyn Fn() -> PinnedFuture<B> + Send>;

pub fn boxed_future_generator<F, Fut, O>(f: F) -> BoxedFutureFn<O>
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = O> + Send + 'static,
    O: Send,
    // E: std::error::Error,
{
    Box::new(move || Box::pin(f()))
}

