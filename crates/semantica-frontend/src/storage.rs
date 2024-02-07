use std::{
    borrow::Cow,
    collections::HashMap,
};

use leptos::{
    Signal,
    WriteSignal,
};
use leptos_use::{
    storage::use_local_storage,
    utils::JsonCodec,
};
use semantica_protocol::{
    auth::AuthSecret,
    user::UserId,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Clone, Debug)]
pub enum StorageKey {
    UserLogins,
}

impl StorageKey {
    pub fn as_str(&self) -> Cow<'static, str> {
        match self {
            Self::UserLogins => "user-logins".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Storage<T: 'static> {
    pub key: StorageKey,
    pub value: Signal<T>,
    pub update_value: WriteSignal<T>,
}

pub fn use_storage<
    T: Serialize + for<'de> Deserialize<'de> + Clone + Default + PartialEq + 'static,
>(
    key: StorageKey,
) -> Storage<T> {
    let (value, update_value, _) = use_local_storage::<T, JsonCodec>(&key.as_str());
    Storage {
        key,
        value,
        update_value,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserLogin {
    pub user_id: UserId,
    pub name: String,
    pub auth_secret: AuthSecret,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UserLogins {
    pub users: HashMap<UserId, UserLogin>,
    pub logged_in: Option<UserId>,
}
