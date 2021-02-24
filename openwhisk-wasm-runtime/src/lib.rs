pub mod core;
mod types;
mod wasm_tests;
#[cfg(feature = "wasmer_rt")]
mod wasmer;
#[cfg(feature = "wasmtime_rt")]
pub mod wasmtime;

// Silences unused warnings
#[cfg(feature = "wasmer_rt")]
pub use self::wasmer::execute_wasm;
