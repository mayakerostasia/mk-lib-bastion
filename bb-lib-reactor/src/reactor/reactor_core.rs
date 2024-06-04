use anyhow::Error;
use petname::Generator;
use rand::thread_rng;
use std::{collections::HashMap, future::Future};
use tokio::time::Duration;
use tower::BoxError;

use super::{Frame, Proc};
use crate::{FrameFuture, ReactorError};

pub struct ReactorCore<F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
{
    name: String,
    request_limit: usize,
    request_limit_time: Duration,
    services: HashMap<String, fn(Vec<String>) -> F>,
    // _counter: Counter,
    // _battery: MakoBattery,
    // _mako_requests: Vec<Mako>,
}

impl<F, R, E> std::fmt::Debug for ReactorCore<F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MakoReactor")
            .field("name", &self.name)
            .field("request_limit", &self.request_limit)
            .field("request_limit_time", &self.request_limit_time)
            .finish()
    }
}

impl<F, R, E> ReactorCore<F, R, E>
where
    F: Future<Output = Result<R, E>>,
    E: Into<BoxError>,
{
    pub fn new(request_limit: usize, request_time: usize) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");

        // let battery_capacity = request_limit;
        // let battery_interval = Duration::from_secs_f64(1.0 / battery_capacity as f64);

        ReactorCore {
            name: name.to_string(),
            request_limit,
            request_limit_time: Duration::new(request_time as u64, 0),
            services: HashMap::new(),
            // _counter: counter!(format!("mako-reactor-{name}")),
            // _mako_requests: Vec::new(),
            // _battery: MakoBattery::new(battery_interval, battery_capacity),
        }
    }

    pub fn get_function(&self, service_name: &str) -> Result<fn(Vec<String>) -> F, ReactorError> {
        match self.services.get(service_name) {
            Some(srv_fn) => Ok(*srv_fn),
            None => Err(ReactorError("No function of that Name".to_string())),
        }
    }

    async fn react(&self, proc: Proc) -> Result<Frame, Error> {
        // let service_fn = self.get_function(proc.cmd.as_str())?;
        // Ok(service_fn(proc.args).await?)
        todo!("Hit react function")
    }

    pub fn register_function(
        &mut self,
        service_name: &str,
        func: fn(Vec<String>) -> F,
    ) -> Result<(), Error> {
        self.services.insert(service_name.to_string(), func);
        Ok(())
    }

    pub async fn call_registered_function(&self, proc: Proc) -> Result<Frame, Error> {
        Ok(self.react(proc).await?)
    }
}
