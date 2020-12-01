use std::ptr::slice_from_raw_parts;

use anyhow::anyhow;
use wasmtime::*;

pub fn execute_wasm(
    parameters: serde_json::Value,
    wasm_bytes: &Vec<u8>,
) -> Result<Result<serde_json::Value, serde_json::Value>, anyhow::Error> {
    let engine = Engine::default();
    // A `Store` is a sort of "global object" in a sense, but for now it suffices
    // to say that it's generally passed to most constructors.
    let store = Store::new(&engine);

    // We start off by creating a `Module` which represents a compiled form
    // of our input wasm module. In this case it'll be JIT-compiled after
    // we parse the text format.
    let module = Module::new(&engine, wasm_bytes)?;

    // After we have a compiled `Module` we can then instantiate it, creating
    // an `Instance` which we can actually poke at functions on.
    let instance = Instance::new(&store, &module, &[])?;

    // The `Instance` gives us access to various exported functions and items,
    // which we access here to pull out our `answer` exported function and
    // run it.
    let main = instance
        .get_func("wrapped_func")
        .expect("The module did not export the expected `wrapped_func` function");

    // There's a few ways we can call the `answer` `Func` value. The easiest
    // is to statically assert its signature with `get0` (in this case asserting
    // it takes no arguments and returns one i32) and then call it.
    let main = main.get1::<i32, i32>()?;

    let len = pass_string_arg(&instance, &parameters)?;

    // And finally we can call our function! Note that the error propagation
    // with `?` is done to handle the case where the wasm function traps.
    let len: i32 = main(len as i32)?;

    Ok(get_return_value(&instance, len as usize))
}

fn pass_string_arg(instance: &Instance, json: &serde_json::Value) -> Result<usize, anyhow::Error> {
    let json_bytes = serde_json::to_vec(json).unwrap();

    let memory_ptr_func = instance
        .get_func("get_wasm_memory_buffer_pointer")
        .ok_or_else(|| anyhow!("Expected the module to export `get_wasm_memory_buffer_pointer`"))?
        .get0::<i32>()?;

    let memory_ptr_offset = memory_ptr_func().unwrap();

    let memory_base_ptr = instance
        .get_memory("memory")
        .ok_or_else(|| anyhow!("Expected the module to export a memory named `memory`"))?
        .data_ptr();

    for (i, b) in json_bytes.iter().enumerate() {
        unsafe {
            *memory_base_ptr.offset((memory_ptr_offset as isize) + i as isize) = *b;
        }
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
    fn test_pass_str() {
        let wasm_bytes = include_bytes!(
            "../../testfns-openwhisk/str_pass/target/wasm32-unknown-unknown/release/str_pass.wasm"
        );

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
    fn test_add_fail() {
        let wasm_bytes = include_bytes!(
            "../../testfns-openwhisk/str_pass/target/wasm32-unknown-unknown/release/str_pass.wasm"
        );

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
}
