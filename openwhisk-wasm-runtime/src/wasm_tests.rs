#[cfg(test)]
mod runtime_tests {

    use std::fs::read;

    use crate::types::{ActionCapabilities, WasmRuntime};

    #[cfg(test)]
    pub fn execute_precompiled_wasm(
        module_bytes: Vec<u8>,
        capabilities: ActionCapabilities,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, serde_json::Value> {
        let module_bytes: Vec<u8> = base64::decode(module_bytes).unwrap();

        #[cfg(feature = "wasmtime_rt")]
        let runtime = crate::wasmtime::Wasmtime::default();

        #[cfg(feature = "wasmer_rt")]
        let runtime = crate::wasmer::Wasmer::default();

        runtime
            .initialize_action(
                "action_name".to_owned(),
                capabilities,
                module_bytes.to_vec(),
            )
            .unwrap();

        let result = runtime.execute("action_name", input).unwrap();

        result
    }

    fn get_module_bytes(path: &str) -> Vec<u8> {
        let mut path = path.to_owned();

        #[cfg(feature = "wasmtime_rt")]
        path.push_str(".wasmtime");

        #[cfg(feature = "wasmer_rt")]
        path.push_str(".wasmer");

        let contents = read(path).unwrap();
        contents
    }

    #[test]
    fn test_can_call_precompiled_add() {
        let wasm_bytes = get_module_bytes("../target/wasm32-wasi/release/examples/add");

        let capabilities = ActionCapabilities::default();

        let res = execute_precompiled_wasm(
            wasm_bytes,
            capabilities,
            serde_json::json!({"param1": 5, "param2": 4}),
        )
        .unwrap();

        assert_eq!(res.get("result").unwrap().as_u64().unwrap(), 9);
    }

    #[test]
    fn test_add_error_is_correctly_returned() {
        let wasm_bytes = get_module_bytes("../target/wasm32-wasi/release/examples/add");

        let capabilities = ActionCapabilities::default();

        let res =
            execute_precompiled_wasm(wasm_bytes, capabilities, serde_json::json!({"param1": 5}))
                .unwrap_err();

        assert_eq!(
            res.get("error").unwrap().as_str().unwrap(),
            "Expected param2."
        );
    }

    #[test]
    fn test_can_execute_wasm32_wasi_clock_module() {
        let wasm_bytes = get_module_bytes("../target/wasm32-wasi/release/examples/clock");

        let capabilities = ActionCapabilities::default();

        let res =
            execute_precompiled_wasm(wasm_bytes, capabilities, serde_json::json!({})).unwrap();

        assert!(res.get("elapsed").unwrap().as_u64().unwrap() > 0)
    }

    #[test]
    fn test_can_execute_wasm32_wasi_random_module() {
        let wasm_bytes = get_module_bytes("../target/wasm32-wasi/release/examples/random");

        let capabilities = ActionCapabilities::default();

        let res =
            execute_precompiled_wasm(wasm_bytes, capabilities, serde_json::json!({})).unwrap();

        let rand = res.get("random").unwrap().as_u64().unwrap();
        assert!(rand > 0)
    }

    #[test]
    fn test_can_execute_wasm32_wasi_filesystem_module() {
        let wasm_bytes = get_module_bytes("../target/wasm32-wasi/release/examples/filesys");

        let capabilities = ActionCapabilities {
            dir: Some("/tmp/filesys".into()),
        };

        let res =
            execute_precompiled_wasm(wasm_bytes, capabilities, serde_json::json!({})).unwrap();

        assert_eq!(
            res.get("content").unwrap().as_str().unwrap(),
            "Hello, Wasm."
        );
    }
}
