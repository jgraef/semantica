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
#[derive(Clone, Serialize, PartialEq, Deserialize, derive_more::From)]
#[serde(transparent)]
pub struct Secret(pub String);

impl Secret {
    pub fn unwrap(self) -> String {
        self.0
    }
}

impl Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Secret").field(&"xxxx").finish()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AuthSecret(pub Secret);

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
