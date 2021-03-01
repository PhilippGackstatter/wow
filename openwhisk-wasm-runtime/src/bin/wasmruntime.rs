extern crate tide;

use std::time::Duration;

use async_std::task::{self};
use openwhisk_wasm_runtime::core;
use serde_json::json;
use tide_tracing::TraceMiddleware;
use tracing::Level;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "wasmtime_rt")]
    let runtime = openwhisk_wasm_runtime::wasmtime::Wasmtime::default();

    #[cfg(feature = "wasmer_rt")]
    let runtime = openwhisk_wasm_runtime::wasmer::Wasmer::default();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");

    let mut app = tide::with_state(runtime);
    app.with(TraceMiddleware::new());

    app.at("/start").post(core::start);
    app.at("/init").post(core::init);
    app.at("/run").post(core::run);
    app.at("/block").get(block);

    app.listen("127.0.0.1:9000").await.unwrap();

    Ok(())
}

pub async fn block(
    mut _req: tide::Request<impl openwhisk_wasm_runtime::types::WasmRuntime>,
) -> tide::Result<serde_json::Value> {
    let thread_id = std::thread::current().id();

    // task::spawn_blocking(|| {
    std::thread::sleep(Duration::new(1, 0));
    // })
    // .await;

    Ok(json!({
        "thread_id": format!("{:?}", thread_id),
    }))
}
