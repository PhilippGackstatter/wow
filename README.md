<div align="center">
  <h1>WebAssembly-flavored OpenWhisk</h1>

<strong>A WebAssembly-based container runtime for the Apache OpenWhisk serverless platform.</strong>

</div>

## Why?

The cold start problem is well-known in serverless platforms. The first invocation of a function requires provisioning all the resources necessary for its execution. Typically that means starting a docker container, i.e. a `node.js` container to then inject some JavaScript code for execution. This process can take hundreds of milliseconds and is thus unacceptable for latency-critical tasks. This project is an evaluation of using WebAssembly instead of Docker containers to implement serverless functions. The simple idea is that WebAssembly is a lighter-weight sandboxing mechanism so the cold start latency should be much smaller. It builds on top of [`Apache OpenWhisk`](https://github.com/apache/openwhisk) as the serverless platform. There are three repositories relevant to this:

- This repository, which implements the WebAssembly-based container runtime.
- The [OpenWhisk fork](https://github.com/PhilippGackstatter/openwhisk/tree/invoker-wasm) that implements the runtimes support on the OpenWhisk side
- The [diploma thesis](https://github.com/PhilippGackstatter/diploma-thesis/) repository, which contains the thesis describing the motivation, background, design and evaluation of this idea.

## Crates Overview

The project is split into multiple crates, which are:

- `ow-common` contains common types such as the `WasmRuntime` trait or types that represent OpenWhisk payloads.
- `ow-executor` implements the actual container runtime and the OpenWhisk runtime protocol.
- `ow-wamr` implements the `WasmRuntime` trait for the [WebAssembly Micro Runtime](https://github.com/bytecodealliance/wasm-micro-runtime/).
- `ow-wasmer` implements the `WasmRuntime` trait for [Wasmer](https://github.com/wasmerio/wasmer).
- `ow-wasmtime` implements the `WasmRuntime` trait for [Wasmtime](https://github.com/bytecodealliance/wasmtime).
- `ow-wasm-action` contains abstractions for building WebAssembly serverless functions ("actions" in jOpenWhisk terminology) and has a number of example actions.
- `ow-wasmtime-precompiler` implements Ahead-of-Time compilation for `wasmtime` (which is part of the runtime as of version `0.26`), making this crate obsolete once `ow-wasmtime` is migrated to that version.
- `ow-evaluation` contains some tests used for evaluating the performance of the system, such as concurrency tests and cold start tests.

## Tutorial with Wasmtime

As a small tutorial, let's build the wasmtime executor and run one of the examples.

1. To build the executor with wasmtime run the following command from the root of this repository:

```sh
cargo build --manifest-path ./ow-executor/Cargo.toml --release --features wasmtime_rt
```

2. Next we build the `add` example for the `wasm32-wasi` target with:

```sh
cargo build --manifest-path ./ow-wasm-action/Cargo.toml --release --example add --target wasm32-wasi --no-default-features

# Optional step to optimize the compiled Wasm if `wasm-opt` is installed
# On Ubuntu it can be installed with `sudo apt install binaryen`
wasm-opt -O4 -o ./target/wasm32-wasi/release/examples/add.wasm ./target/wasm32-wasi/release/examples/add.wasm
```

3. Precompile the example for efficient execution with wasmtime:

```sh
cargo run --manifest-path ./ow-wasmtime-precompiler/Cargo.toml --release --bin wasmtime ./target/wasm32-wasi/release/examples/add.wasm
```

4. Install wsk-cli from https://github.com/apache/openwhisk-cli/releases/tag/1.2.0

5. Clone the openwhisk repo, checkout the appropriate branch and run OpenWhisk in a separate terminal:

```sh
git clone git@github.com:PhilippGackstatter/openwhisk.git
git checkout invoker-wasm
./gradlew core:standalone:bootRun
```

This will print something like the following:

```
[ WARN  ] Configure wsk via below command to connect to this server as [guest]

wsk property set --apihost 'http://172.17.0.1:3233' --auth '23bc46b1-71f6-4ed5-8c54-816aa4f8c502:123zO3xZCLrMN6v2BKK1dXYFpXlPkccOFqm12CdAsMgRU4VrNZ9lyGVCGuMDGIwP'
```

Execute this command.

6. Run the executor in a separate terminal. OpenWhisk will forward execution requests for Wasm to it:

```sh
./target/release/executor
```

7. Upload the example zip to OpenWhisk:

```sh
wsk action create --kind wasm:0.1 add ./target/wasm32-wasi/release/examples/add-wasmtime.zip
```

8. Run the action. We need to provide the correct parameters for this action, which are defined in our action source file `ow-wasm-action/examples/add.rs`. Let's see what the result of `2+2` is:

```sh
wsk action invoke --blocking add --param param1 2 --param param2 2
```

which prints:

```json
{
  "activationId": "b67ca8877a494fcdbca8877a494fcd73",
  "annotations": [
    {
      "key": "path",
      "value": "guest/add"
    },
    {
      "key": "waitTime",
      "value": 23
    },
    {
      "key": "kind",
      "value": "wasm:0.1"
    },
    {
      "key": "timeout",
      "value": false
    },
    {
      "key": "limits",
      "value": {
        "concurrency": 1,
        "logs": 10,
        "memory": 256,
        "timeout": 60000
      }
    },
    {
      "key": "initTime",
      "value": 3
    }
  ],
  "duration": 5,
  "end": 1655918974484,
  "logs": [],
  "name": "add",
  "namespace": "guest",
  "publish": false,
  "response": {
    "result": {
      "result": {
        "result": 4
      },
      "status": "success",
      "status_code": 0,
      "success": true
    },
    "size": 73,
    "status": "success",
    "success": true
  },
  "start": 1655918974479,
  "subject": "guest",
  "version": "0.0.1"
}
```

And we get `4` as a result. And that's it, we've run a serverless function with WebAssembly! Try running one of the other examples in ``ow-wasm-action/examples/`, the appropriate compilation commands and features can be found in the `justfile`.

## Building

We can also use [just](https://github.com/casey/just) to make building a bit easier. The recipes are defined in the `justfile`. If you don't want to install this tool, you can copy the commands from there into your terminal.

The `just` commands generally run operations for all three runtimes, but if not all tools for all Wasm runtimes are installed, some of the steps can be skipped.

To build the `executor` with the `wasmtime` runtime, run `just build-amd64 wasmtime_rt`. Other possible values for the runtime argument are `wasmer_rt` and `wamr_rt` to use `wasmer` or `wamr` respectively.

To build example actions use `just build-wasm-examples`. This requires binaryen's `wasm-opt` to be present in your `PATH`, which optimizes the produced wasm. For convenience you can also skip this part. To precompile the examples use `just precompile`, which requires `wasmer` and `wamrc` to be installed or in your `PATH`. Of course only one of the precompilation steps is necessary to run a given module for a certain runtime, so you can skip some of these steps as well.
