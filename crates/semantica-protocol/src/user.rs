use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    derive_more::From,
    derive_more::Display, derive_more::FromStr
)]
pub struct UserId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: UserId,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserLink {
    pub user_id: UserId,
    pub name: String,
}
