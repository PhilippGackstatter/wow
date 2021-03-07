use bench::{execute_requests, generate_requests, init};
use rand::prelude::SliceRandom;
use serde_json::{json, Value};

const SAMPLE_SIZE: usize = 50;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let net_file = "../target/wasm32-wasi/release/examples/net.wasmtime";
    let prime_file = "../target/wasm32-wasi/release/examples/prime.wasmtime";

    init(
        "net",
        net_file.to_owned(),
        openwhisk_wasm_runtime::types::ActionCapabilities {
            net_access: true,
            ..Default::default()
        },
    )
    .await?;

    init("prime", prime_file.to_owned(), Default::default()).await?;

    let num_requests = num_cpus::get() * 4;
    let mut summary = Vec::with_capacity(num_requests);
    let mut exec_times = Vec::with_capacity(SAMPLE_SIZE);

    for _ in 0..SAMPLE_SIZE {
        let (exec_time, sample) = take_sample(num_requests).await;
        exec_times.push(exec_time);
        summary.append(&mut summarize(sample));
        async_std::task::sleep(std::time::Duration::new(0, 50_000_000u32)).await;
    }

    let result = json!({
        "times": exec_times,
        "requests": summary,
    });

    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

async fn take_sample(num_requests: usize) -> (u64, Vec<Value>) {
    let mut requests = generate_requests("prime", num_requests / 2, json!({}));
    requests.append(&mut generate_requests("net", num_requests / 2, json!({})));

    let mut rng = rand::thread_rng();
    requests.shuffle(&mut rng);

    execute_requests(requests).await
}

fn summarize(responses: Vec<Value>) -> Vec<Value> {
    let mut summary = vec![];

    for response in responses {
        let result = response.get("result").unwrap();
        let duration = result.get("exit_at").unwrap().as_f64().unwrap()
            - result.get("entry_at").unwrap().as_f64().unwrap();
        if let Some(_) = result.get("prime") {
            summary.push(json!({
                "duration": duration,
                "type": "prime"
            }));
        } else {
            summary.push(json!({
                "duration": duration,
                "type": "net"
            }));
        }
    }

    summary
}
