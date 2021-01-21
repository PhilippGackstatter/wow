use crate::types::WasmAction;

/*
macro_rules! test_runtime {
    ($mod_name:ident, $exec_wasm:path) => {

    #[cfg(test)]
    mod $mod_name {
        use std::time::Instant;

        use crate::types::{ActionCapabilities, WasmAction};


        #[test]
        fn test_can_call_simple_add() {
            let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/add.wasm");

            let wasm_action = WasmAction {
                code: wasm_bytes.to_vec(),
                capabilities: ActionCapabilities::default(),
            };

            let res = $exec_wasm(serde_json::json!({"param1": 5, "param2": 4}), &wasm_action)
                .unwrap()
                .unwrap();

            assert_eq!(
                res,
                serde_json::json!({
                    "result": 9
                })
            );
        }

        #[test]
        fn test_add_error_is_correctly_returned() {
            let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/add.wasm");

            let wasm_action = WasmAction {
                code: wasm_bytes.to_vec(),
                capabilities: ActionCapabilities::default(),
            };

            let res = $exec_wasm(serde_json::json!({"param1": 5}), &wasm_action)
                .unwrap()
                .unwrap_err();

            assert_eq!(
                res,
                serde_json::json!({
                    "error": "Expected param2."
                })
            );
        }

        #[test]
        fn test_can_execute_wasm32_wasi_module() {
            let wasm_bytes =
                include_bytes!("../../target/wasm32-wasi/release/examples/println-wasi.wasm");

            let wasm_action = WasmAction {
                code: wasm_bytes.to_vec(),
                capabilities: ActionCapabilities::default(),
            };

            let timestamp = Instant::now();

            let res = $exec_wasm(serde_json::json!({"param": 5}), &wasm_action)
                .unwrap()
                .unwrap();

            println!("execute wasm took {}ms", timestamp.elapsed().as_millis());

            assert_eq!(
                res,
                serde_json::json!({
                    "result": 5
                })
            );
        }

        #[test]
        fn test_can_execute_wasm32_wasi_clock_module() {
            let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/clock.wasm");

            let wasm_action = WasmAction {
                code: wasm_bytes.to_vec(),
                capabilities: ActionCapabilities::default(),
            };

            let res = $exec_wasm(serde_json::json!({}), &wasm_action)
                .unwrap()
                .unwrap();

            assert!(res.get("elapsed").unwrap().as_u64().unwrap() > 0)
        }

        #[test]
        fn test_can_execute_wasm32_wasi_random_module() {
            let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/random.wasm");

            let wasm_action = WasmAction {
                code: wasm_bytes.to_vec(),
                capabilities: ActionCapabilities::default(),
            };

            let res = $exec_wasm(serde_json::json!({}), &wasm_action)
                .unwrap()
                .unwrap();

            let rand = res.get("random").unwrap().as_u64().unwrap();
            assert!(rand > 0)
        }

        #[test]
        fn test_can_execute_wasm32_wasi_filesystem_module() {
            let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/filesys.wasm");

            let wasm_action = WasmAction {
                code: wasm_bytes.to_vec(),
                capabilities: ActionCapabilities {
                    dir: Some("/tmp/filesys".into()),
                },
            };

            let res = $exec_wasm(serde_json::json!({}), &wasm_action)
                .unwrap()
                .unwrap();

            assert_eq!(
                res,
                serde_json::json!({
                    "content": "Hello, Wasm."
                })
            );
        }
    }
    }
}
*/
// #[cfg(all(test, feature = "wasmer_rt"))]
// test_runtime!(wasmer_tests, super::super::wasmer::execute_wasm);
// test_runtime!(wasmtime_tests, super::super::wasmtime::execute_wasm);

// Runtime-specific tests

pub fn execute_precompiled_wasm(
    wasm_action: WasmAction,
    input: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    #[cfg(all(feature = "wasmer_rt"))]
    let result = super::wasmer::execute_wasm(input, &wasm_action).unwrap();

    #[cfg(all(feature = "wasmtime_rt"))]
    let result = super::wasmtime::execute_wasm(input, &wasm_action).unwrap();

    result
}

#[cfg(all(test, feature = "wasmer_rt"))]
mod wasmer_specific_tests {

    use crate::types::{ActionCapabilities, WasmAction};

    use super::execute_precompiled_wasm;

    #[test]
    fn wasmer_test_can_call_precompiled_add() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/add.wasmer");

        let wasm_action = WasmAction {
            code: wasm_bytes.to_vec(),
            capabilities: ActionCapabilities::default(),
        };

        let res =
            execute_precompiled_wasm(wasm_action, serde_json::json!({"param1": 5, "param2": 4}))
                .unwrap();

        assert_eq!(
            res,
            serde_json::json!({
                "result": 9
            })
        );
    }

    #[test]
    fn test_add_error_is_correctly_returned() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/add.wasmer");

        let wasm_action = WasmAction {
            code: wasm_bytes.to_vec(),
            capabilities: ActionCapabilities::default(),
        };

        let res =
            execute_precompiled_wasm(wasm_action, serde_json::json!({"param1": 5})).unwrap_err();

        assert_eq!(
            res,
            serde_json::json!({
                "error": "Expected param2."
            })
        );
    }

    #[test]
    fn test_can_execute_wasm32_wasi_clock_module() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/clock.wasmer");

        let wasm_action = WasmAction {
            code: wasm_bytes.to_vec(),
            capabilities: ActionCapabilities::default(),
        };

        let res = execute_precompiled_wasm(wasm_action, serde_json::json!({})).unwrap();

        assert!(res.get("elapsed").unwrap().as_u64().unwrap() > 0)
    }

    #[test]
    fn test_can_execute_wasm32_wasi_random_module() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/random.wasmer");

        let wasm_action = WasmAction {
            code: wasm_bytes.to_vec(),
            capabilities: ActionCapabilities::default(),
        };

        let res = execute_precompiled_wasm(wasm_action, serde_json::json!({})).unwrap();

        let rand = res.get("random").unwrap().as_u64().unwrap();
        assert!(rand > 0)
    }

    #[test]
    fn test_can_execute_wasm32_wasi_filesystem_module() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/filesys.wasmer");

        let wasm_action = WasmAction {
            code: wasm_bytes.to_vec(),
            capabilities: ActionCapabilities {
                dir: Some("/tmp/filesys".into()),
            },
        };

        let res = execute_precompiled_wasm(wasm_action, serde_json::json!({})).unwrap();

        assert_eq!(
            res,
            serde_json::json!({
                "content": "Hello, Wasm."
            })
        );
    }
}

#[cfg(all(test, feature = "wasmtime_rt"))]
mod wasmtime_specific_tests {

    use std::time::Instant;

    use crate::types::{ActionCapabilities, WasmAction};

    use super::super::wasmtime;

    #[test]
    fn wasmtime_test_can_call_precompiled_add() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/add.wasmtime");

        let wasm_action = WasmAction {
            code: wasm_bytes.to_vec(),
            capabilities: ActionCapabilities::default(),
        };

        let timestamp = Instant::now();

        let res =
            wasmtime::execute_wasm(serde_json::json!({"param1": 5, "param2": 4}), &wasm_action)
                .unwrap()
                .unwrap();

        println!("execute wasm took {}ms", timestamp.elapsed().as_millis());

        assert_eq!(
            res,
            serde_json::json!({
                "result": 9
            })
        );
    }
}
