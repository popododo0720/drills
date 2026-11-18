//! Checks whether HTTP endpoints are up.

use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let urls: Vec<String> = std::env::args().skip(1).collect();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;

    let mut tasks = tokio::task::JoinSet::new();

    for url in urls {
        let client = client.clone();
        tasks.spawn(async move {
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
