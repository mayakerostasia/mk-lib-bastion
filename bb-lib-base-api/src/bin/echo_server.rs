#[cfg(not(feature = "echoserver"))]
fn main() {
    println!("This example requires the 'server' feature to be enabled");
}

#[cfg(feature = "echoserver")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let handle = server::start().await?;
    handle.await?;
    Ok(())
}

#[cfg(feature = "echoserver")]
pub mod server {
    use http_body_util::{
        BodyExt,
        Full,
        combinators::BoxBody, };
    
    use serde_json::Value;
    use std::{net::SocketAddr, collections::HashMap};
    use tokio::{
        task::JoinHandle,
        net::TcpListener,
    };
    use serde_json::json;
    
    use hyper::{
        service::service_fn,
        server::conn::http1::Builder,
        HeaderMap,
        StatusCode,
        body::{
            self,
            Body,
            Bytes,
        },
    };
    
    use hyper_util::rt::TokioIo;
    use reqwest::{ Method, Url };
    
    use base_api::{traits::RestClient, paged::Paged, client::Rest};
    
    // use super::calls::{TestCall, TestEchoCall};
    
    fn empty() -> BoxBody<Bytes, hyper::Error> {
        Full::new(Bytes::new())
        .map_err(|never| match never {} )
        .boxed()
    }
    
    fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
        Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
    }
    
    fn create_resp_bytes(val: Value) -> Bytes {
        let body = serde_json::to_string(&val).unwrap();
        Bytes::from(body)
    }
    
    fn hyper_response(val: Value) -> hyper::Response<BoxBody<Bytes, hyper::Error>> {
        hyper::Response::new(full(create_resp_bytes(val)))
    }
    
    async fn echo(req: hyper::Request<body::Incoming>) -> Result<hyper::Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        match (req.method(), &req.uri().path()[..]) {
            (&Method::GET, "/") => {
                // Ok(hyper::Response::new(full(create_resp_bytes(json!({"response": []})))))
                Ok(hyper_response(json!({"response": []})))
            },
            (&Method::POST, "/echo") => {
                // Ok(hyper_response(json!({"response": []})))
                println!("Echo: {:?}", req.body());
                Ok(hyper::Response::new(req.into_body().boxed()))
            },
            (_, x) => {
                println!("404: {}", x);
                Ok(hyper::Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(empty())
                    .unwrap())
            },
        }
    }
    
    pub async fn start() -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 6969));

        // We create a TcpListener and bind it to 127.0.0.1:3000
        let listener = TcpListener::bind(addr).await?;

        // We start a loop to continuously accept incoming connections
        Ok(tokio::task::spawn( async move {         
            loop {
                let (stream, _) = listener.accept().await.unwrap();

                // Use an adapter to access something implementing `tokio::io` traits as if they implement
                // `hyper::rt` IO traits.
                let io = TokioIo::new(stream);

                // Spawn a tokio task to serve multiple connections concurrently
                tokio::task::spawn(async move {
                    // Finally, we bind the incoming connection to our `hello` service

                    if let Err(err) = Builder::new()
                        // .http1_only(true)
                        // .timer(TokioTimer::default())
                        // `service_fn` converts our function in a `Service`
                        .serve_connection(io, service_fn(echo))
                        .await
                    {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
        }))
    }
        
}
