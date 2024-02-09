use std::fmt::Debug;

use serde::{
    Deserialize,
    Serialize,
};

use crate::user::UserId;

/// Generic wrapper for secrets.
///
/// # TODO
///
/// - overwrite with zeroes on drop.
#[derive(
    Clone,
    Serialize,
    PartialEq,
    Deserialize,
    derive_more::From,
    derive_more::Display,
    derive_more::FromStr,
)]
#[serde(transparent)]
pub struct Secret<T>(pub T);

impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Secret").field(&"REDACTED").finish()
    }
}

#[derive(
    Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::Display, derive_more::FromStr,
)]
#[serde(transparent)]
pub struct AuthSecret(pub Secret<String>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthRequest {
    Secret {
        user_id: UserId,
        auth_secret: AuthSecret,
    },
}

/// # Note
///
/// the secret to authenticate further requests is returned as a cookie.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user_id: UserId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUserRequest {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUserResponse {
    pub user_id: UserId,
    pub auth_secret: AuthSecret,
}
