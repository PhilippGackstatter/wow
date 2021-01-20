use std::time::Instant;

use anyhow;
use clap::{App, Arg};
use wasm_precompiler::precompile;

pub fn precompile_wasmtime(filename: &str) -> anyhow::Result<Vec<u8>> {
    let store = wasmtime::Store::default();

    let timestamp = Instant::now();

    let module = wasmtime::Module::from_file(store.engine(), filename).unwrap();

    println!("Precompiling took {}ms", timestamp.elapsed().as_millis());

    module.serialize()
}

fn main() -> anyhow::Result<()> {
    let matches = App::new("Wasmtime Precompiler")
        .version("0.1")
        .author("Philipp Gackstatter")
        .about("Precompiles Wasm modules to runtime-specific code")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();

    precompile(filename, precompile_wasmtime, "wasmtime")
}
