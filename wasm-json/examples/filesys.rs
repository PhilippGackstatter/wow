use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

#[cfg(feature = "wasm")]
wasm_json::pass_json!(func);

#[cfg(feature = "bin")]
wasm_json::json_args!(func);

// Needs to be created with --annotation dir "/tmp/filesys"

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    // let generation_time = std::time::Instant::now();
    let path = format!(
        "/tmp/filesys/{}.txt",
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Time went backwards.")
            .as_secs()
    );

    let write_read_time = std::time::Instant::now();

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)?;

    file.write_all(b"Hello, Wasm.").unwrap();

    let mut res = String::new();
    file.read_to_string(&mut res).unwrap();

    Ok(serde_json::json!({
        "success": true,
        "write_read_time": write_read_time.elapsed().as_millis() as u64,
        "content": res
    }))
}
