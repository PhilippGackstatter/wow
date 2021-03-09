use std::io::Write;

#[cfg(feature = "wasm")]
wasm_json::pass_json!(func);

#[cfg(feature = "bin")]
wasm_json::json_args!(func);

// Needs to be created with --annotation dir "/tmp/filesys"

const PATH: &'static str = "/tmp/filesys/test.txt";

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    // let generation_time = std::time::Instant::now();
    let write_read_time = std::time::Instant::now();

    let mut file = std::fs::File::create(PATH).unwrap();

    file.write_all(b"Hello, Wasm.").unwrap();

    let _read_bytes = std::fs::read(PATH)?;

    Ok(serde_json::json!({
        "success": true,
        "write_read_time": write_read_time.elapsed().as_millis() as u64,
    }))
}
