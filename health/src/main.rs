//! Checks whether HTTP endpoints are up.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

// The maximum number of requests in flight at the same time.
const MAX_CONCURRENT_CHECKS: usize = 2;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let urls: Vec<String> = std::env::args().skip(1).collect();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CHECKS));

    let mut tasks = tokio::task::JoinSet::new();

    for url in urls {
        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        tasks.spawn(async move {
            let _permit = semaphore
                .acquire()
                .await
                .expect("semaphore is never closed");
            let result = check(&client, &url).await;
            (url, result)
        });
    }

    while let Some(joined) = tasks.join_next().await {
        let (url, result) = joined.expect("check task should not panic");
        match result {
            Ok(status) => println!("{status} {url}"),
            Err(e) => println!("ERROR {url}: {e}"),
        }
    }

    Ok(())
}

async fn check(client: &reqwest::Client, url: &str) -> Result<reqwest::StatusCode, reqwest::Error> {
    let response = client.get(url).send().await?;
    Ok(response.status())
}
