// use std::io::Write;
// use reqwest::Response;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

use futures::stream::FuturesOrdered;
use futures::StreamExt;
use reqwest::header::{self, HeaderMap};
use reqwest::StatusCode;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, warn, instrument};

use crate::traits::{RestCall, RestClient};
use crate::{paged::Paged, RestSvcError};

#[derive(Debug, Clone)]
pub struct RestSvcReq {}

#[derive(Debug, Clone)]
pub struct RestSvcResp(pub StatusCode, pub Value);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateResponse {
    #[serde(alias = "issues", alias = "objects")]
    pub response: Value,
    #[serde(flatten)]
    pub paged: Paged,
}

impl Add for IntermediateResponse {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = self.clone();
        let mut _resp =
            serde_json::from_value::<HashMap<String, Value>>(self.response.clone()).unwrap();
        let _other =
            serde_json::from_value::<HashMap<String, Value>>(other.response.clone()).unwrap();
        _resp
            .get_mut("response")
            .expect(" Failed to unwrap response during add")
            .as_array_mut()
            .unwrap()
            .extend(_other["response"].as_array().unwrap().clone());
        new.response = serde_json::to_value(_resp).unwrap();
        new.paged = other.paged;
        new
    }
}

impl AddAssign for IntermediateResponse {
    fn add_assign(&mut self, other: Self) {
        let mut _resp = serde_json::from_value::<Vec<Value>>(self.response.clone()).unwrap();
        let _other = serde_json::from_value::<Vec<Value>>(other.response.clone()).unwrap();

        _resp.extend(_other);
        self.response = serde_json::to_value(_resp).unwrap();
        self.paged = other.paged;
    }
}

fn rest_ok(response: &reqwest::Response) -> bool {
    response.status().is_success()
}

fn has_content(_response: &reqwest::Response) -> bool {
    true
    // let content_length = dbg!(response.content_length());

    // if let Some(content_length) = content_length {
    //     content_length > 0
    // } else {
    //     false
    // }
}

fn contentless_ok(response: &reqwest::Response) -> bool {
    rest_ok(response) && !has_content(response)
}

fn contentful_ok(response: &reqwest::Response) -> bool {
    rest_ok(response) && has_content(response)
}

#[derive(Debug, Clone)]
pub struct Rest {
    pub client: reqwest::Client,
}

impl Rest {
    fn _build_headers(headers: Option<HeaderMap>) -> header::HeaderMap {
        let mut headers = headers.unwrap_or_else(HeaderMap::new);
        headers.insert(header::ACCEPT, "application/json".parse().unwrap());
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers
    }

    #[instrument(skip_all)]
    pub async fn call(
        &self,
        client: &impl RestClient,
        call: &impl RestCall,
    ) -> Result<RestSvcResp, RestSvcError> {
        let url = format!("{}{}", client.base_url(), call.path());
        let mut req = client.auth(self.client.request(call.method(), &url));

        if let Some(query) = call.query() {
            [&query].iter().for_each(|q| debug!("Query: {:#?}", q));
            req = req.query(&query);
        }
        if let Some(body) = call.body() {
            req = req.body(serde_json::to_string(&body).unwrap());
        }
        debug!("--> Request: {:#?}", req);
        let resp = req.send().await?;
        debug!("<-- Response Status: {:?}", resp.status());
        debug!("<-- Response: {:#?}", &resp);

        if contentful_ok(&resp) {
            Ok(RestSvcResp(
                resp.status(),
                resp.json::<Value>().await.expect("Whoops!"),
            ))
        } else if contentless_ok(&resp) {
            Ok(RestSvcResp(resp.status(), Value::Null))
        } else {
            Err(RestSvcError::OtherError(format!(
                "Failed Request: (status={}) (Content-length={:?}) content={:?}",
                resp.status(),
                resp.content_length(),
                resp.text().await?
            )))
        }
    }

    #[instrument(skip_all)]
    pub async fn paged_call(
        &self,
        client: &impl RestClient,
        call: &impl RestCall,
        abs_limit: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<RestSvcResp, RestSvcError> {
        let resp: IntermediateResponse =
            serde_json::from_value::<IntermediateResponse>(self.call(client, call).await?.1)?;
        let new_rest_call = call.clone();
        // eprintln!("Resp: {:?}", resp);
        // eprintln!("New Call: {:?}", new_call);
        let mut ret_resp = resp.clone();

        let calls = new_rest_call
            .paged(resp.clone(), abs_limit, page_size)
            .unwrap_or_else(|| {
                warn!(
                    "Failed to unwrap calls from paged call: {:?} \n Continuing!",
                    resp.paged
                );
                vec![]
            });
        // eprintln!("Calls: {:?}", calls);

        // let mut page_iter = resp.paged.clone();

        let mut task_que = FuturesOrdered::new();
        calls.iter().for_each(|_call| {
            task_que.push_back(self.call(client, _call));
        });

        while let Some(_resp) = task_que.next().await {
            let new_resp = _resp?;
            debug!(status = ?new_resp.0);
            if let StatusCode::OK = new_resp.0 {
                ret_resp += serde_json::from_value::<IntermediateResponse>(new_resp.1)?;
            }
        }

        // todo!(r#"
        //     // let new_resp = self.call(client, &call).await?;
        //     // if let StatusCode::OK = new_resp.0 {
        //     //     ret_resp += serde_json::from_value::<IntermediateResponse>(new_resp.1)?;
        //     // }
        // "#);
        Ok(RestSvcResp(
            StatusCode::OK,
            serde_json::to_value(ret_resp).unwrap(),
        ))
    }

    pub fn new(client: &impl RestClient) -> Self {
        Rest {
            client: reqwest::Client::builder()
                .default_headers(Self::_build_headers(client.headers()))
                .build()
                .expect("Failed to create reqwest client"),
        }
    }
}
