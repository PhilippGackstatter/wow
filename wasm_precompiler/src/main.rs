use anyhow;
use clap::{App, Arg, SubCommand};
use std::{fs::File, io::prelude::*, path::Path, time::Instant};

const WASMTIME: &'static str = "wasmtime";
const WASMER: &'static str = "wasmer";

fn main() -> anyhow::Result<()> {
    let matches = App::new("Wasm Precompiler")
        .version("0.1")
        .author("Philipp Gackstatter")
        .about("Precompiles Wasm modules to runtime-specific code")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .subcommand(SubCommand::with_name(WASMTIME).about("Precompiles for wasmtime"))
        .subcommand(SubCommand::with_name(WASMER).about("Precompiles for wasmer"))
        .get_matches();

    let filename = matches.value_of("INPUT").unwrap();

    match matches.subcommand_name() {
        Some(WASMTIME) => precompile(filename, precompile_wasmtime, WASMTIME),
        Some(WASMER) => precompile(filename, precompile_wasmer, WASMER),
        _ => panic!("No subcommand provided"),
    }
}

fn precompile<F: FnOnce(&str) -> anyhow::Result<Vec<u8>>>(
    filename: &str,
    precompile_fn: F,
    runtime_name: &'static str,
) -> anyhow::Result<()> {
    let precompiled_bytes = precompile_fn(&filename)?;

    write_precompiled(filename, runtime_name, precompiled_bytes)?;

    Ok(())
}

fn precompile_wasmtime(filename: &str) -> anyhow::Result<Vec<u8>> {
    let store = wasmtime::Store::default();

    let timestamp = Instant::now();

    let module = wasmtime::Module::from_file(store.engine(), filename).unwrap();

    println!("Precompiling took {}ms", timestamp.elapsed().as_millis());

    module.serialize()
}

fn precompile_wasmer(filename: &str) -> anyhow::Result<Vec<u8>> {
    // let compiler = LLVM::default();

    // let store = Store::new(&JIT::new(compiler).engine());

    let store = wasmer::Store::default();

    let before = Instant::now();
    let module = wasmer::Module::from_file(&store, filename)?;

    println!("wasmer compiling took {}ms", before.elapsed().as_millis());

    Ok(module.serialize()?)
}

fn write_precompiled(
    filename: &str,
    runtime_name: &'static str,
    precompiled: Vec<u8>,
) -> anyhow::Result<()> {
    let file_path = Path::new(&filename);

    let mut new_filepath = file_path.parent().unwrap().to_owned();

    let new_file_stem =
        file_path.file_stem().unwrap().to_string_lossy().to_string() + "." + runtime_name;

    new_filepath.push(new_file_stem);

    println!("Serializing precompiled bytes to {:?}", new_filepath);

    let mut file = File::create(new_filepath)?;

    file.write(&precompiled)?;

    Ok(())
}
