use crate::SERVER_URI;

use async_std::eprintln;
use futures::{stream::FuturesUnordered, Future, StreamExt};
use serde_json::Value;

static mut SERVER_ACTION_URI: String = String::new();
const ABORT_AFTER: usize = 15;
const INCREASE_AFTER: u128 = 5000;
static mut WHISK_AUTH: String = String::new();

pub fn make_request(body: surf::Body) -> impl Future<Output = Result<Value, surf::Error>> {
    let uri = unsafe { &SERVER_ACTION_URI };
    let auth = unsafe { &WHISK_AUTH };

    let request = surf::post(uri)
        .body(body)
        .header("Authorization", format!("Basic {}", auth))
        .recv_json::<serde_json::Value>();

    request
}
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ConcurrencyResult {
    pub no_concurrent_requests: usize,
    pub responses: Vec<Value>,
}

pub async fn concurrency_test<F>(action_name: &str, param: F) -> Value
where
    F: Fn() -> Value,
{
    let path = format!(
        "{}/api/v1/namespaces/_/actions/{}?blocking=true&result=false",
        SERVER_URI, action_name
    );

    unsafe { SERVER_ACTION_URI = path };

    let auth = base64::encode(std::env::var("WHISK_AUTH").unwrap());
    unsafe {
        WHISK_AUTH = auth;
    }

    let parameters = param();
    let body = surf::Body::from_json(&parameters).unwrap();

    let requests = vec![make_request(body)];

    let mut futures = requests.into_iter().collect::<FuturesUnordered<_>>();

    let mut responses = Vec::with_capacity(ABORT_AFTER * 3 * ABORT_AFTER);
    let mut concurrency_results = Vec::with_capacity(ABORT_AFTER);

    let mut num_added = 1;
    let mut last_added = std::time::Instant::now();

    while let Some(res) = futures.next().await {
        match res {
            Ok(response) => responses.push(response),
            Err(err) => eprintln!("Err: {:?}", err).await,
        }

        if num_added > ABORT_AFTER {
            eprintln!("Aborting after {} additions", num_added).await;
            break;
        } else if last_added.elapsed().as_millis() > INCREASE_AFTER {
            concurrency_results.push(ConcurrencyResult {
                no_concurrent_requests: num_added,
                responses,
            });

            responses = Vec::with_capacity(ABORT_AFTER);

            let parameters = param();
            let body = surf::Body::from_json(&parameters).unwrap();
            futures.push(make_request(body));

            last_added = std::time::Instant::now();
            num_added += 1;

            eprintln!("Issuing {} concurrent requests", num_added).await;
        }

        let parameters = param();
        let body = surf::Body::from_json(&parameters).unwrap();
        futures.push(make_request(body));
    }

    serde_json::to_value(concurrency_results).unwrap()
}
