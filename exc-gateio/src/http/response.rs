use serde::Deserialize;

/// Raw HTTP response data.
#[derive(Debug, Deserialize)]
pub struct HttpResponse(serde_json::Value);

impl From<serde_json::Value> for HttpResponse {
    fn from(v: serde_json::Value) -> Self {
        Self(v)
    }
}

impl HttpResponse {
    /// Access underlying json value.
    pub fn into_inner(self) -> serde_json::Value {
        self.0
    }
}
