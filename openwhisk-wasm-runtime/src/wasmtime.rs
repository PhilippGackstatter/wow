use std::{
    fs::{DirBuilder, File},
    io::Write,
    ptr::slice_from_raw_parts,
    time::Instant,
};

use anyhow::anyhow;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{Wasi, WasiCtx, WasiCtxBuilder};

use crate::types::{ActionCapabilities, WasmAction};

pub fn execute_wasm(
    parameters: serde_json::Value,
    wasm_action: &WasmAction,
) -> Result<Result<serde_json::Value, serde_json::Value>, anyhow::Error> {
    let config = make_wasmtime_config()?;
    let engine = Engine::new(&config);

    let store = Store::new(&engine);

    let mut linker = Linker::new(&store);

    let json_bytes = serde_json::to_vec(&parameters).unwrap();

    let ctx = build_wasi_context(&wasm_action.capabilities, json_bytes.len())?;
    let wasi = Wasi::new(&store, ctx);
    wasi.add_to_linker(&mut linker)?;

    let timestamp = Instant::now();

    // let module = Module::new(store.engine(), &wasm_action.code)?;
    let module = Module::deserialize(store.engine(), &wasm_action.code)?;

    println!(
        "wasmtime compiling took {}ms",
        timestamp.elapsed().as_millis()
    );

    let instance = linker.instantiate(&module)?;
    let main = linker.instance("", &instance)?.get_default("")?;

    pass_string_arg(&instance, json_bytes)?;

    main.call(&[])?;

    Ok(get_return_value(&instance))
}

fn build_wasi_context(
    capabilities: &ActionCapabilities,
    arg_len: usize,
) -> Result<WasiCtx, anyhow::Error> {
    let mut ctx_builder = WasiCtxBuilder::new();

    ctx_builder.inherit_stdout().inherit_stderr();
    ctx_builder.arg(format!("{}", arg_len));

    if let Some(dir) = &capabilities.dir {
        // Can be made async

        DirBuilder::new().recursive(true).create(dir).unwrap();
        let dir_file = File::open(dir)?;

        ctx_builder.preopened_dir(dir_file, dir);
    }

    Ok(ctx_builder.build()?)
}

fn pass_string_arg(instance: &Instance, json_bytes: Vec<u8>) -> Result<(), anyhow::Error> {
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

    Ok(())
}

fn get_return_value(instance: &Instance) -> Result<serde_json::Value, serde_json::Value> {
    // We can unwrap here, because we handled these exact errors earlier
    // so we wouldn't reach this point if the functions wouldn't exist.
    let memory_ptr_func = instance
        .get_func("get_wasm_memory_buffer_pointer")
        .unwrap()
        .get0::<i32>()
        .unwrap();

    let memory_buf_len_func = instance
        .get_func("get_wasm_memory_buffer_len")
        .unwrap()
        .get0::<i32>()
        .unwrap();

    let memory_buf_len = memory_buf_len_func().unwrap();

    let memory_ptr_offset = memory_ptr_func().unwrap();

    let memory_base_ptr = instance.get_memory("memory").unwrap().data_ptr();

    let wasm_mem_slice = slice_from_raw_parts(
        unsafe { memory_base_ptr.offset(memory_ptr_offset as isize) as *const u8 },
        memory_buf_len as usize,
    );

    serde_json::from_slice(unsafe { &*wasm_mem_slice }).unwrap()
}

fn make_wasmtime_config() -> anyhow::Result<Config> {
    let mut config = Config::default();

    make_cache_config(&mut config)?;

    Ok(config)
}

fn make_cache_config(config: &mut Config) -> anyhow::Result<()> {
    let cache_config_toml = r#"
        [cache]
        enabled = true
        directory = "/tmp/wasmtime-cache/"
        files-total-size-soft-limit = "256Mi"
        "#;

    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(cache_config_toml.as_bytes())?;

    config.cache_config_load(file.path())?;

    Ok(())
}
