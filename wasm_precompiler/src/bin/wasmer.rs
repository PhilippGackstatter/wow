use std::time::Instant;

use anyhow;
use clap::{App, Arg};
use wasm_precompiler::precompile;

pub fn precompile_wasmer(filename: &str) -> anyhow::Result<Vec<u8>> {
    let compiler = wasmer::LLVM::default();

    // let store = wasmer::Store::new(&wasmer::JIT::new(compiler).engine());
    let store = wasmer::Store::new(&wasmer::Native::new(compiler).engine());

    let before = Instant::now();
    let module = wasmer::Module::from_file(&store, filename)?;

    println!("wasmer compiling took {}ms", before.elapsed().as_millis());

    Ok(module.serialize()?)
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

    precompile(filename, precompile_wasmer, "wasmer")
}
