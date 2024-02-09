use std::{
    borrow::Cow,
    collections::HashMap,
    hash::Hash,
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
use uuid::uuid;

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

pub fn use_user_logins() -> Storage<UserLogins> {
    use_storage(StorageKey::UserLogins)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserLogin {
    pub user_id: UserId,
    pub name: String,
    pub auth_secret: AuthSecret,
    #[serde(default)]
    pub login_link_noticed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserLogins {
    pub users: HashMap<UserId, UserLogin>,
    pub logged_in: Option<UserId>,
}

impl Default for UserLogins {
    fn default() -> Self {
        let mut users = HashMap::default();
        let user_id = uuid!("43d65ac1-2778-49e8-b28d-65c7334cec32").into();
        users.insert(
            user_id,
            UserLogin {
                user_id,
                name: "test".to_owned(),
                auth_secret: AuthSecret("1UePwNhkVj_MyoE7wfXlBgCH6zncFLYv".to_owned().into()),
                login_link_noticed: false,
            },
        );
        UserLogins {
            users,
            logged_in: None,
        }
    }
}
