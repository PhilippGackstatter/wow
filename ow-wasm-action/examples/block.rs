#[cfg(feature = "wasm")]
ow_wasm_action::pass_json!(func);

#[cfg(not(feature = "wasm"))]
ow_wasm_action::json_args!(func);

#[link(wasm_import_module = "http")]
extern "C" {
    fn get() -> i32;
}

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let t = std::time::Instant::now();
    //std::thread::sleep(std::time::Duration::new(0, 300_000_000));
    let _ = unsafe { get() };
    Ok(serde_json::json!({
        "sleep": format!("{}", t.elapsed().as_millis())
    }))
}
