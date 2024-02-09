use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use semantica_protocol::error::ApiError;

use crate::api::AsStatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("axum")]
    Axum(#[from] axum::Error),

    #[error("sqlx")]
    Sqlx(#[from] sqlx::Error),

    #[error("sqlx migrate")]
    SqlxMigrate(#[from] sqlx::migrate::MigrateError),

    #[error("json")]
    Json(#[from] serde_json::Error),

    #[error("askama")]
    Askama(#[from] askama::Error),

    #[error("hf-api")]
    HfApi(#[from] hf_textgen::Error),

    #[error("api")]
    Api(#[from] ApiError),

    #[error("session")]
    TowerSession(#[from] tower_sessions::session::Error),

    #[error("password hash")]
    PasswordHash,

    #[error("db conversion")]
    DbConversion(#[from] crate::utils::convert::DbConversionError),

    #[error("io")]
    Io(#[from] std::io::Error),
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        match error {
            Error::Api(error) => error,
            _ => {
                let mut error: &dyn std::error::Error = &error;
                tracing::error!("returning internal server error: {error}");
                while let Some(source) = error.source() {
                    tracing::error!(" - {source}");
                    error = source;
                }
                ApiError::Internal
            }
        }
    }
}

impl From<argon2::password_hash::Error> for Error {
    fn from(_: argon2::password_hash::Error) -> Self {
        Self::PasswordHash
    }
}

impl AsStatusCode for Error {
    fn as_status_code(&self) -> StatusCode {
        match self {
            Error::Api(error) => error.as_status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let api_error = ApiError::from(self);
        let status_code = api_error.as_status_code();
        (status_code, Json(api_error)).into_response()
    }
}
