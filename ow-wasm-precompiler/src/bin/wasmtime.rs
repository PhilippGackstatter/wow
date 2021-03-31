use std::time::Instant;

use anyhow;
use wasm_precompiler::{get_filenames, precompile};

pub fn precompile_wasmtime(filename: &str) -> anyhow::Result<Vec<u8>> {
    let store = wasmtime::Store::default();

    let timestamp = Instant::now();

    let module = wasmtime::Module::from_file(store.engine(), filename).unwrap();

    println!("Precompiling took {}ms", timestamp.elapsed().as_millis());

    module.serialize()
}

fn main() -> anyhow::Result<()> {
    let filenames = get_filenames();

    for filename in filenames {
        precompile(&filename, precompile_wasmtime, "wasmtime")?;
    }

    Ok(())
}
