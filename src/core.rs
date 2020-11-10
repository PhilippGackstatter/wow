use serde::Serialize;
use tide::Request;

#[derive(Serialize)]
struct RuntimeResponse {
    containerId: String,
    port: i32,
}

pub async fn start(mut req: Request<()>) -> tide::Result<serde_json::Value> {
    println!("start called with {:#?}", req.body_string().await);

    let resp = RuntimeResponse {
        containerId: "1".into(),
        port: 9000,
    };

    Ok(serde_json::to_value(resp).unwrap())
}

pub async fn init(mut req: Request<()>) -> tide::Result<tide::StatusCode> {
    println!("init called with {:#?}", req.body_string().await);

    Ok(tide::StatusCode::Ok)
}

pub async fn run(mut req: Request<()>) -> tide::Result<tide::StatusCode> {
    println!("run called with {:#?}", req.body_string().await);

    Ok(tide::StatusCode::Ok)
}
