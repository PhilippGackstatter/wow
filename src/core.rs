use std::{collections::HashMap, sync::Arc};

use crate::types::{ActivationContext, ActivationInit, ActivationResponse};
use async_std::sync::RwLock;
use serde::Serialize;
use tide::{Request, StatusCode};

type AtomicHashMap = Arc<RwLock<HashMap<String, Vec<u8>>>>;

#[derive(Serialize)]
struct RuntimeResponse {
    #[serde(rename(serialize = "containerId"))]
    container_id: String,
    port: i32,
}

pub async fn start(mut _req: Request<AtomicHashMap>) -> tide::Result<serde_json::Value> {
    // println!("start called with {:#?}", req.body_string().await);

    let resp = RuntimeResponse {
        container_id: "1".into(),
        port: 9000,
    };

    Ok(serde_json::to_value(resp).unwrap())
}

pub async fn init(mut req: Request<AtomicHashMap>) -> tide::Result<tide::StatusCode> {
    let activation_init: ActivationInit = req.body_json().await?;

    println!("/init {:#?}", activation_init);

    let wasm_bytes = activation_init.value.code.into();
    let key = activation_init
        .value
        .env
        .get("__OW_ACTION_NAME")
        .unwrap()
        .clone();

    let mut map = req.state().write().await;

    map.insert(key, wasm_bytes);

    Ok(tide::StatusCode::Ok)
}

pub async fn run(mut req: Request<AtomicHashMap>) -> tide::Result<serde_json::Value> {
    let activation_context: ActivationContext = req.body_json().await?;

    println!("/run {:#?}", activation_context);

    let map = req.state().write().await;

    let wasm_bytes = map
        .get(&activation_context.action_name)
        .ok_or_else(|| tide::Error::from_str(StatusCode::NotFound, "No action with that name"))?;

    // TODO: Don't try!, but pass error into ActivationRes. and use `ApplicationDeveloperError`
    let response = crate::wasmtime::execute_wasm(activation_context.value, wasm_bytes)?;

    let response = ActivationResponse::new(response);

    Ok(serde_json::to_value(response).unwrap())
}
