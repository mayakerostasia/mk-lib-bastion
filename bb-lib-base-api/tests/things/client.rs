use http_body_util::{
    BodyExt,
    Full,
    combinators::BoxBody,
};

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
        // Body,
        Bytes
    }
};

use hyper_util::rt::TokioIo;
use reqwest::{ Method, Url } ;

use bb_lib_base_api::{traits::RestClient, paged::Paged, client::Rest} ;

use super::calls::{TestCall, TestEchoCall};

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

fn create_resp_bytes(val: Value) -> Bytes 
{
    let body = serde_json::to_string(&val).unwrap();
    Bytes::from(body)
}

fn hyper_response(val: Value) -> hyper::Response<BoxBody<Bytes, hyper::Error>> {
    hyper::Response::new(full(create_resp_bytes(val)))
}

async fn echo(req: hyper::Request<body::Incoming>) -> Result<hyper::Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // Watch out for extra `/` in path... dunno why it's happening
    match (req.method(), &req.uri().path()[1..]) {
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

#[derive(Debug)]
pub struct TestClient {
    _handle: tokio::task::JoinHandle<()>,
}

impl TestClient {
    async fn start() -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
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

    pub async fn new() -> Self {
            let handle = TestClient::start();
            TestClient {_handle: handle.await.unwrap(),}
        }
}

impl RestClient for TestClient {
    fn base_url(&self) -> Url {
        Url::parse("http://127.0.0.1:6969").unwrap()
    }

    fn headers(&self) -> Option<HeaderMap> {
        None
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        req
    }
}

impl Drop for TestClient {
    fn drop(&mut self) {
        // self.handle.abort();
    }
}

fn _serde_de_ser(val: Value) -> Value {
    let val = serde_json::to_string(&val).unwrap();
    serde_json::from_str::<Value>(&val).unwrap()
}

fn gen_echo_call(offset: usize) -> (TestEchoCall, TestEchoCall) {
    let tc = TestEchoCall {
        counter: 0,
        paged: Paged {
            total: 100,
            offset,
            page_size: 10,
            abs_limit: 20,
            extra: HashMap::new(),
        },
        response: vec![],
    };
    (tc.clone(), tc)
}

fn gen_call(offset: usize) -> (TestCall, TestCall) {
    let tc = TestCall {
        counter: 0,
        paged: Paged {
            total: 100,
            offset,
            page_size: 10,
            abs_limit: 20,
            extra: HashMap::new(),
        }
    };
    (tc.clone(), tc)
}

pub async fn test_rest_call(client: &TestClient) {
    let rest = Rest::new(client);
    let (call, _call_control) = gen_call(0);   
    let resp = rest.call(client, &call).await.unwrap();
    assert_eq!(resp.0, StatusCode::OK);
    println!("{}", serde_json::to_string_pretty(&resp.1).unwrap());
    assert_eq!(resp.1, json!({"response":[]}));

}

pub async fn test_paged_call(client: &TestClient) {
    let rest = Rest::new(client);
    let (mut paged, paged_control) = gen_echo_call(0);   
    let (mut call, call_control) = gen_echo_call(0); 

    let paged_response = rest.paged_call(client, &paged, Some(100), Some(10)).await.unwrap();
    assert_eq!(paged_response.0, StatusCode::OK);
    assert_eq!(paged_response.1, json!({"total": 0, "page_size": 10, "offset": 0, "abs_limit": 100, "response":[]}));
        
    for i in 0..10 {
        let resp = rest.call(client, &call).await.unwrap();
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1, json!(gen_echo_call(i*10).0));
        call += call.clone();
    }

    // assert_eq!(resp.0, StatusCode::OK);
    // assert_eq!(resp.1, json!({"paged": Paged::default(), "response": []}));
}
