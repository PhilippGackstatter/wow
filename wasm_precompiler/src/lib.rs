use anyhow;
use std::{fs::File, io::prelude::*, path::Path};

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
        file_path.file_stem().unwrap().to_string_lossy().to_string() + "." + runtime_name;

    new_filepath.push(new_file_stem);

    println!("Serializing precompiled bytes to {:?}", new_filepath);

    let mut file = File::create(new_filepath)?;

    file.write(&precompiled)?;

    Ok(())
}
