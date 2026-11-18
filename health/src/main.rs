//! Checks whether HTTP endpoints are up.

use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let url = "https://example.com";

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;
    let response = client.get(url).send().await?;
    let status = response.status();
    println!("{status} {url}");
    Ok(())
}
