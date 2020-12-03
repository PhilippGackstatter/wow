wasm_json::pass_json!(func);

fn func(json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let param = json
        .get("param")
        .ok_or(anyhow::anyhow!("Expected param"))?
        .as_i64()
        .ok_or(anyhow::anyhow!("Expected param to be i64"))?;

    println!("Param is {}", param);

    Ok(serde_json::json!({ "result": param }))
}

fn main() {}
