use crate::{core::FrameFuture, Decoder, Frame, Kong, NSLibError};
use anyhow::anyhow;
use bb_lib_http_listener::Server;
use bytes::Bytes;
use core::future::Future;
use petname::Generator;
use rand::thread_rng;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::{AbortHandle, JoinHandle, JoinSet};
use tower::{BoxError, Service};
use tracing::{debug, error, info, info_span, instrument, instrument::Instrumented, Instrument};

type Error = NSLibError;

#[derive(Debug)]
pub struct KingKong {
    pub name: String,
    subject: String,
    nats_addr: String,
    listeners: JoinSet<Result<Instrumented<JoinHandle<Result<(), BoxError>>>, BoxError>>,
    abort_handles: Vec<Instrumented<AbortHandle>>,
    _http_listener: Option<Server>,
    _http_started: bool,
}

// impl Service<Bytes> for KingKong {
//     type Error = BoxError;
//     type Response = Frame;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     fn poll_ready(
//         &mut self,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         std::task::Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, req: Bytes) -> Self::Future {
//         // let payload = req.payload.clone();
//         let frame: Frame = Frame::decode(&req);
//         Box::pin(async { Ok(frame) })
//     }
// }

// impl Service<Frame> for KingKong {
//     type Error = BoxError;
//     type Response = Frame;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

//     fn poll_ready(
//         &mut self,
//         _cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         std::task::Poll::Ready(Ok(()))
//     }

//     fn call(&mut self, req: Frame) -> Self::Future {
//         // let payload = req.payload.clone();
//         // let frame: Frame = Frame::decode(&req);
//         Box::pin(async { Ok(req) })
//     }
// }

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
            _http_started: false,
        }
    }

    #[instrument(skip_all, fields(kingkong_name = %self.name, kingkong_subject = %self.subject))]
    async fn start_kong(
        &mut self,
        fut: impl futures::Future<
                Output = Result<Instrumented<JoinHandle<Result<(), BoxError>>>, BoxError>,
            > + Send
            + 'static,
    ) -> Result<(), BoxError> {
        self.abort_handles.push(
            self.listeners
                .spawn(async move {
                    match fut.await {
                        Ok(handle) => {
                            debug!("Kong Started");
                            Ok(handle)
                        }
                        Err(e) => {
                            error!("Kong Failed to start");
                            Err(Error::Anyhow(anyhow!(e)))?
                            // Err(Error::from(e))
                        }
                    }
                })
                .instrument(info_span!("kong_listener")),
        );
        Ok(())
    }

    #[instrument(skip_all, fields(kingkong_name = %self.name, kingkong_subject = %self.subject, kong_subject = %subject))]
    pub async fn new_kong<T, U>(&mut self, subject: &str, func: fn() -> T) -> Result<(), BoxError>
    where
        T: Send + core::future::Future<Output = U> + 'static,
        U: Send + 'static + Into<Bytes> + std::fmt::Debug,
    {
        let kong = Kong::new(
            format!("{}.{}", self.subject, subject).as_str(),
            self.nats_addr.as_str(),
        )
        .await;
        let name = kong.name.clone();
        self.start_kong(
            async move { Ok::<_, BoxError>(kong.service(func).await?) }
                .instrument(info_span!("kong_func").or_current()),
        )
        .await?;
        info!(kong_name = name, kong_subject = subject, "Kong Up");
        Ok::<_, BoxError>(())
    }

    #[instrument(skip_all, fields(kingkong_name = %self.name, kingkong_subject = %self.subject, kong_subject = %subject))]
    pub async fn new_future_kong<O, T>(
        &mut self,
        subject: &str,
        func: fn(Frame) -> O,
        // func: fn(T) -> U,
    ) -> Result<(), BoxError>
    where
        O: Future<Output = Result<T, BoxError>> + Send + 'static,
        T: Into<Bytes> + std::fmt::Debug + Send + 'static,
        // T: Send + core::future::Future<Output = U> + 'static,
        // U: Send + 'static + Into<Bytes> + std::fmt::Debug,
    {
        let kong = Kong::new(
            format!("{}.{}", self.subject, subject).as_str(),
            self.nats_addr.as_str(),
        )
        .await;
        let name = kong.name.clone();
        self.start_kong(
            async move { 
                Ok::<_, BoxError>(kong.service_future(func).await?) 
            }.instrument(info_span!("kong_func").or_current()),
        ).await?;
        info!(kong_name = name, kong_subject = subject, "Kong Up");
        Ok::<_, BoxError>(())
    }

    pub async fn new_tower_kong<'a, S>(
        &'a mut self,
        subject: &'a str,
        service: Arc<Mutex<S>>, // bytes: Bytes,
    ) -> Result<(), BoxError>
    where
        S: Clone + tower::Service<Frame> + Send + Sync + 'static,
        S::Future: Send + Sync,
        S::Response: Into<Bytes> + Send + Sync,
        S::Error: Into<BoxError>,
    {
        let kong = Kong::new(
            format!("{}.{}", self.subject, subject).as_str(),
            self.nats_addr.as_str(),
        )
        .await;
        let name = kong.name.clone();
        self.start_kong(
            async move { Ok(kong.tower_service(service).await?) }
                .instrument(info_span!("kong_func").or_current()),
        )
        .await?;
        info!(kong_name = name, kong_subject = subject, "Kong Up");
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
        tokio::select! {
            _ = fut1 => {}
            _ = fut2 => {}
        };
        Ok(())
    }
}
