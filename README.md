<div align="center">
  <h1><b>W</b>ebAssembly-flavored <b>O</b>pen<b>W</b>hisk</h1>

<strong>A WebAssembly-based container runtime for the Apache OpenWhisk serverless platform.</strong>

</div>

## Why?

The cold start problem is a well-known one in serverless platforms. The first invocation of a function requires provisioning all the resources necessary for its execution. Typically that means starting a docker container, i.e. a `node.js` container to then inject some JavaScript code for execution. This process can take hundreds of milliseconds and is thus unacceptable for latency-critical tasks. This project is an evaluation of using WebAssembly instead of Docker containers to implement serverless functions. The simple idea is that WebAssembly is a lighter-weight sandboxing mechanism so the cold start latency should be much smaller. It builds on top of [`Apache OpenWhisk`](https://github.com/apache/openwhisk) as the serverless platform. There are three repositories relevant to this:

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

## Building

We use [just](https://github.com/casey/just) to make building a bit easer. The recipes are defined in the `justfile`. If you don't want to install this tool, you can copy the command from there into your terminal.

To build the `executor` with the `wasmtime` runtime, run `just build-amd64 wasmtime_rt`. Other possible values for the runtime argument are `wasmer_rt` and `wamr_rt` to use `wasmer` or `wamr` respectively.

To build example actions use `just build-wasm-examples`. This requires binaryen's `wasm-opt` to be present in your `PATH`, which optimizes the produced wasm. For convenience you can also skip this part. To precompile the examples use `just precompile`, which requires `wasmer` and `wamrc` to be installed or in your `PATH`. Of course only one of the precompilation steps is necessary to run a given module for a certain runtime, so you can skip some of these steps as well.
