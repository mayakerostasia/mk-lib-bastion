use super::MakoReactor;
use bytes::Bytes;
use std::{future::Future, marker::PhantomData};
use tower::{BoxError, Layer, Service};

pub struct MakoLayer<T> {
    request_limit: usize,
    request_time: usize,
    _tee: PhantomData<T>,
}

impl<T> MakoLayer<T> {
    pub fn new(request_limit: usize, request_time: usize) -> MakoLayer<T> {
        MakoLayer {
            request_limit,
            request_time,
            _tee: PhantomData,
        }
    }
}

impl<S, T> Layer<S> for MakoLayer<T>
where
    S: Service<T>,
    S::Future: Future<Output = Result<S::Response, S::Error>>,
    S::Response: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    type Service = MakoReactor<S, S::Future, S::Response, S::Error>;

    fn layer(&self, inner: S) -> Self::Service {
        MakoReactor::new(self.request_limit, self.request_time, inner)
    }
}
