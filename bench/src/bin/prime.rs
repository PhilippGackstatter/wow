use bench::{benchmark, get_first_arg, init};
use serde_json::json;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let file_name = get_first_arg()?;

    init(
        file_name,
        openwhisk_wasm_runtime::types::ActionCapabilities {
            net_access: true,
            ..Default::default()
        },
    )
    .await?;

    let num_requests = num_cpus::get() * 2;

    let responses = benchmark(num_requests, json!({})).await;

    // for response in responses.iter() {
    //     println!(
    //         "response: {}",
    //         response//.get("result").unwrap().get("calc_time").unwrap()
    //     );
    // }

    println!("{}", serde_json::to_string_pretty(&responses)?);

    Ok(())
}
