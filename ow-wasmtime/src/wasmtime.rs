use std::{
    fs::{DirBuilder, File},
    io::Write,
    ptr::slice_from_raw_parts,
    sync::Arc,
    // time::Instant,
};

use anyhow::anyhow;
use cap_std::fs::Dir;
use dashmap::DashMap;
use wasi_cap_std_sync::WasiCtxBuilder;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};
use wasmtime_wasi::{Wasi, WasiCtx};

use ow_common::{util, ActionCapabilities, WasmAction, WasmRuntime};

#[derive(Clone)]
pub struct Wasmtime {
    pub engine: Engine,
    pub modules: Arc<DashMap<String, WasmAction<Module>>>,
}

impl Default for Wasmtime {
    fn default() -> Self {
        Self {
            engine: Engine::new(&make_wasmtime_config().unwrap()),
            modules: Arc::new(DashMap::new()),
        }
    }
}

impl WasmRuntime for Wasmtime {
    fn initialize(
        &self,
        container_id: String,
        capabilities: ActionCapabilities,
        module: Vec<u8>,
    ) -> anyhow::Result<()> {
        let module = Module::deserialize(&self.engine, &module)?;

        let action = WasmAction {
            module,
            capabilities,
        };

        self.modules.insert(container_id, action);

        Ok(())
    }

    fn destroy(&self, container_id: &str) {
        if let None = self.modules.remove(container_id) {
            println!("No container with id {} existed.", container_id);
        }
    }

    fn run(
        &self,
        container_id: &str,
        parameters: serde_json::Value,
    ) -> Result<Result<serde_json::Value, serde_json::Value>, anyhow::Error> {
        let store = Store::new(&self.engine);

        let mut linker = Linker::new(&store);

        let json_bytes = serde_json::to_vec(&parameters).unwrap();

        let wasm_action = self
            .modules
            .get(container_id)
            .ok_or_else(|| anyhow!(format!("No action named {}", container_id)))?;

        let ctx = build_wasi_context(&wasm_action.capabilities, json_bytes.len())?;
        let wasi = Wasi::new(&store, ctx);
        wasi.add_to_linker(&mut linker)?;

        if let Some(true) = wasm_action.capabilities.net_access {
            link_net(&mut linker)?;
        }

        // let timestamp = Instant::now();

        let module = &wasm_action.module;

        // println!(
        //     "wasmtime compiling took {}ms",
        //     timestamp.elapsed().as_millis()
        // );

        let instance = linker.instantiate(module)?;
        let main = linker.instance("", &instance)?.get_default("")?;

        pass_string_arg(&instance, json_bytes)?;

        main.call(&[])?;

        Ok(get_return_value(&instance))
    }
}

fn link_net(linker: &mut Linker) -> anyhow::Result<()> {
    linker.func("http", "get", || -> i32 {
        // Fake call by blocking for 300ms
        std::thread::sleep(std::time::Duration::new(0, 300_000_000));

        0
    })?;

    Ok(())
}

fn build_wasi_context(
    capabilities: &ActionCapabilities,
    arg_len: usize,
) -> Result<WasiCtx, anyhow::Error> {
    let mut ctx_builder = WasiCtxBuilder::new();

    ctx_builder = ctx_builder
        .inherit_stdout()
        .inherit_stderr()
        .arg(&format!("{}", arg_len))?;

    if let Some(dir) = &capabilities.dir {
        // Can be made async

        DirBuilder::new().recursive(true).create(dir).unwrap();

        let cap_dir = unsafe { Dir::from_std_file(File::open(dir)?) };

        ctx_builder = ctx_builder.preopened_dir(cap_dir, dir)?;
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

pub fn make_wasmtime_config() -> anyhow::Result<Config> {
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
