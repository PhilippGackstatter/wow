#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(feature = "bin")]
ow_wasm_action::json_args!(func);

pub fn func(json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let iterations = json.get("iterations").unwrap().as_i64().unwrap() as usize;
    let input = json.get("input").unwrap().as_str().unwrap();

    #[cfg(feature = "hash")]
    let hash = {
        use sha2::Digest;
        let mut prev_output;
        let mut hash = input.as_bytes();

        for _ in 0..iterations {
            prev_output = sha2::Sha512::digest(hash);
            hash = prev_output.as_slice()
        }

        hash.to_vec()
    };

    Ok(serde_json::json!({ "random": format!("{:x?}", hash) }))
}
