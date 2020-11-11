use crate::types::{ActivationContext, ActivationInit, ActivationResponse};
use serde::Serialize;
use tide::Request;

#[derive(Serialize)]
struct RuntimeResponse {
    #[serde(rename(serialize = "containerId"))]
    container_id: String,
    port: i32,
}

pub async fn start(mut _req: Request<()>) -> tide::Result<serde_json::Value> {
    // println!("start called with {:#?}", req.body_string().await);

    let resp = RuntimeResponse {
        container_id: "1".into(),
        port: 9000,
    };

    Ok(serde_json::to_value(resp).unwrap())
}

pub async fn init(mut req: Request<()>) -> tide::Result<tide::StatusCode> {
    let activation_init: ActivationInit = req.body_json().await?;

    println!("/init {:#?}", activation_init);

    Ok(tide::StatusCode::Ok)
}

pub async fn run(mut req: Request<()>) -> tide::Result<serde_json::Value> {
    let activation_context: tide::Result<ActivationContext> = req.body_json().await;

    println!("/run {:#?}", activation_context.unwrap());

    Ok(serde_json::to_value(ActivationResponse::default()).unwrap())
}
