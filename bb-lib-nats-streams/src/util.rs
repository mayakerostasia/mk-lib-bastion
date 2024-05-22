// use crate::Error;
use core::future::Future;
use std::pin::Pin;

// pub type Incrementer = Box<dyn FnOnce(u32) -> Pin<Box<dyn Future<Output = u32>>>>;

// pub fn force_boxed<T>(f: fn(u32) -> T) -> Incrementer 
// where
//     T: Future<Output = u32> + 'static
// {
//     Box::new(move |n| Box::pin(f(n)))
// }

pub fn annotate<T, F>(f: F) -> F where F: Fn(T) -> T {  
    f  
}  

// async fn async_hello_string() -> Result<String, Error> {
//     Ok(async { "Hello".to_string() }.await)
// }

pub type PinnedFuture<F> = Pin<Box<dyn Future<Output = F> + Send>>;
pub type BoxedFutureFn<B> = Box<dyn Fn() ->PinnedFuture<B> + Send>;

pub fn boxed_future_generator<F, Fut, O>(f: F) -> BoxedFutureFn<O>
where
    F: Fn() -> Fut + Send + 'static,
    Fut: Future<Output = O> + Send + 'static,
    O: Send + 'static,
    // E: std::error::Error,
{
    Box::new(move || Box::pin(f()))
}

// pub fn boxed_trait_generator<T: Send>(f: Box<fn() -> T>) -> BoxedFutureFn
// where
//     T: Future<Output = Result<String, Error>> + 'static,
// {
//     Box::new(move || Box::pin(f()))
// }

// pub async fn print_and_return_future<T: std::fmt::Debug>(inc: BoxedFutureFn<T>) -> Result<T, Error> {
//     let increment = inc().await;
//     println!("{:#?}", &increment);
//     Ok(increment)
// }
