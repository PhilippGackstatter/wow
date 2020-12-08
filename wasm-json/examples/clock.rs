use std::time::Instant;

wasm_json::pass_json!(func);

fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let now = Instant::now();

    let elapsed = now.elapsed().as_nanos() as usize;

    Ok(serde_json::json!({ "elapsed": elapsed }))
}
