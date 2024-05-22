use std::fmt::Debug;
use serde_json::Value;
use reqwest::{ header::HeaderMap, Method, RequestBuilder, Url };
use crate::client::IntermediateResponse;

pub trait RestClient {
    fn base_url(&self) -> Url;
    fn headers(&self) -> Option<HeaderMap> ;
    fn auth(&self, req: RequestBuilder) -> RequestBuilder;
}

pub trait RestCall where Self: Sized + Debug + Clone {
    fn path(&self) -> String;
    fn method(&self) -> Method;
    fn query(&self) -> Option<Value>;
    fn body(&self) -> Option<Value>;
    fn paged(&self, response: IntermediateResponse, abs_limit: Option<usize>, page_size: Option<usize>) -> Option<Vec<impl RestCall>>; 
}
