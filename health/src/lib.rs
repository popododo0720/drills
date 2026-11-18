//! Checks whether HTTP endpoints are up.

use std::time::{Duration, Instant};

use reqwest::StatusCode;

/// The result of checking a single URL.
#[derive(Debug)]
pub struct Report {
    /// The URL that was checked.
    pub url: String,
    /// How long the check took.
    pub elapsed: Duration,
    /// What happened during the check.
    pub outcome: Outcome,
}

/// What happened when a URL was checked.
#[derive(Debug)]
pub enum Outcome {
    /// The server responded with a success status.
    Up(StatusCode),
    /// The server responded with a non-success status.
    Down(StatusCode),
    /// The request did not finish in time.
    Timeout,
    /// The request failed before a response was received.
    Error(reqwest::Error),
}

/// Sends a GET request to `url` and reports how it went.
pub async fn check(client: &reqwest::Client, url: String) -> Report {
    let start = Instant::now();

    let outcome = match client.get(&url).send().await {
        Ok(response) if response.status().is_success() => Outcome::Up(response.status()),
        Ok(response) => Outcome::Down(response.status()),
        Err(e) if e.is_timeout() => Outcome::Timeout,
        Err(e) => Outcome::Error(e),
    };

    let elapsed = start.elapsed();
    Report {
        url,
        elapsed,
        outcome,
    }
}
