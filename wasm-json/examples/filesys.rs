use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

#[cfg(feature = "wasm")]
wasm_json::pass_json!(func);

#[cfg(feature = "bin")]
wasm_json::json_args!(func);

// Needs to be created with --annotation dir "/tmp/filesys"

const NUM_BYTES: usize = 5_000_000;
const PATH: &'static str = "/tmp/filesys/test.txt";

pub fn func(_json: serde_json::Value) -> Result<serde_json::Value, anyhow::Error> {
    let generation_time = std::time::Instant::now();
    let mut random_bytes: Vec<u8> = Vec::with_capacity(NUM_BYTES);

    let mut rng = Xoshiro256StarStar::seed_from_u64(0);

    for i in 0..NUM_BYTES {
        random_bytes.push(rng.gen());
    }

    let generation_time = generation_time.elapsed().as_millis();

    let write_read_time = std::time::Instant::now();

    std::fs::write(PATH, &random_bytes)?;

    let read_bytes = std::fs::read(PATH)?;

    Ok(serde_json::json!({
        "success": read_bytes == random_bytes,
        "generation_time": generation_time as u64,
        "write_read_time": write_read_time.elapsed().as_millis() as u64,
    }))
}
