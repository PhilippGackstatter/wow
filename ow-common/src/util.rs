use std::io::Cursor;

#[inline(always)]
pub fn b64_decode(b64_string: String) -> anyhow::Result<Vec<u8>> {
    let time = std::time::Instant::now();
    let module_bytes: Vec<u8> = base64::decode(b64_string)?;
    println!("base64 decoding took {} ms", time.elapsed().as_millis());
    Ok(module_bytes)
}

pub fn unzip(bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut target = Cursor::new(Vec::with_capacity(bytes.len()));
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).unwrap();

    let mut file = archive.by_index(0)?;

    std::io::copy(&mut file, &mut target)?;

    Ok(target.into_inner())
}
