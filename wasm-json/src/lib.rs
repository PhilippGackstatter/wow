// Expects to be called on a function with the signature
// (json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error>
#[macro_export]
macro_rules! pass_json {
    ($($t:ident)*) => ($(
        static mut MEMORY_BUFFER: Vec<u8> = Vec::new();

        // Function to return a pointer to our buffer
        // in Wasm memory
        #[no_mangle]
        pub fn get_wasm_memory_buffer_pointer() -> *mut u8 {
            unsafe { MEMORY_BUFFER.as_mut_ptr() }
        }

        #[no_mangle]
        pub fn wasm_memory_buffer_allocate_space(num_elems: usize) {
            unsafe {
                MEMORY_BUFFER.reserve(num_elems);
                // This is technically unsafe, but as the host will write
                // exactly `num_elems` into this buffer after this function
                // completes, it's fine. It simply saves another invocation.
                MEMORY_BUFFER.set_len(num_elems);
            }
        }

        // Function to get the string from the buffer and add the text to it
        #[no_mangle]
        pub fn wrapped_func(len: usize) -> usize {
            let json = deserialize_slice(len);

            let result = $t(json);

            let result = result.map_err(|err: anyhow::Error| {
                let err_string = err.to_string();

                serde_json::json!({ "error": err_string })
            });

            let mut result = serde_json::to_vec(&result).expect("Could not serialize result.");

            serialize_vec(&mut result)
        }

        fn serialize_vec(vec: &mut Vec<u8>) -> usize {
            unsafe {
                MEMORY_BUFFER.clear();
                MEMORY_BUFFER.append(vec);
                MEMORY_BUFFER.len()
            }
        }

        fn deserialize_slice(len: usize) -> serde_json::Value {
            let slice = unsafe { &MEMORY_BUFFER[..len] };
            serde_json::from_slice(slice).expect("Could not deserialize slice")
        }
    )*)
}
