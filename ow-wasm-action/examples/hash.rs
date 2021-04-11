#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(not(feature = "wasm"))]
ow_wasm_action::json_args!(func);

pub fn func(json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let iterations = json.get("iterations").unwrap().as_i64().unwrap() as usize;
    let input = json.get("input").unwrap().as_str().unwrap();

    #[cfg(feature = "hash")]
    let hash = {
        let mut prev_output;
        let mut hash = input.as_bytes();

        for _ in 0..iterations {
            prev_output = blake3::hash(hash);
            hash = prev_output.as_bytes()
        }

        hash.to_vec()
    };

    Ok(serde_json::json!({ "hash": format!("{:x?}", hash) }))
}
