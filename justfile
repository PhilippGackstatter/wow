test:
    cargo build --release --examples --target wasm32-wasi
    cargo test --package openwhisk-wasm-runtime

build:
    cargo build --release --package openwhisk-wasm-runtime
