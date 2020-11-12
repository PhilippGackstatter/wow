use wasmtime::*;

pub fn execute_wasm(
    parameters: serde_json::Value,
    wasm_bytes: &Vec<u8>,
) -> Result<serde_json::Value, anyhow::Error> {
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
    let add = instance
        .get_func("add")
        .expect("`add` was not an exported function");

    // There's a few ways we can call the `answer` `Func` value. The easiest
    // is to statically assert its signature with `get0` (in this case asserting
    // it takes no arguments and returns one i32) and then call it.
    let add = add.get2::<i32, i32, i32>()?;

    let first = parameters["first"].as_i64().unwrap() as i32;
    let second = parameters["second"].as_i64().unwrap() as i32;

    // And finally we can call our function! Note that the error propagation
    // with `?` is done to handle the case where the wasm function traps.
    let result: i32 = add(first, second)?;

    Ok(serde_json::json!({ "result": result }))
}
