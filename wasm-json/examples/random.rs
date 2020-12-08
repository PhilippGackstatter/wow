wasm_json::pass_json!(func);

fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    Ok(serde_json::json!({ "random": rand::random::<u64>() }))
}
