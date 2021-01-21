test testname="":
    cargo build --release --examples --target wasm32-wasi
    cargo test --package openwhisk-wasm-runtime {{testname}} -- --nocapture --test-threads=1

build:
    cargo build --release --package openwhisk-wasm-runtime

build-examples:
    cargo build --release --examples --target wasm32-wasi

precompile:
    cd wasm_precompiler
    cargo run --bin wasmer ../target/wasm32-wasi/release/examples/*.wasm
    cargo run --bin wasmtime ../target/wasm32-wasi/release/examples/*.wasm
    cd ..