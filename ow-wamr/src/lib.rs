use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    ptr::{null, slice_from_raw_parts},
};

use wamr_sys::*;

pub fn run_module() -> anyhow::Result<()> {
    let wasm_module_bytes = include_bytes!(
        "/home/morgan/git/wasm-openwhisk/target/wasm32-wasi/release/examples/add.wasm"
    );
    let mut error_buf: Vec<i8> = vec![32; 128];
    const STACK_SIZE: u32 = 8092;

    unsafe {
        if !wasm_runtime_init() {
            anyhow::bail!("runtime_init returned false");
        }

        // let init_args = wamr_sys::RuntimeInitArgs {
        //     mem_alloc_type: wamr_sys::mem_alloc_type_t_Alloc_With_Pool,
        //     mem_alloc_option: wamr_sys::MemAllocOption {
        //         pool: wamr_sys::MemAllocOption__bindgen_ty_1 {
        //             heap_buf: heap_buffer.as_mut_ptr() as *mut c_void,
        //             heap_size: heap_buffer.len() as u32,
        //         }
        //         allocator:
        //     }
        // };

        let module = wasm_runtime_load(
            wasm_module_bytes.as_ptr(),
            wasm_module_bytes.len() as u32,
            error_buf.as_mut_ptr(),
            error_buf.len() as u32,
        );

        if module.is_null() {
            panic!(
                "Module is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let module_inst = wasm_runtime_instantiate(
            module,
            STACK_SIZE,
            0,
            error_buf.as_mut_ptr(),
            error_buf.len() as u32,
        );

        if module_inst.is_null() {
            panic!(
                "Instatiated module is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let exec_env = wasm_runtime_create_exec_env(module_inst, STACK_SIZE);

        if exec_env.is_null() {
            panic!(
                "Exec env is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }
        let json_bytes = serde_json::to_vec(&serde_json::json!({
            "param1": 1,
            "param2": 3,
        }))
        .unwrap();
        pass_string_arg(exec_env, module_inst, json_bytes)?;
        let ret_val = get_return_value(exec_env, module_inst);

        println!("{:?}", ret_val);

        // Maybe we can use wasm_runtime_lookup_wasi_start_function
        let func_name = CString::new("main").unwrap();
        let start_func =
            wasm_runtime_lookup_function(module_inst, func_name.as_c_str().as_ptr(), null());

        if start_func.is_null() {
            panic!(
                "start func is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let mut args: [u32; 1] = [0];
        if wasm_runtime_call_wasm(exec_env, start_func, 0, args.as_mut_ptr()) {
            println!("Call succeeded, retval: {}", args[0]);
        } else {
            println!(
                "Call failed: {}",
                CStr::from_ptr(wasm_runtime_get_exception(module_inst)).to_string_lossy()
            );
        }
    }

    Ok(())
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

fn get_return_value(exec_env: wasm_exec_env_t, instance: wasm_module_inst_t) -> serde_json::Value {
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
            // This will be overwritten by a return value, hence we can use uninitialized memory.
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
                "Call failed: {}",
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
        Err(anyhow::anyhow!("Function {} not found", func_name))
    }
}
