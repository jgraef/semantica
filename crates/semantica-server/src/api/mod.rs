use axum::{
    http::StatusCode,
    routing::{
        any,
        get,
        post,
    },
    Router,
};
use semantica_protocol::error::ApiError;

pub mod auth;
pub mod crafting;
pub mod events;

use crate::{
    context::Context,
    error::Error,
};

pub trait AsStatusCode {
    fn as_status_code(&self) -> StatusCode;
}

impl AsStatusCode for ApiError {
    fn as_status_code(&self) -> StatusCode {
        match self {
            ApiError::AuthenticationFailed | ApiError::NotAuthenticated => StatusCode::UNAUTHORIZED,
            ApiError::Unknown | ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

pub fn routes() -> Router<Context> {
    Router::new()
        .route("/", get(index))
        .route("/login", post(auth::login))
        .route("/logout", get(auth::logout))
        .route("/register", post(auth::register))
        .route("/events", get(events::subscribe))
        .fallback(any(not_found))
}

async fn index() -> &'static str {
    "This is the Semantica API."
}

async fn not_found() -> Error {
    ApiError::NotFound.into()
}
