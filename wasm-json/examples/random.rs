#[cfg(feature = "wasm")]
wasm_json::pass_json!(func);

#[cfg(feature = "bin")]
wasm_json::json_args!(func);

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    Ok(serde_json::json!({ "random": rand::random::<u64>() }))
}
