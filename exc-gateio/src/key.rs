use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use exc_core::Str;

/// Signing error.
#[derive(Debug, Error)]
pub enum SignError {
    /// Secret key length error.
    #[error("secret key length error")]
    SecretKeyLength,

    /// Timestamp formatting error.
    #[error("timestamp format error: {0}")]
    Timestamp(#[from] time::error::Format),
}

type HmacSha512 = Hmac<Sha512>;

/// Gate.io API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateioKey {
    /// API key.
    pub key: Str,
    /// Secret.
    pub secret: Str,
}

/// Signature result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature string.
    #[serde(rename = "SIGN")]
    pub sign: Str,
    /// Timestamp used.
    #[serde(rename = "Timestamp")]
    pub timestamp: Str,
}

impl GateioKey {
    /// Create new key.
    pub fn new(key: &str, secret: &str) -> Self {
        Self {
            key: Str::new(key),
            secret: Str::new(secret),
        }
    }

    /// Sign with given parameters at specific time.
    pub fn sign_at(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body: Option<&str>,
        time: OffsetDateTime,
    ) -> Result<Signature, SignError> {
        let ts = time.format(&Rfc3339)?;
        let mut hasher = Sha512::new();
        if let Some(body) = body {
            hasher.update(body.as_bytes());
        }
        let hashed = format!("{:x}", hasher.finalize());
        let raw = format!(
            "{}\n{}\n{}\n{}\n{}",
            method,
            path,
            query.unwrap_or(""),
            hashed,
            ts
        );
        let mut mac =
            HmacSha512::new_from_slice(self.secret.as_str().as_bytes()).map_err(|_| SignError::SecretKeyLength)?;
        mac.update(raw.as_bytes());
        let sign = hex::encode(mac.finalize().into_bytes());
        Ok(Signature {
            sign: Str::new(sign),
            timestamp: Str::new(ts),
        })
    }

    /// Sign with current timestamp.
    pub fn sign_now(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body: Option<&str>,
    ) -> Result<Signature, SignError> {
        self.sign_at(method, path, query, body, OffsetDateTime::now_utc())
    }
}
