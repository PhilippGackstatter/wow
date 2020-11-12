extern crate tide;

use async_std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use wasm_openwhisk::core;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(RwLock::new(HashMap::new()));

    let mut app = tide::with_state(state);

    app.at("/start").post(core::start);
    app.at("/init").post(core::init);
    app.at("/run").post(core::run);

    app.listen("127.0.0.1:9000").await.unwrap();

    Ok(())
}
