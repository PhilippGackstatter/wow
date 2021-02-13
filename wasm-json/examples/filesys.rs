use std::fs::File;
use std::io::prelude::*;

#[cfg(feature = "wasm")]
wasm_json::pass_json!(func);

#[cfg(feature = "bin")]
wasm_json::json_args!(func);

// Needs to be created with --annotation dir "/tmp/filesys"

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    {
        let mut file = File::create("/tmp/filesys/test.txt").unwrap();

        file.write_all(b"Hello, Wasm.").unwrap();
    }
    {
        let mut file = File::open("/tmp/filesys/test.txt").unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        Ok(serde_json::json!({ "content": contents }))
    }
}
