mod client;
mod stream;

pub use client::Client;
use reqwest::StatusCode;
use semantica_protocol::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http")]
    Reqwest(#[from] reqwest::Error),

    #[error("http sse")]
    EventSource(#[from] eventsource_stream::EventStreamError<reqwest::Error>),

    #[error("http sse json")]
    EventJson(serde_json::Error),

    #[error("api: {status_code}")]
    Api {
        status_code: StatusCode,
        api_error: ApiError,
    },
}
