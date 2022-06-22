use ow_executor::core;
// use tide_tracing::TraceMiddleware;
// use tracing::Level;

static ADDRESS: &str = "127.0.0.1:9000";

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "wasmtime_rt")]
    let runtime = ow_wasmtime::Wasmtime::default();

    #[cfg(feature = "wasmer_rt")]
    let runtime = ow_wasmer::Wasmer::default();

    #[cfg(feature = "wamr_rt")]
    let runtime = ow_wamr::Wamr::default();

    // let subscriber = tracing_subscriber::fmt()
    //     .with_max_level(Level::TRACE)
    //     .finish();

    // tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");

    let mut executor = tide::with_state(runtime);
    // executor.with(TraceMiddleware::new());

    executor.at("/:container_id/destroy").post(core::destroy);
    executor.at("/:container_id/init").post(core::init);
    executor.at("/:container_id/run").post(core::run);

    println!("Listening on: {}", ADDRESS);

    executor.listen(ADDRESS).await.unwrap();

    Ok(())
}
