pub mod core;
mod types;
mod wasm_tests;
mod wasmer;
mod wasmtime;

// Silences unused warnings
pub use self::wasmer::execute_wasm;
