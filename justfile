test testname="":
    cargo build --release --examples --target wasm32-wasi
    cargo test --package openwhisk-wasm-runtime {{testname}} -- --nocapture --test-threads=1

build:
    cargo build --release --package openwhisk-wasm-runtime

build-examples:
    cargo build --release --examples --target wasm32-wasi
    ../binaryen/bin/wasm-opt -O4 -o target/wasm32-wasi/release/examples/add.wasm target/wasm32-wasi/release/examples/add.wasm
    ../binaryen/bin/wasm-opt -O4 -o target/wasm32-wasi/release/examples/clock.wasm target/wasm32-wasi/release/examples/clock.wasm
    ../binaryen/bin/wasm-opt -O4 -o target/wasm32-wasi/release/examples/filesys.wasm target/wasm32-wasi/release/examples/filesys.wasm
    ../binaryen/bin/wasm-opt -O4 -o target/wasm32-wasi/release/examples/random.wasm target/wasm32-wasi/release/examples/random.wasm

precompile:
    cd wasm_precompiler
    cargo run --bin wasmer ../target/wasm32-wasi/release/examples/*.wasm
    cargo run --bin wasmtime ../target/wasm32-wasi/release/examples/*.wasm
    cd ..