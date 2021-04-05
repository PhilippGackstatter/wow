#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(feature = "bin")]
ow_wasm_action::json_args!(func);

pub fn func(json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let iterations = json.get("iterations").unwrap().as_i64().unwrap() as usize;

    #[cfg(feature = "hash")]
    let hash = {
        use sha2::Digest;
        let input = b"BtonIPhlXO0dwNKBVBFBL18jLVVRMSQHUvGqZb3kAIZavaE02MXuSflyfDaK2lBLEGs6qWsF8feFdcWnn76LYIpokehDlfuMiwH4e7MAPCp3yyXnJRe5MDwuYFlBa85yDYRx0yaXvnoho3gqYQ4RpqZm6GiaUTWh35HZVrbTRQCwksuoTvT1Jea9bgwlXPqffwoFnyqF8eTp2J8M2TWp7t7ZOeaQj1TULzEcVCeEJcGjFmevdnAk24dz7l3zJUph".to_vec();
        let mut prev_output;
        let mut hash = input.as_slice();

        for _ in 0..iterations {
            prev_output = sha2::Sha512::digest(hash);
            hash = prev_output.as_slice()
        }

        hash.to_vec()
    };

    Ok(serde_json::json!({ "random": format!("{:x?}", hash) }))
}
