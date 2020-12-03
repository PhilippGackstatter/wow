test:
    cargo build --release --examples --target wasm32-wasi
    cargo test --package openwhisk-wasm-runtime
