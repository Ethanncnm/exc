use http::{Method, Request};
use serde::Serialize;

use crate::key::GateioKey;

use super::response::HttpResponse;
use exc_core::ExchangeError;

/// HTTP request variants.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum HttpRequest {
    /// List futures tickers.
    ListFuturesTickers {
        settle: String,
        #[serde(skip_serializing_if = "Option::is_none")] contract: Option<String>,
    },
    /// Get a single futures contract.
    GetFuturesContract { settle: String, contract: String },
    /// List all futures contracts.
    ListFuturesContracts { settle: String },
    /// List all currencies' details.
    ListCurrencies,
    /// Get details of a specific currency.
    GetCurrency { currency: String },
}

impl HttpRequest {
    pub(crate) fn method(&self) -> Method {
        Method::GET
    }

    pub(crate) fn path(&self) -> String {
        match self {
            Self::ListFuturesTickers { settle, .. } => format!("/futures/{}/tickers", settle),
            Self::GetFuturesContract { settle, contract } => {
                format!("/futures/{}/contracts/{}", settle, contract)
            }
            Self::ListFuturesContracts { settle } => {
                format!("/futures/{}/contracts", settle)
            }
            Self::ListCurrencies => String::from("/spot/currencies"),
            Self::GetCurrency { currency } => {
                format!("/spot/currencies/{}", currency)
            }
        }
    }

    pub(crate) fn query(&self) -> Option<String> {
        match self {
            Self::ListFuturesTickers { contract: Some(c), .. } => {
                Some(format!("contract={}", c))
            }
            _ => None,
        }
    }

    pub(crate) fn to_request(
        &self,
        host: &str,
        key: Option<&GateioKey>,
    ) -> Result<Request<hyper::Body>, ExchangeError> {
        let method = self.method();
        let path = self.path();
        let query = self.query();
        let uri = match &query {
            Some(q) => format!("{}{}?{}", host, &path, q),
            None => format!("{}{}", host, &path),
        };
        let mut builder = Request::builder().method(&method).uri(uri);
        if let Some(key) = key {
            let sign = key
                .sign_now(method.as_str(), &path, query.as_deref(), None)
                .map_err(|e| ExchangeError::Other(e.into()))?;
            builder = builder
                .header("KEY", key.key.as_str())
                .header("Timestamp", sign.timestamp.as_str())
                .header("SIGN", sign.sign.as_str());
        }
        builder
            .body(hyper::Body::empty())
            .map_err(|e| ExchangeError::Other(e.into()))
    }
}

/// HTTP response.
pub type HttpResponseData = HttpResponse;
