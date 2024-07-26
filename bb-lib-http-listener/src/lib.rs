use crate::handlers::{healthz_handler, readyz_handler};
use anyhow::Error;
use axum::{
    http::Request,
    routing::{get, Router},
};
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use shutdown::shutdown_signal;
use timer::TokioTimer;
use tokio::sync::watch;
use tower::Service;
use tracing::debug;
mod handlers;
mod shutdown;
mod timer;

#[derive(Clone, Debug)]
pub struct Server {
    _bind: String,
}

impl Server {
    pub fn new(bind: &str) -> Self {
        Server {
            _bind: bind.to_string(),
        }
    }

    pub async fn listen(&self) -> Result<(), Error> {
        let app = Router::new()
            .route("/healthz", get(healthz_handler))
            .route("/readyz", get(readyz_handler));
        let listener = tokio::net::TcpListener::bind(self._bind.clone())
            .await
            .unwrap();

        let (close_tx, close_rx) = watch::channel(());

        loop {
            let (socket, remote_addr) = tokio::select! {
                result = listener.accept() => {
                    result.unwrap()
                }
                _ = shutdown_signal() => {
                    debug!("signal received, not accepting new connections");
                    break;
                }
            };

            debug!(name: "health ->", remote_addr = %remote_addr);

            let tower_service = app.clone();
            let close_rx = close_rx.clone();

            tokio::spawn(async move {
                let socket = TokioIo::new(socket);

                let hyper_service =
                    hyper::service::service_fn(move |request: Request<Incoming>| {
                        tower_service.clone().call(request)
                    });

                let conn = hyper::server::conn::http1::Builder::new()
                    .header_read_timeout(tokio::time::Duration::from_millis(200))
                    .timer(TokioTimer)
                    .serve_connection(socket, hyper_service)
                    .with_upgrades();

                let mut conn = std::pin::pin!(conn);

                loop {
                    tokio::select! {
                        result = conn.as_mut() => {
                            if let Err(err) = result {
                                debug!("failed to serve connection: {err:#}");
                            }
                            break;
                        }
                        _ = shutdown_signal() => {
                            debug!("signal received, starting graceful shutdown");
                            conn.as_mut().graceful_shutdown();
                        }
                    }
                }

                debug!("connection {remote_addr} closed");

                drop(close_rx);
            });
        }
        drop(close_rx);
        drop(listener);
        debug!("waiting for {} tasks to finish", close_tx.receiver_count());
        close_tx.closed().await;
        Ok(())
    }
}
