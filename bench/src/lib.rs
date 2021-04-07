pub mod concurrency;

use std::collections::HashMap;

use async_std::{eprintln, println};
use futures::{stream::FuturesUnordered, Future, StreamExt};
use ow_common::{ActionCapabilities, ActivationContext, ActivationInit, ActivationInitInner};
use serde_json::Value;

pub const SERVER_URI: &'static str = "http://172.17.0.1:3233";

pub async fn init(
    action_name: &str,
    file_name: String,
    capabilities: ActionCapabilities,
) -> anyhow::Result<()> {
    let code = async_std::fs::read_to_string(file_name).await?;

    let mut env = HashMap::new();
    env.insert("__OW_ACTION_NAME".to_owned(), action_name.to_owned());

    let activation_init = ActivationInit {
        value: ActivationInitInner {
            name: action_name.to_owned(),
            main: "doesntmatter".to_owned(),
            code,
            binary: false,
            env,
            annotations: capabilities,
        },
    };

    let body = surf::Body::from_json(&activation_init).unwrap();
    let res = surf::post(SERVER_URI.to_owned() + "/init")
        .body(body)
        .await
        .unwrap();

    if !res.status().is_success() {
        anyhow::bail!("Status code: {:?}", res.status());
    }

    Ok(())
}

pub fn make_activation_ctx(action_name: &str, parameters: Value) -> Value {
    let activation_ctx = ActivationContext {
        value: parameters,
        namespace: "".to_owned(),
        action_name: action_name.to_owned(),
        api_host: None,
        api_key: None,
        activation_id: "".to_owned(),
        transaction_id: "".to_owned(),
        deadline: 0,
    };

    let mut activation_ctx_json = serde_json::to_value(activation_ctx).unwrap();

    // OW sends this as a str, but we've defined it as a u64
    if let serde_json::Value::Object(map) = &mut activation_ctx_json {
        map.insert(
            "deadline".to_owned(),
            serde_json::Value::String("0".to_owned()),
        );
    }

    activation_ctx_json
}

pub async fn benchmark(num_requests: usize, parameters: Value) -> Vec<Value> {
    let mut requests = Vec::with_capacity(num_requests);

    let activation_ctx = ActivationContext {
        value: parameters,
        namespace: "".to_owned(),
        action_name: "bench".to_owned(),
        api_host: None,
        api_key: None,
        activation_id: "".to_owned(),
        transaction_id: "".to_owned(),
        deadline: 0,
    };

    let mut activation_ctx_json = serde_json::to_value(activation_ctx).unwrap();

    // OW sends this as a str, but we've defined it as a u64
    if let serde_json::Value::Object(map) = &mut activation_ctx_json {
        map.insert(
            "deadline".to_owned(),
            serde_json::Value::String("0".to_owned()),
        );
    }

    for _ in 0..num_requests {
        let body = surf::Body::from_json(&activation_ctx_json).unwrap();

        let request = surf::post(SERVER_URI.to_owned() + "/run")
            .body(body)
            .recv_json::<serde_json::Value>();

        requests.push(request);
    }

    let mut futures = requests.into_iter().collect::<FuturesUnordered<_>>();

    let mut responses = Vec::with_capacity(num_requests);

    let before = std::time::Instant::now();

    while let Some(res) = futures.next().await {
        match res {
            Ok(json) => {
                println!("Request completed").await;
                responses.push(json);
            }
            Err(err) => {
                eprintln!("Recv Error {:?}", err).await;
            }
        }
    }

    println!("Time: {}ms", before.elapsed().as_millis()).await;

    return responses;
}

pub fn get_first_arg() -> anyhow::Result<String> {
    let mut args = std::env::args();

    // Skip name
    args.next();

    args.next().ok_or_else(|| {
        anyhow::anyhow!("Expected first argument to be a path to a Wasm or precompiled bianry")
    })
}

pub fn generate_requests(
    action_name: &str,
    num_requests: usize,
    parameters: Value,
) -> Vec<impl Future<Output = Result<Value, surf::Error>>> {
    let activation_ctx_json = make_activation_ctx(action_name, parameters);

    let mut requests = vec![];
    for _ in 0..num_requests {
        let body = surf::Body::from_json(&activation_ctx_json).unwrap();

        let request = surf::post(SERVER_URI.to_owned() + "/run")
            .body(body)
            .recv_json::<serde_json::Value>();

        requests.push(request);
    }

    requests
}

pub async fn execute_requests(
    requests: Vec<impl Future<Output = Result<Value, surf::Error>>>,
) -> (u64, Vec<Value>) {
    let mut responses = Vec::with_capacity(requests.len());

    let mut futures = requests.into_iter().collect::<FuturesUnordered<_>>();

    let before = std::time::Instant::now();

    while let Some(res) = futures.next().await {
        match res {
            Ok(json) => {
                eprintln!("Request completed").await;
                responses.push(json);
            }
            Err(err) => {
                eprintln!("Recv Error {:?}", err).await;
            }
        }
    }

    (before.elapsed().as_millis() as u64, responses)
}
