use ow_executor::core;
use tide_tracing::TraceMiddleware;
use tracing::Level;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "wasmtime_rt")]
    let runtime = ow_wasmtime::Wasmtime::default();

    #[cfg(feature = "wasmer_rt")]
    let runtime = ow_wasmer::Wasmer::default();

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");

    let mut app = tide::with_state(runtime);
    app.with(TraceMiddleware::new());

    app.at("/:container_id/destroy").post(core::destroy);
    app.at("/:container_id/init").post(core::init);
    app.at("/:container_id/run").post(core::run);

    app.listen("127.0.0.1:9000").await.unwrap();

    Ok(())
}
