use std::convert::Infallible;

use chrono::{
    DateTime,
    NaiveDateTime,
    TimeZone,
    Utc,
};
use semantica_protocol::{
    node::NodeId,
    spell::{
        RecipeId,
        SpellId,
    },
    user::UserId,
};
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum DbConversionError {
    #[error("number conversion")]
    Number(#[from] std::num::TryFromIntError),

    #[error("json")]
    Json(#[from] serde_json::Error),
}

impl From<Infallible> for DbConversionError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

pub trait FromDb<T> {
    fn from_db(self) -> Result<T, DbConversionError>;
}

pub trait ToDb<T> {
    fn to_db(&self) -> Result<T, DbConversionError>;
}

impl<T, U> FromDb<Option<U>> for Option<T>
where
    T: FromDb<U>,
{
    fn from_db(self) -> Result<Option<U>, DbConversionError> {
        self.map(|x| x.from_db()).transpose()
    }
}

impl<T, U> ToDb<Option<U>> for Option<T>
where
    T: ToDb<U>,
{
    fn to_db(&self) -> Result<Option<U>, DbConversionError> {
        self.as_ref().map(|x| x.to_db()).transpose()
    }
}

impl FromDb<DateTime<Utc>> for NaiveDateTime {
    fn from_db(self) -> Result<DateTime<Utc>, DbConversionError> {
        Ok(Utc.from_utc_datetime(&self))
    }
}

impl ToDb<NaiveDateTime> for DateTime<Utc> {
    fn to_db(&self) -> Result<NaiveDateTime, DbConversionError> {
        Ok(self.naive_utc())
    }
}

macro_rules! impl_id {
    ($ty:ident) => {
        impl FromDb<$ty> for Uuid {
            fn from_db(self) -> Result<$ty, DbConversionError> {
                Ok($ty(self))
            }
        }

        impl ToDb<Uuid> for $ty {
            fn to_db(&self) -> Result<Uuid, DbConversionError> {
                Ok(self.0)
            }
        }
    };
}

impl_id!(UserId);
impl_id!(NodeId);
impl_id!(SpellId);
impl_id!(RecipeId);

macro_rules! impl_number {
    ($num_ty:ident, $db_ty:ident) => {
        impl FromDb<$num_ty> for $db_ty {
            fn from_db(self) -> Result<$num_ty, DbConversionError> {
                self.try_into().map_err(Into::into)
            }
        }

        impl ToDb<$db_ty> for $num_ty {
            fn to_db(&self) -> Result<$db_ty, DbConversionError> {
                (*self).try_into().map_err(Into::into)
            }
        }
    };
}

impl_number!(usize, i32);

impl<T: for<'de> Deserialize<'de>> FromDb<T> for serde_json::Value {
    fn from_db(self) -> Result<T, DbConversionError> {
        serde_json::from_value(self).map_err(Into::into)
    }
}

impl<T: Serialize> ToDb<serde_json::Value> for T {
    fn to_db(&self) -> Result<serde_json::Value, DbConversionError> {
        serde_json::to_value(self).map_err(Into::into)
    }
}
