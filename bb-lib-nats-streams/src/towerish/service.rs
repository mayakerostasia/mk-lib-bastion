use crate::towerish::error::FrameSendIssue;
use crate::Monkey;
use bytes::Bytes;
use futures::Future;
use pin_project_lite::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{BoxError, Service};
use tracing::{debug, error};

pin_project! {
    /// a Service<R: Into<Bytes>> that sends a
    /// msg to a nats subject and then calls an inner service
    ///
    #[derive(Debug, Clone)]
    pub struct NatsSend<S, Request>
    where
        S: Service<Request>,
        Request: Into<Bytes>,
    {
        #[pin]
        monkey: Option<Monkey>,
        pub subject: String,
        pub nats_url: String,
        phantom: PhantomData<Request>,
        pub inner_service: S,
    }
}

impl<S, Request> NatsSend<S, Request>
where
    S: Service<Request>,
    Request: Into<Bytes> + Clone + From<Bytes>,
{
    pub fn new(subject: &str, nats_url: &str, inner_service: S) -> Self {
        NatsSend {
            monkey: None,
            subject: subject.to_string(),
            nats_url: nats_url.to_string(),
            phantom: PhantomData,
            inner_service,
        }
    }

    async fn make_request(&self, req: Request) -> Result<Request, BoxError>
    where
        S: Service<Request>,
        Request: Clone + From<Bytes> + Into<Bytes>,
    {
        let monkey = self.monkey.as_ref().unwrap();
        let orig_req = req.clone();
        match monkey.msg(orig_req).await {
            Ok(resp) => {
                debug!("Response: {:#?}", resp);
                let new_req: Request = Into::<Request>::into(resp.payload.clone());
                Ok(new_req)
            }
            Err(e) => {
                error!("Error in `NatsSend::make_request` ! {:#?}", e);
                Err(FrameSendIssue.into())
            }
        }
    }
}

impl<S, Request> Service<Request> for NatsSend<S, Request>
where
    Request: Clone + Into<Bytes> + From<Bytes> + Send + Sync + 'static,
    S: Service<Request, Response = Request> + Clone + Send + Sync + 'static,
    S::Future: Send + Sync,
    S::Error: Into<BoxError> + Send + Sync,
    Self: Clone,
{
    type Response = Request;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, BoxError>> + Send + Sync>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Initialize nats client
        let subj = self.subject.clone();
        let url = self.nats_url.clone();
        let mut monkey_future = Box::pin(Monkey::new(subj.as_str(), url.as_str()));
        while self.monkey.is_none() {
            match monkey_future.as_mut().poll(cx) {
                Poll::Pending => {},
                Poll::Ready(monkey) => {
                    debug!("Monkey ready");
                    let _ = self.monkey.insert(monkey);
                    return Poll::Ready(Ok(()));
                }
            };
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let clone = self.inner_service.clone();
        let self_clone = self.clone();
        let mut inner = std::mem::replace(&mut self.inner_service, clone);
        let fut = async move {
            inner
                .call(self_clone.make_request(req).await?)
                .await
                .map_err(Into::into)
        };
        Box::pin(fut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::Service;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct MockService;

    impl Service<Bytes> for MockService {
        type Response = Bytes;
        type Error = BoxError;
        type Future =
            Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: Bytes) -> Self::Future {
            let fut = async move {
                // Process the request and return a response
                Ok(req)
            };
            Box::pin(fut)
        }
    }

    #[tokio::test]
    async fn test_nats_send() -> Result<(), BoxError> {
        let subject = "test_subject";
        let nats_url = "nats://10.2.4.106:4222";
        let inner_service = MockService;
        let mut nats_send = NatsSend::new(subject, nats_url, inner_service.clone());
        let sender = nats_send.ready().await;
        assert!(sender.is_ok());
        Ok(())
    }
}
