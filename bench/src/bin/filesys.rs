use bench::{benchmark, get_first_arg, init};
use openwhisk_wasm_runtime::types::ActionCapabilities;
use serde_json::json;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let file_name = get_first_arg()?;

    let capabilities = ActionCapabilities {
        dir: Some("/tmp/filesys".into()),
        ..Default::default()
    };

    init(file_name, capabilities).await?;

    generate_test_file().await;

    let num_requests = num_cpus::get() * 2;

    let responses = benchmark(num_requests, json!({})).await;

    for response in responses.iter() {
        let res = response.get("result").unwrap();
        println!(
            "filesys write/read time: {}ms",
            res.get("write_read_time").unwrap(),
        );
    }

    Ok(())
}

async fn generate_test_file() {
    const NUM_BYTES: usize = 30_000_000;

    let mut random_bytes: Vec<u8> = Vec::with_capacity(NUM_BYTES);

    for i in 0..NUM_BYTES {
        random_bytes.push((i % 255) as u8);
    }

    async_std::fs::write("/tmp/filesys/test.txt", &random_bytes)
        .await
        .unwrap();
}
