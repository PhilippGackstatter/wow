# cargos --feature option cannot be combined with --package because of
# missing virtual workspaces support :(

test testname="" runtime="wasmtime_rt":
    cd ow-executor && cargo test --package ow-executor --features {{runtime}} {{testname}} -- --nocapture --test-threads=1

build-amd64 runtime="":
    cd ow-executor && cargo build --release --features {{runtime}}

build-arm runtime="":
    cd ow-executor && cargo build --release --features {{runtime}} --target armv7-unknown-linux-gnueabihf

build-wasm-examples:
    cd ow-wasm-action && cargo build --release --features wasm --no-default-features --examples --target wasm32-wasi
    find target/wasm32-wasi/release/examples/ -name "*.wasm" -exec ../binaryen/bin/wasm-opt -O4 -o {} {} \;

build-bin-examples:
    cd ow-wasm-action &&\
    cargo build --release --examples --target x86_64-unknown-linux-musl --no-default-features --features bin
    python3 make_bin_actions.py "target/x86_64-unknown-linux-musl/release/examples"

build-aarch64-bin-examples:
    cd ow-wasm-action &&\
    cargo build --release --examples --target aarch64-unknown-linux-musl --no-default-features --features bin
    python3 make_bin_actions.py "target/aarch64-unknown-linux-musl/release/examples"

precompile mod_name="*":
    ./wasmer_precompile.fish target/wasm32-wasi/release/examples/{{mod_name}}.wasm
    cd ow-wasm-precompiler && cargo run --release --bin wasmtime ../target/wasm32-wasi/release/examples/{{mod_name}}.wasm
    ./wamrc_precompile.fish target/wasm32-wasi/release/examples/{{mod_name}}.wasm
