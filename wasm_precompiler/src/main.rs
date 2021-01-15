use anyhow;
use std::{fs::File, io::prelude::*, path::Path, time::Instant};
use wasmtime::{Module, Store};

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();

    args.next();

    let filename = args.next().unwrap();

    let pre_wasmtime = precompile_wasmtime(&filename)?;

    write_precompiled(filename, pre_wasmtime)?;

    Ok(())
}

fn precompile_wasmtime(filename: &str) -> anyhow::Result<Vec<u8>> {
    let store = Store::default();

    let timestamp = Instant::now();

    let module = Module::from_file(store.engine(), filename).unwrap();

    println!("Precompiling took {}ms", timestamp.elapsed().as_millis());

    module.serialize()
}

fn write_precompiled(filename: String, precompiled: Vec<u8>) -> anyhow::Result<()> {
    let file_path = Path::new(&filename);

    let mut new_filepath = file_path.parent().unwrap().to_owned();

    let new_file_stem = file_path.file_stem().unwrap().to_string_lossy().to_string() + ".wasmtime";

    new_filepath.push(new_file_stem);

    println!("Serializing precompiled bytes to {:?}", new_filepath);

    let mut file = File::create(new_filepath)?;

    file.write(&precompiled)?;

    Ok(())
}
