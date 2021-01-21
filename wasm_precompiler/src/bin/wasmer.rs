use std::time::Instant;

use anyhow;
use wasm_precompiler::{get_filenames, precompile};

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
    let filenames = get_filenames();

    for filename in filenames {
        precompile(&filename, precompile_wasmer, "wasmer")?;
    }

    Ok(())
}
