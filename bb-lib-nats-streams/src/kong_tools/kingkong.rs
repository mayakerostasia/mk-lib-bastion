use crate::{Frame, Kong, NSLibError, Monkey, Decoder};
use anyhow::anyhow;
use bb_lib_http_listener::Server;
use bytes::Bytes;
use core::future::Future;
use petname::Generator;
use rand::thread_rng;
use std::{collections::HashMap, time::Duration};
use tokio::task::{AbortHandle, JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;
use tower::BoxError;
use tracing::{debug, error, info, instrument};

type Error = NSLibError;
type JoinHandleResult = JoinHandle<Result<(), BoxError>>;

#[derive(Debug)]
pub struct KingKong {
    pub name: String,
    pub subject: String,
    nats_addr: String,
    addr_table: HashMap<String, String>,
    abort_handles: Vec<AbortHandle>,
    listeners: JoinSet<Result<JoinHandleResult, BoxError>>,
    cancel_token: CancellationToken,
    http_listener: Server,
    _http_started: bool,
    monkey: Monkey,
}

impl KingKong {
    pub async fn new(subject: &str, nats_addr: &str, health_bind: &str) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");

        let mut kk = KingKong {
            name: name.clone(),
            subject: subject.to_string(),
            nats_addr: nats_addr.to_string(),
            addr_table: HashMap::new(),
            listeners: JoinSet::new(),
            abort_handles: Vec::new(),
            cancel_token: CancellationToken::new(),
            http_listener: Server::new(health_bind),
            _http_started: false,
            monkey: Monkey::new(subject,nats_addr).await,
        };
        kk.new_kong(format!("{}-health", name.clone()).as_str(), || async { Frame::pong() }).await.expect("Failed to start Kong");
        kk
    }

    pub fn get_subjects(&self) ->HashMap<String, String> {
        self.addr_table.clone()
    }

    async fn check_subject(&self, subject: &str) -> bool {
        eprintln!("Checking Subject");
        let mut monk = self.monkey.clone();
        monk.set_subject(subject).expect("Failed to set monkey subject");
        let resp = monk.msg_timeout(Frame::ping(), Some(Duration::from_millis(500))).await;
        match resp {
            Ok(resp_msg) => {
                let resp_frame = Frame::decode(&resp_msg.payload).expect("Failed to decode Frame");
                Frame::pong() == resp_frame
            },
            Err(e) => {
                eprintln!("Error! -> {e:#?}");
                false
            }
        }
    }

    pub async fn health(&self) -> bool {
        eprintln!("Checking Health");
        for (subject, name) in self.get_subjects().iter() {
            if self.check_subject(subject).await {
                eprintln!("subject={} name={} OK!", subject, name);
                continue;
            } else {
                return false;
            }
        }
        true
    }

    async fn init_kong(&mut self, subject: &str) -> Result<(String, String, Kong), Error> {
        debug!("Init Kong @ {subject}");
        let nats_subject = format!("{}.{}", self.subject.as_str(), subject);
        let kong = Kong::new(&nats_subject, self.nats_addr.as_str()).await?;
        let name = kong.name.clone();
        self.addr_table.insert(nats_subject.clone(), name.clone());
        Ok((nats_subject, name, kong))
    }

    async fn start_kong(
        &mut self,
        fut: impl std::future::Future<Output = Result<JoinHandleResult, BoxError>> + Send + 'static,
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
        self.start_kong(async move { kong.service(func).await })
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
        let fut2 = async { self.http_listener.clone().listen().await };
        let cancel_token = self.cancel_token.cancelled();

        let health_failure = async {
            loop {
                tokio::time::sleep(Duration::from_secs(10)).await;
                if self.health().await {
                    eprintln!("Health OK!");
                } else {
                    return Err::<(), _>(NSLibError::Anyhow(anyhow!("Health Failure!")));
                };
            };
        };

        tokio::select! {
            _ = fut1 => {}
            _ = fut2 => {}
            _ = cancel_token => {
                eprintln!("Kancelled! Exiting!");
                return Ok(())
            }
            _ = health_failure => {
                eprintln!("King Kong Unhealthy Going down!");
            }
        };
        Ok(())
    }
}
