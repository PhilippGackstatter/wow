use anyhow;
use clap::{App, Arg};
use std::{fs::File, io::prelude::*, path::Path};

pub fn get_filenames<'a>() -> Vec<String> {
    let matches = App::new("Wasm Precompiler")
        .version("0.1")
        .author("Philipp Gackstatter")
        .about("Precompiles Wasm modules to runtime-specific code")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file(s) to use")
                .required(true)
                .multiple(true),
        )
        .get_matches();

    matches.values_of_lossy("INPUT").unwrap()
}

pub fn precompile<F: FnOnce(&str) -> anyhow::Result<Vec<u8>>>(
    filename: &str,
    precompile_fn: F,
    runtime_name: &'static str,
) -> anyhow::Result<()> {
    let precompiled_bytes = precompile_fn(&filename)?;

    write_precompiled(filename, runtime_name, precompiled_bytes)?;

    Ok(())
}

fn write_precompiled(
    filename: &str,
    runtime_name: &'static str,
    precompiled: Vec<u8>,
) -> anyhow::Result<()> {
    let file_path = Path::new(&filename);

    let mut new_filepath = file_path.parent().unwrap().to_owned();

    let new_file_stem =
        file_path.file_stem().unwrap().to_string_lossy().to_string() + "-" + runtime_name + ".zip";

    new_filepath.push(new_file_stem);

    println!("Serializing precompiled bytes to {:?}", new_filepath);

    let file = File::create(new_filepath)?;

    write_zip(file, precompiled)?;

    Ok(())
}

fn write_zip(file: File, bytes: Vec<u8>) -> anyhow::Result<()> {
    let mut zip = zip::ZipWriter::new(file);

    let options =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("content", options)?;
    zip.write_all(&bytes)?;

    zip.finish()?;

    Ok(())
}
