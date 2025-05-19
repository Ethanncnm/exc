//! Gate.io exchange services.

#![deny(missing_docs)]

/// REST API support.
pub mod http;

/// API Key utilities.
pub mod key;

/// Service definitions.
pub mod service;

pub use crate::service::{Endpoint, Gateio, GateioRequest, GateioResponse};
