use tide::prelude::*;
use tide::Request;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    app.at("/block").get(block);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

pub async fn block(mut _req: tide::Request<()>) -> tide::Result<serde_json::Value> {
    let thread_id = std::thread::current().id();

    async_std::task::sleep(std::time::Duration::new(1, 0)).await;

    Ok(json!({
        "thread_id": format!("{:?}", thread_id),
    }))
}
