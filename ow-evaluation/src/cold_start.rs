use crate::SERVER_URI;

use async_std::eprintln;
use futures::{stream::FuturesUnordered, Future, StreamExt};
use serde::Serialize;
use serde_json::Value;

const ABORT_AFTER: usize = 10;

pub fn make_request(
    uri: &str,
    auth: &str,
    body: surf::Body,
) -> impl Future<Output = Result<Value, surf::Error>> {
    let request = surf::post(uri)
        .body(body)
        .header("Authorization", format!("Basic {}", auth))
        .recv_json::<serde_json::Value>();

    request
}

pub fn make_concurrent_requests<F>(
    count: usize,
    uri: &str,
    auth: &str,
    param: &mut F,
) -> FuturesUnordered<impl Future<Output = Result<Value, surf::Error>>>
where
    F: Fn() -> Value,
{
    let mut futures = Vec::with_capacity(count);

    for _ in 0..count {
        let parameters = param();
        let body = surf::Body::from_json(&parameters).unwrap();
        futures.push(make_request(uri, auth, body))
    }

    futures.into_iter().collect::<FuturesUnordered<_>>()
}

#[derive(Serialize, Debug)]
pub struct ColdStartResult {
    pub no_concurrent_requests: usize,
    pub responses: Vec<Value>,
}

pub async fn cold_start_test<F>(action_name: &str, mut param: F) -> Value
where
    F: Fn() -> Value,
{
    let path = format!(
        "{}/api/v1/namespaces/_/actions/{}?blocking=true&result=false",
        SERVER_URI, action_name
    );

    let auth = base64::encode(std::env::var("WHISK_AUTH").unwrap());

    let mut cold_start_results = Vec::with_capacity(ABORT_AFTER);

    for i in 1..ABORT_AFTER {
        let mut futures = make_concurrent_requests(i, &path, &auth, &mut param);

        let mut responses = Vec::with_capacity(ABORT_AFTER * 3 * ABORT_AFTER);

        eprintln!("Sending {} concurrent requests", i).await;

        while let Some(res) = futures.next().await {
            match res {
                Ok(response) => responses.push(response),
                Err(err) => eprintln!("Err: {:?}", err).await,
            }
        }

        eprintln!("Finished {} concurrent requests test", i).await;

        cold_start_results.push(ColdStartResult {
            no_concurrent_requests: i,
            responses,
        });

        // Wait for containers to deallocate
        async_std::task::sleep(std::time::Duration::new(15, 0)).await;
    }

    serde_json::to_value(cold_start_results).unwrap()
}
