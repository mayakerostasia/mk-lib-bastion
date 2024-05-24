use std::ops::{ Add, AddAssign };
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Method;
use bb_lib_base_api::paged::Paged;
use bb_lib_base_api::traits::RestCall;
use bb_lib_base_api::client::IntermediateResponse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCall {
    pub paged: Paged,
    pub counter: usize,
}

impl Add for TestCall {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = self.clone();
        new.paged.offset = other.paged.offset + self.paged.page_size;
        new
    }
}

impl AddAssign for TestCall {
    fn add_assign(&mut self, other: Self) {
        self.paged.offset = other.paged.offset + self.paged.page_size;
    }
}

impl RestCall for TestCall {
    fn path(&self) -> String {
        r#"/"#.to_string()
    }

    fn method(&self) -> Method {
        Method::GET
    }

    fn query(&self) -> Option<Value> {
        None
    }

    fn body(&self) -> Option<Value> {
        None
    }

    fn paged(&self, response: IntermediateResponse, abs_limit: Option<usize>, _page_size: Option<usize>) -> Option<Vec<impl RestCall>> {
        let mut calls = vec![];
        let mut paged = response.paged.clone();

        paged.set_limit(abs_limit);

        let new_call = TestCall {
            counter: self.counter + 1,
            paged: Paged {
                total: paged.total,
                offset: paged.offset + paged.page_size,
                page_size: paged.page_size,
                abs_limit: paged.abs_limit,
                extra: paged.extra.clone(),
            }
        };
        calls.push(new_call);
        Some(calls)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestEchoCall {
    pub paged: Paged,
    pub counter: usize,
    pub response: Vec<Value>,
}

impl Add for TestEchoCall {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = self.clone();
        new.paged.offset = other.paged.offset + self.paged.page_size;
        new
    }
}

impl AddAssign for TestEchoCall {
    fn add_assign(&mut self, other: Self) {
        self.paged.offset = other.paged.offset + self.paged.page_size;
    }
}

impl RestCall for TestEchoCall {
    fn path(&self) -> String {
        "/echo".to_string()
    }

    fn method(&self) -> Method {
        Method::POST
    }

    fn query(&self) -> Option<Value> {
        None
    }

    fn body(&self) -> Option<Value> {
        Some(serde_json::to_value(self.clone()).unwrap())
    }

    fn paged(&self, response: IntermediateResponse, abs_limit: Option<usize>, _page_size: Option<usize>) -> Option<Vec<impl RestCall>> {
        let mut calls = vec![];
        let mut paged = response.paged.clone();

        paged.set_limit(abs_limit);

        let new_call = TestCall {
            counter: self.counter + 1,
            paged: Paged {
                total: paged.total,
                offset: paged.offset + paged.page_size,
                page_size: paged.page_size,
                abs_limit: paged.abs_limit,
                extra: paged.extra.clone(),
            }
        };
        calls.push(new_call);
        Some(calls)
    }
}
