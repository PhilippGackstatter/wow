use async_std::task;
use ow_common::{util, ActivationContext, ActivationInit, ActivationResponse, WasmRuntime};
use serde::Serialize;
use tide::{Request, StatusCode};

#[derive(Serialize)]
struct RuntimeResponse {
    #[serde(rename(serialize = "containerId"))]
    container_id: String,
    port: i32,
}

pub async fn destroy(mut req: Request<impl WasmRuntime>) -> tide::Result<StatusCode> {
    let container_id = req.body_string().await?;

    println!("Removing container with id {}", container_id);

    req.state().destroy(&container_id);

    Ok(StatusCode::Ok)
}

pub async fn init(mut req: Request<impl WasmRuntime>) -> tide::Result<StatusCode> {
    let activation_init = req.body_json().await;

    if let Err(err) = &activation_init {
        println!("/init err: {:?}", err);
    }

    let activation_init: ActivationInit = activation_init?;

    println!("/init {}", activation_init.value.name);

    let container_id = req.param("container_id").unwrap().to_owned();

    println!("Initializing container with id {}", container_id);

    let runtime = req.state();

    let module_bytes = util::b64_decode(activation_init.value.code)?;

    let module = util::unzip(module_bytes)?;

    runtime.initialize(container_id, activation_init.value.annotations, module)?;

    Ok(StatusCode::Ok)
}

pub async fn run(
    mut req: Request<impl WasmRuntime + Send + Sync + 'static>,
) -> tide::Result<serde_json::Value> {
    let activation_context: ActivationContext = req.body_json().await?;

    println!("/run {}", activation_context.action_name);

    println!(
        "Running container with id {}",
        req.param("container_id").unwrap()
    );

    // Create a cheap clone of the runtime that can be moved onto another thread
    let runtime = req.state().clone();

    let result = task::spawn_blocking(move || {
        runtime.run(req.param("container_id").unwrap(), activation_context.value)
    })
    .await;

    println!("Wasm Execution returned {:?}", result);

    let response = ActivationResponse::new(result?);

    Ok(serde_json::to_value(response).unwrap())
}
