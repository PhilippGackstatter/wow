# cargos --feature option cannot be combined with --package because of
# missing virtual workspaces support :(

test testname="":
    cargo build --release --examples --target wasm32-wasi
    cargo test --package openwhisk-wasm-runtime {{testname}} -- --nocapture --test-threads=1

build runtime="":
    cd openwhisk-wasm-runtime && cargo build --release --features {{runtime}} && cd ..

build-wasm-examples:
    cd wasm-json && cargo build --release --examples --target wasm32-wasi && cd ..
    find target/wasm32-wasi/release/examples/ -name "*.wasm" -exec ../binaryen/bin/wasm-opt -O4 -o {} {} \;

build-bin-examples:
    cd wasm-json &&\
    cargo build --release --examples --target x86_64-unknown-linux-musl --no-default-features --features bin &&\
    cd ..
    python3 make_bin_actions.py

precompile:
    cd wasm_precompiler && cargo run --bin wasmer ../target/wasm32-wasi/release/examples/*.wasm && cd ..
    cd wasm_precompiler && cargo run --bin wasmtime ../target/wasm32-wasi/release/examples/*.wasm && cd ..
