#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(not(feature = "wasm"))]
ow_wasm_action::json_args!(func);

fn func(json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let param1 = json
        .get("param1")
        .ok_or_else(|| anyhow::anyhow!("Expected param1 to be present"))?
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Expected param1 to be an i64"))?;

    let param2 = json
        .get("param2")
        .ok_or_else(|| anyhow::anyhow!("Expected param2 to be present"))?
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Expected param2 to be an i64"))?;

    Ok(serde_json::json!({ "result": param1 + param2 }))
}
