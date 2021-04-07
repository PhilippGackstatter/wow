#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(not(feature = "wasm"))]
ow_wasm_action::json_args!(func);

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    #[cfg(feature = "random")]
    let random = rand::random::<u64>();

    Ok(serde_json::json!({ "random": random }))
}
