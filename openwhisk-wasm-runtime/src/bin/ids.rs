use std::time::Instant;

use async_std::{eprintln, println};
use futures::{stream::FuturesUnordered, StreamExt};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let num_requests = num_cpus::get() * 2;

    let mut requests = Vec::with_capacity(num_requests);

    for _ in 0..num_requests {
        let request = surf::get("http://127.0.0.1:9000/test").recv_json::<serde_json::Value>();

        requests.push(request);
    }

    let mut futures = requests.into_iter().collect::<FuturesUnordered<_>>();

    let before = Instant::now();

    while let Some(res) = futures.next().await {
        match res {
            Ok(json) => {
                println!("{}", json.get("thread_id").unwrap()).await;
            }
            Err(err) => {
                eprintln!("{:?}", err).await;
            }
        }
    }

    println!("Passed time: {}", before.elapsed().as_millis()).await;

    Ok(())
}
