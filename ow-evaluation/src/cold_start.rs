use crate::SERVER_URI;

use async_std::eprintln;
use futures::{stream::FuturesUnordered, Future, StreamExt};
use serde::Serialize;
use serde_json::Value;

const ABORT_AFTER: usize = 11;

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
    action_name: &str,
    auth: &str,
    param: &mut F,
) -> FuturesUnordered<impl Future<Output = Result<Value, surf::Error>>>
where
    F: Fn() -> Value,
{
    let mut futures = Vec::with_capacity(count);

    for i in 0..count {
        let mut action = action_name.to_owned();
        action.push_str(&(i + 1).to_string());
        std::eprintln!("Executing action: {}", action);

        let path = format!(
            "{}/api/v1/namespaces/_/actions/{}?blocking=true&result=false",
            SERVER_URI, action
        );

        let parameters = param();
        let body = surf::Body::from_json(&parameters).unwrap();
        futures.push(make_request(&path, auth, body))
    }

    futures.into_iter().collect::<FuturesUnordered<_>>()
}

async fn collect_activation(activation_id: &str, auth: &str) -> Value {
    let path = format!(
        "{}/api/v1/namespaces/_/activations/{}",
        SERVER_URI, activation_id
    );

    surf::get(&path)
        .header("Authorization", format!("Basic {}", auth))
        .recv_json::<serde_json::Value>()
        .await
        .unwrap()
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
    let auth = base64::encode(std::env::var("WHISK_AUTH").unwrap());

    let mut cold_start_results = Vec::with_capacity(ABORT_AFTER);

    for i in 1..ABORT_AFTER {
        let mut futures = make_concurrent_requests(i, action_name, &auth, &mut param);

        let mut responses = Vec::with_capacity(ABORT_AFTER * 3 * ABORT_AFTER);

        eprintln!("Sending {} concurrent requests", i).await;

        let mut failed = false;

        let mut activation_ids = vec![];

        loop {
            // match async_std::future::timeout(std::time::Duration::from_secs(360), futures.next())
            match futures.next().await {
                // Ok(Some(res)) => match res {
                Some(response) => match response {
                    Ok(res) => {
                        if let Some(_) = res.get("response") {
                            eprintln!("Response: {:?}", res).await;
                            responses.push(res);
                        } else {
                            let activation_id = res.get("activationId").unwrap().as_str().unwrap();
                            activation_ids.push(activation_id.to_owned());
                        }
                    }
                    Err(err) => {
                        eprintln!("Err: {:?}", err).await;
                        failed = true;
                    }
                },
                None => break, // Err(err) => {
                               //     eprintln!("Err: {:?}", err).await;
                               //     failed = true;
                               // }
                               // },
                               // Ok(None) => {
                               //     break;
                               // }
                               // Err(_) => {
                               //     eprintln!("Timeout").await;
                               //     failed = true;
                               //     break;
                               // }
            }
        }

        if !activation_ids.is_empty() {
            eprintln!("Collect {num} activations?", num = activation_ids.len()).await;
            let stdin = async_std::io::stdin();
            let mut line = String::new();
            stdin.read_line(&mut line).await.unwrap();

            if line.starts_with("y") {
                for activation_id in activation_ids {
                    eprintln!("Collecting {activation_id}", activation_id = activation_id).await;
                    let response = collect_activation(&activation_id, &auth).await;
                    responses.push(response);
                }
            }
        }

        // while let Ok(Some(res)) =
        //     async_std::future::timeout(std::time::Duration::from_secs(60), futures.next()).await
        // {
        //     match res {
        //         Ok(response) => responses.push(response),
        //         Err(err) => {
        //             eprintln!("Err: {:?}", err).await;
        //             failed = true;
        //         }
        //     }
        // }

        eprintln!("Finished {} concurrent requests test", i).await;

        cold_start_results.push(ColdStartResult {
            no_concurrent_requests: i,
            responses,
        });

        if failed {
            eprintln!("aborting").await;
            break;
        }

        // Wait for containers to deallocate
        eprintln!("Continue?").await;
        let stdin = async_std::io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line).await.unwrap();

        if !line.starts_with("y") {
            break;
        }

        // Wait for containers to deallocate
        // async_std::task::sleep(std::time::Duration::new(40, 0)).await;
    }

    serde_json::to_value(cold_start_results).unwrap()
}
