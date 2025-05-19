use exc_core::transport::http::{channel::HttpsChannel, endpoint::Endpoint as HttpEndpoint};
use exc_core::{Exc, ExchangeError};
use futures::future::BoxFuture;
use futures::FutureExt;
use tower::{buffer::Buffer, Service};

use crate::http::{
    layer::{GateioHttpApi, GateioHttpApiLayer},
    request::HttpRequest,
    response::HttpResponse,
};
use crate::key::GateioKey;

/// Gate.io request.
pub type GateioRequest = HttpRequest;

/// Gate.io response.
pub type GateioResponse = HttpResponse;

/// Service for Gate.io HTTP API.
#[derive(Clone)]
pub struct Gateio {
    inner: Buffer<GateioHttpApi<HttpsChannel>, GateioRequest>,
}

impl Service<GateioRequest> for Gateio {
    type Response = GateioResponse;
    type Error = ExchangeError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(ExchangeError::from)
    }

    fn call(&mut self, req: GateioRequest) -> Self::Future {
        self.inner.call(req).map_err(ExchangeError::from).boxed()
    }
}

impl Gateio {
    fn new(inner: GateioHttpApi<HttpsChannel>, cap: usize) -> Self {
        Gateio {
            inner: Buffer::new(inner, cap),
        }
    }

    /// Create a default endpoint.
    pub fn endpoint() -> Endpoint {
        Endpoint::default()
    }
}

/// Endpoint builder.
pub struct Endpoint {
    host: String,
    key: Option<GateioKey>,
    buffer: usize,
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            host: "https://api.gateio.ws/api/v4".to_string(),
            key: None,
            buffer: 128,
        }
    }
}

impl Endpoint {
    /// Set custom host.
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    /// Private mode.
    pub fn private(mut self, key: GateioKey) -> Self {
        self.key = Some(key);
        self
    }

    /// Buffer capacity.
    pub fn buffer(mut self, cap: usize) -> Self {
        self.buffer = cap;
        self
    }

    /// Connect to service.
    pub fn connect(&self) -> Gateio {
        let mut layer = GateioHttpApiLayer::default().host(&self.host);
        if let Some(key) = self.key.clone() {
            layer = layer.key(key);
        }
        let http = HttpEndpoint::default().connect_https();
        let svc = layer.layer(http);
        Gateio::new(svc, self.buffer)
    }

    /// Connect and convert into an exc service.
    pub fn connect_exc(&self) -> Exc<Gateio, GateioRequest> {
        Exc::new(self.connect())
    }
}

