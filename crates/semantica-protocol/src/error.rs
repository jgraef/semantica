use serde::{
    Deserialize,
    Serialize,
};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("internal server error")]
    Internal,

    #[error("authentication failed")]
    AuthenticationFailed,
}
