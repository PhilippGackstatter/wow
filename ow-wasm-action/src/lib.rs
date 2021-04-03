use serde_json::Value;

// Expects to be called on a function with the signature
// (json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error>
#[macro_export]
macro_rules! pass_json {
    ($($t:ident)*) => ($(

        static mut MEMORY_BUFFER: Vec<u8> = Vec::new();

        // Function to return a pointer to our buffer
        // in Wasm memory
        #[no_mangle]
        pub fn get_wasm_memory_buffer_pointer() -> *mut u8 {
            unsafe { MEMORY_BUFFER.as_mut_ptr() }
        }

        #[no_mangle]
        pub fn wasm_memory_buffer_allocate_space(num_elems: usize) {
            unsafe {
                MEMORY_BUFFER.reserve(num_elems);
                // This is technically unsafe, but as the host will write
                // exactly `num_elems` into this buffer after this function
                // completes, it's fine. It simply saves another invocation.
                MEMORY_BUFFER.set_len(num_elems);
            }
        }

        #[no_mangle]
        pub fn get_wasm_memory_buffer_len() -> usize {
            unsafe { MEMORY_BUFFER.len() }
        }

        pub fn main() {
            let args: Vec<String> = std::env::args().collect();

            // wasmer automatically passes the program name as the first arg
            // while wasmtime does not force that, so we just take the last one
            if args.len() > 2 {
                eprintln!("Expected 1 or 2 arguments, got {}: {:?}", args.len(), args);
                return;
            }

            let len: usize = args[args.len() - 1].parse().unwrap();
            let json = deserialize_slice(len);

            let result = $crate::wrap_timestamped(json, $t);

            let result = result.map_err(|err: anyhow::Error| {
                let err_string = err.to_string();

                serde_json::json!({ "error": err_string })
            });

            let mut result = serde_json::to_vec(&result).expect("Could not serialize result.");

            serialize_vec(&mut result);
        }

        fn serialize_vec(vec: &mut Vec<u8>) {
            unsafe {
                MEMORY_BUFFER.clear();
                MEMORY_BUFFER.append(vec);
            }
        }

        fn deserialize_slice(len: usize) -> serde_json::Value {
            let slice = unsafe { &MEMORY_BUFFER[..len] };
            serde_json::from_slice(slice).expect("Could not deserialize slice")
        }




    )*)
}

pub fn wrap_timestamped<F>(
    _json: serde_json::Value,
    func: F,
) -> Result<serde_json::Value, anyhow::Error>
where
    F: FnOnce(serde_json::Value) -> Result<serde_json::Value, anyhow::Error>,
{
    let entry_at = timestamp();

    let result = func(_json);

    let exit_at = timestamp();

    result.map(|json| {
        if let Value::Object(mut map) = json {
            map.insert("entry_at".to_owned(), serde_json::json!(entry_at));
            map.insert("exit_at".to_owned(), serde_json::json!(exit_at));
            Value::Object(map)
        } else {
            panic!("Expected a JSON Object as result.");
        }
    })
}

fn timestamp() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards.")
        .as_secs_f64()
}

#[macro_export]
macro_rules! json_args {
    ($($t:ident)*) => ($(

    pub fn main() -> anyhow::Result<()> {
        let json: serde_json::Value = match std::env::args().nth(1) {
            Some(json_str) => Ok(serde_json::from_str(&json_str)?),
            None => {
                println!(
                    "{}",
                    serde_json::json!({
                        "message": "No input provided",
                    })
                );

                Err(anyhow::anyhow!(""))
            }
        }?;

        let result: anyhow::Result<serde_json::Value> = $crate::wrap_timestamped(json, $t);

        match result {
            Ok(success) => {
                println!("{}", success);
            }
            Err(err) => {
                println!("{}", serde_json::json!({
                    "message": err.to_string(),
                }));
            }
        }

        Ok(())
    }
    )*)
}
