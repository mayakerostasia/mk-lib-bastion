// use crate::Error;
use crate::Kong;
use anyhow::{anyhow, Error};
use bytes::Bytes;
use bb_lib_http_listener::Server;
use petname::Generator;
use rand::thread_rng;
use tokio::task::JoinHandle;
use tokio::task::{AbortHandle, JoinSet};
use tracing::info;
use tracing::{debug, error, instrument};


#[derive(Debug)]
pub struct KingKong {
    pub name: String,
    subject: String,
    nats_addr: String,
    listeners: JoinSet<Result<JoinHandle<Result<(), Error>>, Error>>,
    abort_handles: Vec<AbortHandle>,
    _http_listener: Option<Server>,
    _http_started: bool,
}

impl KingKong {
    pub fn new(subject: &str, nats_addr: &str) -> Self {
        let mut rng = thread_rng();
        let name = petname::Petnames::default()
            .generate(&mut rng, 2, "-")
            .expect("Petname Failed");

        let server = Server::new("0.0.0.0:6669");
        KingKong {
            name,
            subject: subject.to_string(),
            nats_addr: nats_addr.to_string(),
            // addr_table: HashMap::new(),
            listeners: JoinSet::new(),
            abort_handles: Vec::new(),
            _http_listener: Some(server),
            _http_started: false
        }
    }

    #[instrument(skip_all, fields(kingkong_name = %self.name, kingkong_subject = %self.subject))]
    async fn start_kong(
        &mut self,
        fut: impl futures::Future<Output = Result<JoinHandle<Result<(), Error>>, Error>>
            + Send
            + 'static,
    ) -> Result<(), Error>
// where
        // T: Send + futures::Future<Output = Result<JoinHandle<Result<(), Error>>, Error>> + 'static,
    {
        self.abort_handles.push(self.listeners.spawn(async move {
            match fut.await {
                Ok(handle) => {
                    info!("Kong Started");
                    Ok(handle)
                }
                Err(e) => {
                    error!("Kong Failed to start");
                    Err(Error::from(e))
                }
            }
        }));
        Ok(())
    }

    #[instrument(skip_all, fields(kingkong_name = %self.name, kingkong_subject = %self.subject, kong_subject = %subject))]
    pub async fn new_kong<T, U>(&mut self, subject: &str, func: fn() -> T) -> Result<(), Error>
    where
        T: Send + core::future::Future<Output = U> + 'static,
        U: Send + 'static + Into<Bytes> + std::fmt::Debug,
    {
        let kong = Kong::new(
            format!("{}.{}", self.subject, subject).as_str(),
            self.nats_addr.as_str(),
        )
        .await;
        self.start_kong(async move { Ok(kong.service(func).await?) })
            .await?;
        debug!("Kong is {:#?}", self);
        Ok(())
    }

    pub async fn wait(&self) -> Result<(), Error> {
        let fut1 = async {
                match tokio::signal::ctrl_c().await {
                Ok(()) => Ok(()),
                Err(err) => Err(anyhow!("Unable to listen for shutdown signal: {}", err)),
            }
        };

        let fut2 = async { self._http_listener.clone().unwrap().listen().await } ;
        tokio::select! {
            _ = fut1 => {}
            _ = fut2 => {}
        };
        Ok(())
    }
}
