#[inline(always)]
pub fn b64_decode(b64_string: String) -> anyhow::Result<Vec<u8>> {
    let time = std::time::Instant::now();
    let module_bytes: Vec<u8> = base64::decode(b64_string)?;
    println!("base64 decoding took {} ms", time.elapsed().as_millis());
    Ok(module_bytes)
}
