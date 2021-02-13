#[cfg(feature = "wasm")]
wasm_json::pass_json!(func);

#[cfg(feature = "bin")]
wasm_json::json_args!(func);

use std::time::Instant;

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let now = Instant::now();

    let elapsed = now.elapsed().as_nanos() as usize;

    Ok(serde_json::json!({ "elapsed": elapsed }))
}
