use ow_common::{ActivationContext, ActivationInit, ActivationResponse, WasmRuntime};
use async_std::task;
use serde::Serialize;
use tide::{Request, StatusCode};

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

pub async fn init(mut req: Request<impl WasmRuntime>) -> tide::Result<StatusCode> {
    let activation_init = req.body_json().await;

    if let Err(err) = &activation_init {
        println!("/init err: {:?}", err);
    }

    let activation_init: ActivationInit = activation_init?;

    println!("/init {:#?}", activation_init);

    let action_name = activation_init
        .value
        .env
        .get("__OW_ACTION_NAME")
        .ok_or_else(|| {
            tide::Error::from_str(StatusCode::BadRequest, "Missing __OW_ACTION_NAME property")
        })?
        .clone();

    let runtime = req.state();

    runtime.initialize_action(
        action_name,
        activation_init.value.annotations,
        activation_init.value.code,
    )?;

    Ok(StatusCode::Ok)
}

pub async fn run(
    mut req: Request<impl WasmRuntime + Send + Sync + 'static>,
) -> tide::Result<serde_json::Value> {
    let activation_context: ActivationContext = req.body_json().await?;

    println!("/run {:#?}", activation_context);

    // Create a cheap clone of the runtime that can be moved onto another thread
    let runtime = req.state().clone();

    let result = task::spawn_blocking(move || {
        runtime.execute(&activation_context.action_name, activation_context.value)
    })
    .await;

    println!("Wasm Execution returned {:?}", result);

    let response = ActivationResponse::new(result?);

    Ok(serde_json::to_value(response).unwrap())
}
