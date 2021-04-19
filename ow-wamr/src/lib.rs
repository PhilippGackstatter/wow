use std::{
    ffi::{CStr, CString},
    fs,
    mem::MaybeUninit,
    os::raw::c_char,
    path::Path,
    ptr::{null, slice_from_raw_parts},
    sync::Arc,
};

use anyhow::{anyhow, bail};
use dashmap::DashMap;
use ow_common::{ActionCapabilities, WasmAction, WasmRuntime};

use wamr_sys::*;

lazy_static::lazy_static! {
    static ref GET_STR: CString = CString::new("get").unwrap();
    static ref SIGNATURE_STR: CString = CString::new("(*)i").unwrap();
    static ref HTTP_STR: CString = CString::new("http").unwrap();
}

static mut NATIVE_SYMBOLS: Vec<NativeSymbol> = Vec::new();

struct WamrRuntimeState {}

impl WamrRuntimeState {
    fn new() -> Self {
        if !unsafe { wasm_runtime_init() } {
            panic!("runtime_init failed");
        }

        let native_get_symbol = NativeSymbol {
            symbol: GET_STR.as_ptr(),
            func_ptr: get as *mut std::ffi::c_void,
            signature: SIGNATURE_STR.as_ptr(),
            attachment: std::ptr::null::<std::ffi::c_void>() as *mut std::ffi::c_void,
        };

        unsafe { NATIVE_SYMBOLS = vec![native_get_symbol] };

        if !unsafe {
            wasm_runtime_register_natives(
                HTTP_STR.as_ptr(),
                NATIVE_SYMBOLS.as_mut_ptr(),
                NATIVE_SYMBOLS.len() as u32,
            )
        } {
            panic!("wasm_runtime_register_natives failed");
        }

        Self {}
    }
}

impl Default for WamrRuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WamrRuntimeState {
    fn drop(&mut self) {
        unsafe {
            wasm_runtime_destroy();
        }
    }
}

#[derive(Clone)]
pub struct Wamr {
    state: Arc<WamrRuntimeState>,
    pub modules: Arc<DashMap<String, WasmAction<Vec<u8>>>>,
}

impl Default for Wamr {
    fn default() -> Self {
        Self {
            state: Arc::new(Default::default()),
            modules: Arc::new(DashMap::new()),
        }
    }
}

impl WasmRuntime for Wamr {
    fn initialize(
        &self,
        container_id: String,
        capabilities: ActionCapabilities,
        module: Vec<u8>,
    ) -> anyhow::Result<()> {
        let action = WasmAction {
            module,
            capabilities,
        };

        self.modules.insert(container_id, action);

        Ok(())
    }

    fn run(
        &self,
        container_id: &str,
        parameters: serde_json::Value,
    ) -> Result<Result<serde_json::Value, serde_json::Value>, anyhow::Error> {
        let wasm_action = self
            .modules
            .get(container_id)
            .ok_or_else(|| anyhow!(format!("No action named {}", container_id)))?;

        wamr_run_module(&wasm_action.capabilities, &wasm_action.module, &parameters)
    }

    fn destroy(&self, container_id: &str) {
        if let None = self.modules.remove(container_id) {
            println!("No container with id {} existed.", container_id);
        }
    }
}

pub fn wamr_run_module(
    capabilities: &ActionCapabilities,
    wasm_module_bytes: &Vec<u8>,
    parameters: &serde_json::Value,
) -> anyhow::Result<Result<serde_json::Value, serde_json::Value>> {
    let mut error_buf: Vec<c_char> = vec![32; 128];
    const STACK_SIZE: u32 = 8092;
    const HEAP_SIZE: u32 = 1024;

    unsafe {
        let time = std::time::Instant::now();

        let module = wasm_runtime_load(
            wasm_module_bytes.as_ptr(),
            wasm_module_bytes.len() as u32,
            error_buf.as_mut_ptr(),
            error_buf.len() as u32,
        );

        println!("wasm_runtime_load: {}ms", time.elapsed().as_millis());

        if module.is_null() {
            bail!(
                "wasm_runtime_load: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let json_bytes = serde_json::to_vec(&parameters).unwrap();
        let mut args = vec![CString::new(format!("{}", json_bytes.len())).unwrap()];
        let mut c_args = args
            .iter_mut()
            .map(|arg| arg.as_ptr() as *mut c_char)
            .collect::<Vec<*mut c_char>>();

        let dir_cap;
        let mut dir_list = if let Some(dir) = &capabilities.dir {
            if !Path::new(dir).exists() {
                fs::create_dir_all(dir)?;
            }
            dir_cap = CString::new(dir.clone()).unwrap();
            vec![dir_cap.as_ptr() as *const c_char]
        } else {
            vec![]
        };

        let null_ptr = 0 as *mut *const c_char;

        wasm_runtime_set_wasi_args(
            module,
            dir_list.as_mut_ptr(),
            dir_list.len() as u32,
            null_ptr,
            0,
            null_ptr,
            0,
            c_args.as_mut_ptr(),
            1,
        );

        let time = std::time::Instant::now();

        let module_inst = wasm_runtime_instantiate(
            module,
            STACK_SIZE,
            HEAP_SIZE,
            error_buf.as_mut_ptr(),
            error_buf.len() as u32,
        );

        println!("wasm_runtime_instantiate: {}ms", time.elapsed().as_millis());

        if module_inst.is_null() {
            bail!(
                "wasm_runtime_instantiate: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let time = std::time::Instant::now();

        let exec_env = wasm_runtime_create_exec_env(module_inst, STACK_SIZE);

        println!(
            "wasm_runtime_create_exec_env: {}ms",
            time.elapsed().as_millis()
        );

        if exec_env.is_null() {
            bail!(
                "wasm_runtime_create_exec_env: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        pass_string_arg(exec_env, module_inst, json_bytes)?;

        let time = std::time::Instant::now();

        let start_func = wasm_runtime_lookup_wasi_start_function(module_inst);

        println!(
            "wasm_runtime_lookup_wasi_start_function: {}ms",
            time.elapsed().as_millis()
        );

        if start_func.is_null() {
            bail!(
                "wasm_runtime_lookup_wasi_start_function: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let time = std::time::Instant::now();

        call_function(exec_env, module_inst, start_func, false, vec![])?;

        let ret_val = get_return_value(exec_env, module_inst);

        println!("call_function: {}ms", time.elapsed().as_millis());

        wasm_runtime_destroy_exec_env(exec_env);
        wasm_runtime_deinstantiate(module_inst);
        wasm_runtime_unload(module);

        Ok(ret_val)
    }
}

pub fn pass_string_arg(
    exec_env: wasm_exec_env_t,
    instance: wasm_module_inst_t,
    json_bytes: Vec<u8>,
) -> Result<(), anyhow::Error> {
    let wasm_memory_buffer_allocate_space =
        lookup_function(instance, "wasm_memory_buffer_allocate_space")?;

    let json_len = wasm_val_t {
        kind: wasm_valkind_enum_WASM_I32 as wasm_valkind_t,
        of: wasm_val_t__bindgen_ty_1 {
            i32_: json_bytes.len() as i32,
        },
    };

    let args = vec![json_len];

    call_function(
        exec_env,
        instance,
        wasm_memory_buffer_allocate_space,
        false,
        args,
    )?;

    let get_wasm_memory_buffer_pointer =
        lookup_function(instance, "get_wasm_memory_buffer_pointer")?;

    let memory_buffer_offset = call_function(
        exec_env,
        instance,
        get_wasm_memory_buffer_pointer,
        true,
        vec![],
    )?;

    unsafe {
        let memory_buffer =
            wasm_runtime_addr_app_to_native(instance, memory_buffer_offset) as *mut u8;

        memory_buffer.copy_from_nonoverlapping(json_bytes.as_ptr(), json_bytes.len());
    }

    Ok(())
}

fn get_return_value(
    exec_env: wasm_exec_env_t,
    instance: wasm_module_inst_t,
) -> Result<serde_json::Value, serde_json::Value> {
    let get_wasm_memory_buffer_pointer =
        lookup_function(instance, "get_wasm_memory_buffer_pointer").unwrap();

    let get_wasm_memory_buffer_len =
        lookup_function(instance, "get_wasm_memory_buffer_len").unwrap();

    let len = call_function(exec_env, instance, get_wasm_memory_buffer_len, true, vec![]).unwrap()
        as usize;

    let memory_buffer_offset = call_function(
        exec_env,
        instance,
        get_wasm_memory_buffer_pointer,
        true,
        vec![],
    )
    .unwrap();

    let memory_buffer =
        unsafe { wasm_runtime_addr_app_to_native(instance, memory_buffer_offset) as *const u8 };

    let wasm_mem_slice = slice_from_raw_parts(memory_buffer, len);

    serde_json::from_slice(unsafe { &*wasm_mem_slice }).unwrap()
}

fn call_function(
    exec_env: wasm_exec_env_t,
    instance: wasm_module_inst_t,
    func: wasm_function_inst_t,
    has_retval: bool,
    mut args: Vec<wasm_val_t>,
) -> anyhow::Result<u32> {
    unsafe {
        // We need to pass in the correct amount of return values, hence this thing.
        let mut results = if has_retval {
            // This will be overwritten by the return value, hence we can use uninitialized memory.
            vec![MaybeUninit::uninit().assume_init()]
        } else {
            vec![]
        };

        if wasm_runtime_call_wasm_a(
            exec_env,
            func,
            results.len() as u32,
            results.as_mut_ptr(),
            args.len() as u32,
            args.as_mut_ptr(),
        ) {
            if has_retval {
                // We're only dealing with 32-bit integers, so we can assume this.
                let retval = results[0].of.i32_;
                Ok(retval as u32)
            } else {
                Ok(0)
            }
        } else {
            Err(anyhow::anyhow!(
                "call failed: {}",
                CStr::from_ptr(wasm_runtime_get_exception(instance)).to_string_lossy()
            ))
        }
    }
}

fn lookup_function(
    instance: wasm_module_inst_t,
    func_name: &str,
) -> anyhow::Result<wasm_function_inst_t> {
    let func_name_cstr = CString::new(func_name).unwrap();

    let start_func = unsafe {
        wasm_runtime_lookup_function(instance, func_name_cstr.as_c_str().as_ptr(), null())
    };

    if !start_func.is_null() {
        Ok(start_func)
    } else {
        Err(anyhow::anyhow!("function `{}` not found", func_name))
    }
}

extern "C" fn get(_exec_env: wasm_exec_env_t) -> i32 {
    std::thread::sleep(std::time::Duration::new(0, 300_000_000));
    0
}
