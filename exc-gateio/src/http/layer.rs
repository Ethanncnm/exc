use exc_core::{retry::RetryPolicy, ExchangeError};
use futures::future::{ready, BoxFuture};
use futures::{FutureExt, TryFutureExt};
use http::{Request, Response};
use hyper::Body;
use tower::{retry::Retry, Layer, Service, ServiceBuilder};

use super::request::HttpRequest;
use super::response::HttpResponse;
use crate::key::GateioKey;

/// Gate.io HTTP API layer.
#[derive(Clone)]
pub struct GateioHttpApiLayer {
    host: String,
    key: Option<GateioKey>,
    retry: RetryPolicy<HttpRequest, HttpResponse, fn(&ExchangeError) -> bool>,
}

impl Default for GateioHttpApiLayer {
    fn default() -> Self {
        Self {
            host: "https://api.gateio.ws/api/v4".to_string(),
            key: None,
            retry: RetryPolicy::never(),
        }
    }
}

impl GateioHttpApiLayer {
    /// Set API key.
    pub fn key(mut self, key: GateioKey) -> Self {
        self.key = Some(key);
        self
    }

    /// Set host.
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    /// Retry on function.
    pub fn retry_on(self, f: fn(&ExchangeError) -> bool) -> Self {
        Self { retry: RetryPolicy::default().retry_on(f), ..self }
    }
}

impl<S> Layer<S> for GateioHttpApiLayer
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone,
    S::Future: Send + 'static,
    S::Error: 'static,
    ExchangeError: From<S::Error>,
{
    type Service = Retry<RetryPolicy<HttpRequest, HttpResponse, fn(&ExchangeError) -> bool>, GateioHttpApi<S>>;

    fn layer(&self, inner: S) -> Self::Service {
        let svc = GateioHttpApi {
            host: self.host.clone(),
            http: inner,
            key: self.key.clone(),
        };
        ServiceBuilder::default()
            .retry(self.retry.clone())
            .service(svc)
    }
}

/// Gate.io HTTP API Service.
#[derive(Clone)]
pub struct GateioHttpApi<S> {
    host: String,
    key: Option<GateioKey>,
    http: S,
}

impl<S> Service<HttpRequest> for GateioHttpApi<S>
where
    S: Service<Request<Body>, Response = Response<Body>>,
    S::Future: Send + 'static,
    S::Error: 'static,
    ExchangeError: From<S::Error>,
{
    type Response = HttpResponse;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.http.poll_ready(cx).map_err(ExchangeError::from)
    }

    fn call(&mut self, req: HttpRequest) -> Self::Future {
        let host = self.host.clone();
        let key = self.key.clone();
        match req.to_request(&host, key.as_ref()) {
            Ok(req) => {
                self.http
                    .call(req)
                    .map_err(ExchangeError::from)
                    .and_then(|resp| {
                        hyper::body::to_bytes(resp.into_body())
                            .map_err(|e| ExchangeError::Other(e.into()))
                    })
                    .and_then(|bytes| ready(serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| ExchangeError::Other(e.into()))))
                    .map_ok(HttpResponse)
                    .boxed()
            }
            Err(err) => ready(Err(err)).boxed(),
        }
    }
}
