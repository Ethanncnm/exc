# exc-gateio

Gate.io exchange REST API support for the `exc` ecosystem.

Currently provides a minimal HTTP client with signed requests and an endpoint
builder. Example usage:

```rust
use exc_gateio::{Gateio, GateioRequest};

#[tokio::main]
async fn main() {
    let mut api = Gateio::endpoint().connect_exc();
    let res = api
        .request(GateioRequest::ListFuturesTickers {
            settle: "usdt".to_string(),
            contract: None,
        })
        .await
        .unwrap();
    println!("{}", res.into_inner());
}
```

