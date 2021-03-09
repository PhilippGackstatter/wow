use anyhow::anyhow;
use std::{fs, path::Path, ptr::slice_from_raw_parts, sync::Arc};

use dashmap::DashMap;
use wasmer::{Instance, Module, Store};
use wasmer_wasi::{WasiEnv, WasiState};

use ow_common::{ActionCapabilities, WasmAction, WasmRuntime, util};

#[derive(Clone)]
pub struct Wasmer {
    pub store: Store,
    pub modules: Arc<DashMap<String, WasmAction<Module>>>,
}

impl Default for Wasmer {
    fn default() -> Self {
        Self {
            store: wasmer::Store::new(&wasmer::Native::headless().engine()),
            modules: Arc::new(DashMap::new()),
        }
    }
}

impl WasmRuntime for Wasmer {
    fn initialize_action(
        &self,
        action_name: String,
        capabilities: ActionCapabilities,
        module_bytes_b64: String,
    ) -> anyhow::Result<()> {
        let module_bytes = util::b64_decode(module_bytes_b64)?;

        let module = unsafe { Module::deserialize(&self.store, &module_bytes)? };

        let action = WasmAction {
            module,
            capabilities,
        };

        self.modules.insert(action_name, action);

        Ok(())
    }

    fn execute(
        &self,
        action_name: &str,
        parameters: serde_json::Value,
    ) -> Result<Result<serde_json::Value, serde_json::Value>, anyhow::Error> {
        let wasm_action = self
            .modules
            .get(action_name)
            .ok_or_else(|| anyhow!(format!("No action named {}", action_name)))?;

        // let before = Instant::now();
        // println!(
        //     "wasmer deserializing took {}ms",
        //     before.elapsed().as_millis()
        // );

        let module = &wasm_action.module;

        let json_bytes = serde_json::to_vec(&parameters).unwrap();

        let mut wasi_env = build_wasi_env(&wasm_action.capabilities, json_bytes.len())?;

        let import_object = wasi_env.import_object(&module)?;

        let instance = Instance::new(&module, &import_object)?;

        let main = instance.exports.get_function("_start")?;

        pass_string_arg(&instance, json_bytes)?;

        main.call(&[])?;

        Ok(get_return_value(&instance))
    }
}

fn build_wasi_env(
    capabilities: &ActionCapabilities,
    arg_len: usize,
) -> Result<WasiEnv, anyhow::Error> {
    let mut builder = WasiState::new("wasm-openwhisk");

    builder.arg(format!("{}", arg_len));

    if let Some(dir) = &capabilities.dir {
        if !Path::new(dir).exists() {
            fs::create_dir_all(dir)?;
        }

        builder.preopen(|p| p.directory(dir).read(true).write(true).create(true))?;
    }

    Ok(builder.finalize()?)
}

fn pass_string_arg(instance: &Instance, json_bytes: Vec<u8>) -> Result<(), anyhow::Error> {
    let wasm_memory_buffer_allocate_space = instance
        .exports
        .get_native_function::<i32, ()>("wasm_memory_buffer_allocate_space")?;

    wasm_memory_buffer_allocate_space.call(json_bytes.len() as i32)?;

    let memory_buffer_func = instance
        .exports
        .get_native_function::<(), i32>("get_wasm_memory_buffer_pointer")?;

    let memory_buffer_offset = memory_buffer_func.call().unwrap();

    let memory_base_ptr = instance.exports.get_memory("memory")?.data_ptr();

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
        .exports
        .get_native_function::<(), i32>("get_wasm_memory_buffer_pointer")
        .unwrap();

    let memory_buf_len_func = instance
        .exports
        .get_native_function::<(), i32>("get_wasm_memory_buffer_len")
        .unwrap();

    let memory_buf_len = memory_buf_len_func.call().unwrap();

    let memory_ptr_offset = memory_ptr_func.call().unwrap();

    let memory_base_ptr = instance.exports.get_memory("memory").unwrap().data_ptr();

    let wasm_mem_slice = slice_from_raw_parts(
        unsafe { memory_base_ptr.offset(memory_ptr_offset as isize) as *const u8 },
        memory_buf_len as usize,
    );

    serde_json::from_slice(unsafe { &*wasm_mem_slice }).unwrap()
}
