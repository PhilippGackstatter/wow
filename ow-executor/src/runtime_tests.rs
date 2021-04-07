#[cfg(test)]
mod runtime_tests {

    use std::fs::read;

    use ow_common::{ActionCapabilities, WasmRuntime};

    #[cfg(test)]
    pub fn execute_precompiled_wasm(
        module_bytes: Vec<u8>,
        capabilities: ActionCapabilities,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, serde_json::Value> {
        #[cfg(feature = "wasmtime_rt")]
        let runtime = ow_wasmtime::Wasmtime::default();

        #[cfg(feature = "wasmer_rt")]
        let runtime = ow_wasmer::Wasmer::default();

        #[cfg(feature = "wamr_rt")]
        let runtime = ow_wamr::Wamr::default();

        runtime
            .initialize("action_name".to_owned(), capabilities, module_bytes)
            .unwrap();

        let result = runtime.run("action_name", input).unwrap();

        result
    }

    fn get_module_bytes(path: &str) -> Vec<u8> {
        let mut path = path.to_owned();

        #[cfg(feature = "wasmtime_rt")]
        path.push_str(".wasmtime");

        #[cfg(feature = "wasmer_rt")]
        path.push_str(".wasmer");

        #[cfg(feature = "wamr_rt")]
        path.push_str(".wamr");

        let path: std::path::PathBuf = path.into();

        if path.exists() {
            let contents = read(path).unwrap();
            contents
        } else {
            panic!("{:?} does not exist", path);
        }
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
            ..Default::default()
        };

        let res =
            execute_precompiled_wasm(wasm_bytes, capabilities, serde_json::json!({})).unwrap();

        println!("Filesys: {:?}", res);

        assert!(res.get("success").unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_can_execute_http_module() {
        let wasm_bytes = get_module_bytes("../target/wasm32-wasi/release/examples/net");

        let capabilities = ActionCapabilities {
            net_access: Some(true),
            ..Default::default()
        };

        let res =
            execute_precompiled_wasm(wasm_bytes, capabilities, serde_json::json!({})).unwrap();

        let req_time = res.get("request_time").unwrap().as_i64().unwrap();
        assert!(req_time > 1);
    }
}
