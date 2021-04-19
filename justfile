# cargos --feature option cannot be combined with --package because of
# missing virtual workspaces support :(

test testname="" runtime="wasmtime_rt":
    cd ow-executor && cargo test --package ow-executor --features {{runtime}} {{testname}} -- --nocapture --test-threads=1

build-amd64 runtime="":
    cd ow-executor && cargo build --release --features {{runtime}}

build-arm runtime="":
    cd ow-executor && cargo build --release --features {{runtime}} --target armv7-unknown-linux-gnueabihf

build-aarch64 runtime="":
    cd ow-executor && cargo build --release --features {{runtime}} --target aarch64-unknown-linux-gnu

build-wasm-examples:
    just build-examples wasm32-wasi wasm
    find target/wasm32-wasi/release/examples/ -name "*.wasm" -exec wasm-opt -O4 -o {} {} \;

build-bin-examples:
    just build-examples x86_64-unknown-linux-musl bin
    python3 make_bin_actions.py "target/x86_64-unknown-linux-musl/release/examples"

build-examples target feature:
    #!/usr/bin/env bash
    cd ow-wasm-action
    cargo build --release --example add --target {{target}} --no-default-features --features {{feature}}
    cargo build --release --example hash --target {{target}} --no-default-features --features {{feature}},hash
    cargo build --release --example block --target {{target}} --no-default-features --features {{feature}}
    cargo build --release --example random --target {{target}} --no-default-features --features {{feature}},random
    cargo build --release --example prime --target {{target}} --no-default-features --features {{feature}},prime
    cargo build --release --example filesys --target {{target}} --no-default-features --features {{feature}}
    cargo build --release --example clock --target {{target}} --no-default-features --features {{feature}}
    cargo build --release --example net --target {{target}} --no-default-features --features {{feature}}

build-aarch64-bin-examples:
    cd ow-wasm-action &&\
    cargo build --release --examples --target aarch64-unknown-linux-musl --no-default-features --features bin
    python3 make_bin_actions.py "target/aarch64-unknown-linux-musl/release/examples"

precompile mod_name="*":
    ./wasmer_precompile.fish target/wasm32-wasi/release/examples/{{mod_name}}.wasm
    cd ow-wasmtime-precompiler && cargo run --release --bin wasmtime ../target/wasm32-wasi/release/examples/{{mod_name}}.wasm
    ./wamrc_precompile.fish target/wasm32-wasi/release/examples/{{mod_name}}.wasm
