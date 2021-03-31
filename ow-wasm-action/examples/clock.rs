#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(feature = "bin")]
ow_wasm_action::json_args!(func);

use std::time::Instant;

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let now = Instant::now();

    let elapsed = now.elapsed().as_nanos() as usize;

    Ok(serde_json::json!({ "elapsed": elapsed }))
}
