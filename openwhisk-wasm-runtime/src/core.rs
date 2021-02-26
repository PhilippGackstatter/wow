use std::time::Instant;

use crate::types::{ActivationContext, ActivationInit, ActivationResponse, WasmRuntime};
use serde::Serialize;
use tide::Request;

#[derive(Serialize)]
struct RuntimeResponse {
    #[serde(rename(serialize = "containerId"))]
    container_id: String,
    port: i32,
}

pub async fn start(mut _req: Request<impl WasmRuntime>) -> tide::Result<serde_json::Value> {
    // println!("start called with {:#?}", req.body_string().await);

    let resp = RuntimeResponse {
        container_id: "1".into(),
        port: 9000,
    };

    Ok(serde_json::to_value(resp).unwrap())
}

pub async fn init(mut req: Request<impl WasmRuntime>) -> tide::Result<tide::StatusCode> {
    let activation_init = req.body_json().await;

    if let Err(err) = &activation_init {
        println!("/init err: {:?}", err);
    }

    let activation_init: ActivationInit = activation_init?;

    println!("/init {:#?}", activation_init);

    let time = Instant::now();
    let module_bytes: Vec<u8> = base64::decode(activation_init.value.code)?;
    println!("base64 decoding took {}µs", time.elapsed().as_micros());

    let action_name = activation_init
        .value
        .env
        .get("__OW_ACTION_NAME")
        .unwrap()
        .clone();

    let runtime = req.state();

    runtime.initialize_action(action_name, activation_init.value.annotations, module_bytes)?;

    Ok(tide::StatusCode::Ok)
}

pub async fn run(mut req: Request<impl WasmRuntime>) -> tide::Result<serde_json::Value> {
    let activation_context: ActivationContext = req.body_json().await?;

    println!("/run {:#?}", activation_context);

    let runtime = req.state();

    let result = runtime.execute(&activation_context.action_name, activation_context.value);

    println!("Wasm Execution returned {:?}", result);

    let response = ActivationResponse::new(result?);

    Ok(serde_json::to_value(response).unwrap())
}
