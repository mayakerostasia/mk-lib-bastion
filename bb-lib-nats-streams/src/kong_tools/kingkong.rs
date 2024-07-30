use crate::{Frame, Kong, NSLibError};
use anyhow::anyhow;
use bb_lib_http_listener::Server;
use bytes::Bytes;
use core::future::Future;
use petname::Generator;
use rand::thread_rng;
use std::collections::HashMap;
use tokio::task::{AbortHandle, JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::{
    debug, error, info, info_span,
    warn, instrument,
};

type Error = NSLibError;
type InstrumentedJoinHandle = JoinHandle<Result<(), BoxError>>;
type InstrumentedAbortHandle = AbortHandle;

#[derive(Debug)]
pub struct KingKong {
    pub name: String,
    subject: String,
    nats_addr: String,
    addr_table: HashMap<String, String>,
    abort_handles: Vec<AbortHandle>,
    kongs: Vec<Kong>,
    listeners: JoinSet<Result<InstrumentedJoinHandle, BoxError>>,
    cancel_token: CancellationToken,
    _http_listener: Option<Server>,
    _http_started: bool,
}

impl KingKong {
    pub fn new(subject: &str, nats_addr: &str, health_bind: &str) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");

        let server = Server::new(health_bind);
        KingKong {
            name,
            subject: subject.to_string(),
            nats_addr: nats_addr.to_string(),
            addr_table: HashMap::new(),
            listeners: JoinSet::new(),
            abort_handles: Vec::new(),
            kongs: Vec::new(),
            cancel_token: CancellationToken::new(),
            _http_listener: Some(server),
            _http_started: false,
        }
    }

    async fn init_kong(&mut self, subject: &str) -> Result<(String, String, Kong), Error> {
        debug!("Init Kong @ {subject}");
        let nats_subject = format!("{}.{}", self.subject.as_str(), subject);
        let kong = Kong::new(&nats_subject, self.nats_addr.as_str()).await?;
        let name = kong.name.clone();
        self.addr_table.insert(nats_subject.clone(), name.clone());
        // self.kongs.insert(name.clone(), kong);
        Ok((nats_subject, name, kong))
    }

    async fn start_kong(
        &mut self,
        fut: impl std::future::Future<Output = Result<InstrumentedJoinHandle, BoxError>>
            + Send
            + 'static,
    ) -> Result<(), BoxError> {
        debug!("Kong Starting");
        let handle = self.listeners.spawn(async move {
            match fut.await {
                Ok(handle) => {
                    debug!("Kong Started");
                    Ok(handle)
                }
                Err(e) => {
                    error!("Kong Failed to start");
                    Err(Error::Anyhow(anyhow!(e)))?
                }
            }
        });
        self.abort_handles.push(handle);
        debug!("Exiting kong_start");
        Ok(())
    }

    pub async fn new_kong<'a, T, U>(
        &'a mut self,
        subject: &'a str,
        func: fn() -> T,
    ) -> Result<(), BoxError>
    where
        T: Send + std::future::Future<Output = U> + 'static,
        U: Send + Into<Bytes> + std::fmt::Debug + 'static,
    {
        let (nats_subject, name, kong) = self.init_kong(subject).await?;
        self.start_kong(async move { 
            kong.service(func).await 
        })
            .await?;
        info!(%nats_subject, king_kong_name = self.name, kong_name = name, kong_subject = subject, "Kong Up");
        Ok::<_, BoxError>(())
    }

    #[instrument]
    pub async fn new_future_kong<'a, O, T>(
        &'a mut self,
        subject: &'a str,
        func: fn(Frame) -> O,
        // func: fn(T) -> U,
    ) -> Result<(), BoxError>
    where
        O: Future<Output = Result<T, BoxError>> + Send + 'static,
        T: Into<Bytes> + std::fmt::Debug + Send,
    {
        let (nats_subject, name, kong) = self.init_kong(subject).await?;
        self.start_kong(async move { kong.service_future(func).await })
            .await?;
        info!(%nats_subject, king_kong_name = self.name, kong_name = name, kong_subject = subject, "Kong Up");
        Ok::<_, BoxError>(())
    }

    pub async fn new_tower_kong<'a, S>(
        &'a mut self,
        subject: &'a str,
        service: S, // bytes: Bytes,
    ) -> Result<(), BoxError>
    where
        S: Clone + tower::Service<Frame> + Send + Sync + 'static,
        S::Future: Send + Sync,
        S::Response: Into<Bytes> + Send + Sync + std::fmt::Debug,
        S::Error: Into<BoxError>,
    {
        let (nats_subject, name, kong) = self.init_kong(subject).await?;
        self.start_kong(async move { kong.tower_service(service).await })
            .await?;
        info!(%nats_subject, king_kong_name = self.name, kong_name = name, kong_subject = subject, "Kong Up");
        Ok::<_, BoxError>(())
    }

    pub async fn wait(&self) -> Result<(), Error> {
        let fut1 = async {
            match tokio::signal::ctrl_c().await {
                Ok(()) => Ok(()),
                Err(err) => Err(anyhow!("Unable to listen for shutdown signal: {}", err)),
            }
        };
        let fut2 = async { self._http_listener.clone().unwrap().listen().await };
        // let cancel_token = self.cancel_token.cancelled();
        tokio::select! {
            _ = fut1 => {}
            _ = fut2 => {}
            // _ = cancel_token => {
            //     warn!("Kancelled! Exiting!");
            //     return Ok(())
            // }
        };
        Ok(())
    }
}
