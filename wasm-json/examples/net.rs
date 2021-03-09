#[cfg(feature = "wasm")]
wasm_json::pass_json!(func);

#[cfg(feature = "bin")]
wasm_json::json_args!(func);

#[link(wasm_import_module = "http")]
extern {
    fn get() -> i32;
}

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let request_time = std::time::Instant::now();

    let _req = unsafe { get() };

    Ok(serde_json::json!({
        "request_time": request_time.elapsed().as_millis() as u64,
    }))
}
