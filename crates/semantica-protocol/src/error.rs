use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("unknown")]
    Unknown,

    #[error("internal server error")]
    Internal,

    #[error("resource not found")]
    NotFound,

    #[error("authentication failed")]
    AuthenticationFailed,

    #[error("not authenticated")]
    NotAuthenticated,
}
