use std::ptr::slice_from_raw_parts;

use anyhow::anyhow;
use wasmtime::*;
use wasmtime_wasi::{Wasi, WasiCtxBuilder};

pub fn execute_wasm(
    parameters: serde_json::Value,
    wasm_bytes: &Vec<u8>,
) -> Result<Result<serde_json::Value, serde_json::Value>, anyhow::Error> {
    let engine = Engine::default();

    let store = Store::new(&engine);

    let mut linker = Linker::new(&store);

    let ctx = WasiCtxBuilder::new()
        .inherit_stdout()
        .inherit_stderr()
        .build()?;
    let wasi = Wasi::new(&store, ctx);
    wasi.add_to_linker(&mut linker)?;

    let module = Module::new(store.engine(), wasm_bytes)?;

    let instance = linker.instantiate(&module)?;

    let main = instance
        .get_func("wrapped_func")
        .expect("The module did not export the expected `wrapped_func` function");

    let main = main.get1::<i32, i32>()?;

    let len = pass_string_arg(&instance, &parameters)?;

    let len: i32 = main(len as i32)?;

    Ok(get_return_value(&instance, len as usize))
}

fn pass_string_arg(instance: &Instance, json: &serde_json::Value) -> Result<usize, anyhow::Error> {
    let json_bytes = serde_json::to_vec(json).unwrap();

    let wasm_memory_buffer_allocate_space = instance
        .get_func("wasm_memory_buffer_allocate_space")
        .ok_or_else(|| {
            anyhow!("Expected the module to export `wasm_memory_buffer_allocate_space`")
        })?
        .get1::<i32, ()>()?;

    wasm_memory_buffer_allocate_space(json_bytes.len() as i32)?;

    let memory_buffer_func = instance
        .get_func("get_wasm_memory_buffer_pointer")
        .ok_or_else(|| anyhow!("Expected the module to export `get_wasm_memory_buffer_pointer`"))?
        .get0::<i32>()?;

    let memory_buffer_offset = memory_buffer_func().unwrap();

    let memory_base_ptr = instance
        .get_memory("memory")
        .ok_or_else(|| anyhow!("Expected the module to export a memory named `memory`"))?
        .data_ptr();

    unsafe {
        memory_base_ptr
            .offset(memory_buffer_offset as isize)
            .copy_from_nonoverlapping(json_bytes.as_ptr(), json_bytes.len());
    }

    Ok(json_bytes.len())
}

fn get_return_value(
    instance: &Instance,
    len: usize,
) -> Result<serde_json::Value, serde_json::Value> {
    // We can unwrap here, because we handled these exact errors earlier
    // so we wouldn't reach this point if the functions wouldn't exist.
    let memory_ptr_func = instance
        .get_func("get_wasm_memory_buffer_pointer")
        .unwrap()
        .get0::<i32>()
        .unwrap();

    let memory_ptr_offset = memory_ptr_func().unwrap();

    let memory_base_ptr = instance.get_memory("memory").unwrap().data_ptr();

    let wasm_mem_slice = slice_from_raw_parts(
        unsafe { memory_base_ptr.offset(memory_ptr_offset as isize) as *const u8 },
        len,
    );

    serde_json::from_slice(unsafe { &*wasm_mem_slice }).unwrap()
}

#[cfg(test)]
mod tests {
    use super::execute_wasm;

    #[test]
    fn test_can_call_simple_add() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/add.wasm");

        let res = execute_wasm(
            serde_json::json!({"param1": 5, "param2": 4}),
            &wasm_bytes.to_vec(),
        )
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

        let res = execute_wasm(serde_json::json!({"param1": 5}), &wasm_bytes.to_vec())
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

        let res = execute_wasm(serde_json::json!({"param": 5}), &wasm_bytes.to_vec())
            .unwrap()
            .unwrap();

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

        let res = execute_wasm(serde_json::json!({}), &wasm_bytes.to_vec())
            .unwrap()
            .unwrap();

        assert!(res.get("elapsed").unwrap().as_u64().unwrap() > 0)
    }

    #[test]
    fn test_can_execute_wasm32_wasi_random_module() {
        let wasm_bytes = include_bytes!("../../target/wasm32-wasi/release/examples/random.wasm");

        let res = execute_wasm(serde_json::json!({}), &wasm_bytes.to_vec())
            .unwrap()
            .unwrap();

        let rand = res.get("random").unwrap().as_u64().unwrap();
        assert!(rand > 0)
    }
}
