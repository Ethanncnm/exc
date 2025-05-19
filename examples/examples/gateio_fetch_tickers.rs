use exc::prelude::*;
use futures::future::TryFutureExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .init();

    let settle = std::env::var("SETTLE").unwrap_or_else(|_| "usdt".to_string());

    let mut gateio = exc::Gateio::endpoint().connect_exc();
    let req = exc_gateio::GateioRequest::ListFuturesTickers {
        settle,
        contract: None,
    };
    match gateio.request(req).await {
        Ok(resp) => println!("{}", resp.into_inner()),
        Err(err) => eprintln!("request error: {err}"),
    }

    Ok(())
}
