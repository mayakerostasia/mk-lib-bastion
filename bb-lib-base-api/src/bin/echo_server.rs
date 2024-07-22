use clap::Parser;

#[cfg(not(feature = "echoserver"))]
fn main() {
    println!("This example requires the 'server' feature to be enabled");
}

#[cfg(feature = "echoserver")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let handle = server::start(&args.bind, &args.port).await?;
    handle.await?;
    Ok(())
}

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
struct Args { 
    #[arg(short, long)]
    bind: String,
    #[arg(short, long)]
    port: String,
}

#[cfg(feature = "echoserver")]
pub mod server {
    use http_body_util::{combinators::BoxBody, BodyExt, Full};

    use serde_json::json;
    use serde_json::Value;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use tokio::{net::TcpListener, task::JoinHandle};

    use hyper::{
        body::{self, Bytes},
        server::conn::http1::Builder,
        service::service_fn,
    };

    use hyper_util::rt::TokioIo;

    // fn empty() -> BoxBody<Bytes, hyper::Error> {
    //     Full::new(Bytes::new())
    //         .map_err(|never| match never {})
    //         .boxed()
    // }

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

    async fn echo(
        req: hyper::Request<body::Incoming>,
    ) -> Result<hyper::Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
        let (_, x) = (req.method(), req.uri().path());
        println!("-----");
        println!("Path {}", x);
        let body = req.boxed();
        println!("Body {:#?}", body.collect().await);
        // .collect().await.inspect(|body| {eprintln!("{:#?}", body)});
        Ok(hyper_response(json!({"response": true})))
    }

    pub async fn start(bind: &str, port:&str) -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from_str(format!("{}:{}",bind, port).as_str())?;

        // We create a TcpListener and bind it to 127.0.0.1:3000
        let listener = TcpListener::bind(addr).await?;

        // We start a loop to continuously accept incoming connections
        Ok(tokio::task::spawn(async move {
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
