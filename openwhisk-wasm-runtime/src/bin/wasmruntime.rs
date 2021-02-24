extern crate tide;

use openwhisk_wasm_runtime::core;
use openwhisk_wasm_runtime::wasmtime::Wasmtime;
use tide_tracing::TraceMiddleware;
use tracing::Level;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Wasmtime::default();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");

    let mut app = tide::with_state(state);
    app.with(TraceMiddleware::new());

    app.at("/start").post(core::start);
    app.at("/init").post(core::init);
    app.at("/run").post(core::run);

    app.listen("127.0.0.1:9000").await.unwrap();

    Ok(())
}
