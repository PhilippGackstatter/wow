use ow_evaluation::concurrency::concurrency_test;
use serde_json::json;

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let responses = concurrency_test("hash", || {
        let random = rand::random::<i64>();
        json!({
            "iterations": 100000,
            "input": random.to_string(),
        })
    })
    .await;

    println!("{}", serde_json::to_string(&responses).unwrap());

    Ok(())
}
