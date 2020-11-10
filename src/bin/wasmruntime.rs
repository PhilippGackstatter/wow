extern crate tide;

use wasm_openwhisk::core;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = tide::new();

    app.at("/start").post(core::start);
    app.at("/init").post(core::init);
    app.at("/run").post(core::run);

    app.listen("127.0.0.1:9000").await.unwrap();

    Ok(())
}
