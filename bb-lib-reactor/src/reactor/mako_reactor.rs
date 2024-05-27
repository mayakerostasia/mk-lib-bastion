#![allow(unused)]
use anyhow::anyhow;
use chrono::Utc;
use anyhow::Error;
use bb_lib_nats_streams::Proc;
use chrono::DateTime;
use bb_lib_nats_streams::Frame;
use bytes::Bytes;
use core::panic;
use std::borrow::BorrowMut;
use metrics::counter;
use metrics::Counter;
use petname::Generator;
use rand::thread_rng;
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use crate::FramedFuture;
// use chrono::Date


#[derive(Debug, Clone)]
pub struct ArcReactor(pub Arc<Mutex<MakoReactor>>);

impl ArcReactor
{
    pub fn new(request_limit: usize, request_time: usize) -> ArcReactor
    {
        ArcReactor(Arc::new(Mutex::new(MakoReactor::new(
            request_limit,
            request_time,
        ))))
    }

    pub async fn get_service(&self, service_name: &str) -> Result<fn(Vec<String>) -> FramedFuture<Frame>, Error> {
        self.0.lock().await.get_service(service_name)
    }

    pub async fn register_service(
        &self,
        service_name: &str,
        func: fn(Vec<String>) -> FramedFuture<Frame>,
    ) -> Result<(), Error> {
        // todo!()
        self.0.lock().await.register_service(service_name, func)
    }

    pub async fn call_service(&self, proc: Proc) -> Result<Frame, Error> {
        Ok(self.0.lock().await.call_service(proc).await?)
    }
}


pub struct MakoReactor
{
    name: String,
    request_limit: usize,
    request_limit_time: Duration,
    services: HashMap<String, fn(Vec<String>) -> FramedFuture<Frame>>,
    // _counter: Counter,
    // _battery: MakoBattery,
    // _mako_requests: Vec<Mako>,
}

impl std::fmt::Debug for MakoReactor
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MakoReactor")
            .field("name", &self.name)
            .field("request_limit", &self.request_limit)
            .field("request_limit_time", &self.request_limit_time)
            .finish()
    }
}

#[allow(unused)]
pub struct Mako {
    request: String,
    time: DateTime<Utc>,
}

impl Mako {
    fn new(name: &str) -> Self {
        Mako {
            request: name.to_string(),
            time: Utc::now()
        }
    }
}

use std::pin::Pin;
use tower::Service;

impl Service<Frame> for ArcReactor
{
    type Response = Frame;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn call(&mut self, req: Frame) -> Self::Future {
        let me = self.clone();
        let fut = async move {
            match req {
                Frame::Exec(proc) => {
                    me.call_service(dbg!(proc)).await
                },
                Frame::Ping => {
                    Ok( Frame::pong() )
                },
                _ => {
                    Err(anyhow!("Not Allowed!"))
                }
            }
        };
        Box::pin(fut)
    }

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

impl MakoReactor
{
    pub fn new(request_limit: usize, request_time: usize) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");

        let battery_capacity = request_limit;
        let battery_interval = Duration::from_secs_f64(1.0 / battery_capacity as f64);

        MakoReactor {
            name: name.to_string(),
            request_limit,
            request_limit_time: Duration::new(request_time as u64, 0),
            services: HashMap::new(),
            // _counter: counter!(format!("mako-reactor-{name}")),
            // _mako_requests: Vec::new(),
            // _battery: MakoBattery::new(battery_interval, battery_capacity),
        }
    }

    pub fn get_service(
        &self,
        service_name: &str,
    ) -> Result<fn(Vec<String>) -> FramedFuture<Frame>, Error> {
        match self.services.get(service_name) {
            Some(srv_fn) => Ok(*srv_fn),
            None => Err(anyhow!("404")),
        }
    }

    async fn react(&self, proc: Proc) -> Result<Frame, Error> {
        let service_fn = self.get_service(proc.cmd.as_str())?;
        Ok(service_fn(proc.args).await?)
    }

    pub fn register_service(
        &mut self,
        service_name: &str,
        func: fn(Vec<String>) -> FramedFuture<Frame>,
    ) -> Result<(), Error> {
        self.services.insert(service_name.to_string(), func);
        Ok(())
    }

    pub async fn call_service(
        &self,
        proc: Proc
    ) -> Result<Frame, Error> {
        Ok(self.react(proc).await?)
    }
}
