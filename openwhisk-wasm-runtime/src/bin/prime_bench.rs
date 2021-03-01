use std::{collections::HashMap, time::Instant};

use async_std::{eprintln, println};
use futures::{stream::FuturesUnordered, StreamExt};
use openwhisk_wasm_runtime::types::{ActivationContext, ActivationInit, ActivationInitInner};

const SERVER_URI: &'static str = "http://127.0.0.1:9000";

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();

    // Skip name
    args.next();

    let file_name = args.next().ok_or_else(|| {
        anyhow::anyhow!("Expected first argument to be a path to a Wasm or precompiled bianry")
    })?;

    println!("Initializing").await;

    init(file_name).await?;

    println!("Benchmarking").await;

    bench().await;

    Ok(())
}

async fn init(file_name: String) -> anyhow::Result<()> {
    let code = async_std::fs::read_to_string(file_name).await?;

    let mut env = HashMap::new();
    env.insert("__OW_ACTION_NAME".to_owned(), "bench".to_owned());

    let activation_init = ActivationInit {
        value: ActivationInitInner {
            name: "bench".to_owned(),
            main: "doesntmatter".to_owned(),
            code,
            binary: false,
            env,
            annotations: Default::default(),
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

async fn bench() {
    let num_requests = num_cpus::get() * 2;

    let mut requests = Vec::with_capacity(num_requests);

    let activation_ctx = ActivationContext {
        value: serde_json::json!({}),
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

    let before = Instant::now();

    while let Some(res) = futures.next().await {
        match res {
            Ok(json) => {
                let result = json.get("result").unwrap();
                let prime = result.get("prime").unwrap();
                assert_eq!(prime, 373587911);
                println!("Calc time: {}", result.get("calc_time").unwrap()).await;
            }
            Err(err) => {
                eprintln!("Recv Error {:?}", err).await;
            }
        }
    }

    println!("Time: {}", before.elapsed().as_millis()).await;
}
