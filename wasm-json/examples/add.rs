wasm_json::pass_json!(func);

fn func(json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let param1 = json
        .get("param1")
        .ok_or(anyhow::anyhow!("Expected param1."))?
        .as_i64()
        .ok_or(anyhow::anyhow!("Expected param1 to be i64"))?;

    let param2 = json
        .get("param2")
        .ok_or(anyhow::anyhow!("Expected param2."))?
        .as_i64()
        .ok_or(anyhow::anyhow!("Expected param2 to be i64"))?;

    Ok(serde_json::json!({ "result": param1 + param2 }))
}
