use std::{
    ffi::{CStr, CString},
    ptr::null,
};

use wamr_sys::*;

pub fn run_module() {
    let wasm_module_bytes = include_bytes!(
        "/home/morgan/git/wasm-openwhisk/target/wasm32-wasi/release/examples/add_simple.wasm"
    );
    let mut error_buf: Vec<i8> = vec![32; 128];
    const STACK_SIZE: u32 = 8092;

    unsafe {
        wasm_runtime_init();

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

        // wasm_runtime_lookup_wasi_start_function
        let func_name = CString::new("wasm_function").unwrap();
        let start_func =
            wasm_runtime_lookup_function(module_inst, func_name.as_c_str().as_ptr(), null());

        if start_func.is_null() {
            panic!(
                "start func is null: {}",
                CStr::from_ptr(error_buf.as_ptr()).to_string_lossy()
            );
        }

        let mut args: [u32; 2] = [2, 5];
        // null::<*const u32>() as *mut u32
        if wasm_runtime_call_wasm(exec_env, start_func, args.len() as u32, args.as_mut_ptr()) {
            println!("Call succeeded, retval: {}", args[0]);
        } else {
            println!(
                "Call failed: {}",
                CStr::from_ptr(wasm_runtime_get_exception(module_inst)).to_string_lossy()
            );
        }
    }
}
