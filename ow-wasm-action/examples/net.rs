#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(feature = "bin")]
ow_wasm_action::json_args!(func);

// #[link(wasm_import_module = "http")]
// extern "C" {
//     fn get() -> i32;
// }

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let request_time = std::time::Instant::now();

    let _req = std::thread::sleep(std::time::Duration::new(0, 300_000_000)); //unsafe { get() };

    Ok(serde_json::json!({
        "request_time": request_time.elapsed().as_millis() as u64,
    }))
}
