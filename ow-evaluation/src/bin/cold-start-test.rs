use ow_evaluation::cold_start::cold_start_test;
use serde_json::json;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let responses = cold_start_test("hash", || {
        let random = rand::random::<i64>();
        json!({
            "iterations": 1000,
            "input": random.to_string(),
        })
    })
    .await;

    println!("{}", serde_json::to_string(&responses).unwrap());

    Ok(())
}
