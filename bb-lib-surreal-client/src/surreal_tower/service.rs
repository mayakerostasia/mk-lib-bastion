use crate::Error; use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::task::{Context, Poll};
use surrealdb::sql::Value;
use tower::{BoxError, Service};

use bytes::Bytes;

use super::db_request::{Cmd, DbRequest, DbResponse, PopRequest};
use std::sync::atomic::Ordering;

pub struct DbService {}

impl DbService { 
    pub fn new() -> Self {
        DbService {  }
    }
}

impl<Request> Service<Request> for DbService
where
    Request: Into<Bytes>,
{
    type Response = Request;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        if let true = self.db.read().unwrap().connected.load(Ordering::Relaxed) {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn call(&mut self, request: DbRequest) -> Self::Future {
        let db = self.db.read().unwrap();
        let client = db.client;
        let cmd = request;
        let response = match cmd.pop_request() {
            // Cmd::CreateTable(name, value) => {
            //     let response = crate::create_table(name.as_str(), value);
            //     DbResponse { future: response }
            // }
            // Cmd::Get(table, id) => {
            //     let response = crate::get_db_record(table.as_str(),id.as_str());
            //     DbResponse { future: response }
            // }
            // Cmd::Set(table, id, value) => {
            //     let response = crate::query(id, value);
            //     DbResponse { future: response }
            // }
            Cmd::Query(query) => {
                let response = crate::query(query.as_str());
                DbResponse { future: response }
            }
            // Cmd::CallFn(name, args) => {
            //     let response = crate::call_fn(name, args);
            //     DbResponse { future: response }
            // }
            _ => DbResponse {
                future: Err(Error::ServiceError("Invalid command".to_string())),
            },
        };
        DbResponse { future: response }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_database() -> Result<(), Error> {
        Ok(())
    }
}
