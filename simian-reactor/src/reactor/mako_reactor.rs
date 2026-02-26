use crate::ReactorError;
// use anyhow::Error;

use super::BoxError;
type Error = BoxError;
use super::{Frame, Proc, ReactorCore};
use std::{future::Future, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct MakoReactor<S, F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
    S: Clone,
{
    pub core: Arc<Mutex<ReactorCore<F, R, E>>>,
    pub service: S,
}

impl<S, F, R, E> Clone for MakoReactor<S, F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            service: self.service.clone(),
        }
    }
}

impl<S, F, R, E> MakoReactor<S, F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
    S: Clone,
{
    pub fn new(request_limit: usize, request_time: usize, srv: S) -> MakoReactor<S, F, R, E> {
        MakoReactor {
            core: Arc::new(Mutex::new(ReactorCore::new(request_limit, request_time))),
            service: srv,
        }
    }

    pub async fn get_function(
        &self,
        service_name: &str,
    ) -> Result<fn(Vec<String>) -> F, ReactorError> {
        self.core.lock().await.get_function(service_name)
    }

    pub async fn register_function(
        &self,
        service_name: &str,
        func: fn(Vec<String>) -> F,
    ) -> Result<(), Error> {
        // todo!()
        Ok(self.core.lock().await.register_function(service_name, func)?)
    }

    pub async fn call_registered_function(&self, proc: Proc) -> Result<Frame, Error> {
        Ok(self
            .core
            .lock()
            .await
            .call_registered_function(proc)
            .await?)
    }
}
