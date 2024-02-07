use std::sync::Arc;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use shuttle_secrets::SecretStore;
use sqlx::{
    PgConnection,
    PgPool,
    Postgres,
};

use crate::{
    ai::Ai,
    error::Error,
};

#[derive(Debug)]
struct Inner {
    pool: PgPool,
    ai: Ai,
    cookie_key: Key,
}

#[derive(Clone, Debug)]
pub struct Context {
    inner: Arc<Inner>,
}

impl Context {
    pub async fn new(pool: PgPool, secrets: &SecretStore) -> Result<Self, Error> {
        sqlx::migrate!("../../migrations").run(&pool).await?;

        let ai = Ai::new(secrets);

        let cookie_key = Key::generate();

        Ok(Self {
            inner: Arc::new(Inner {
                pool,
                ai,
                cookie_key,
            }),
        })
    }

    pub async fn transaction(&self) -> Result<Transaction, Error> {
        let transaction = self.inner.pool.begin().await?;
        Ok(Transaction {
            context: self.clone(),
            transaction,
        })
    }
}

pub struct Transaction<'a> {
    context: Context,
    transaction: sqlx::Transaction<'a, Postgres>,
}

impl<'a> Transaction<'a> {
    pub async fn commit(self) -> Result<(), Error> {
        self.transaction.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<(), Error> {
        self.transaction.rollback().await?;
        Ok(())
    }

    pub fn db(&mut self) -> &mut PgConnection {
        &mut *self.transaction
    }
}

impl FromRef<Context> for Key {
    fn from_ref(context: &Context) -> Self {
        context.inner.cookie_key.clone()
    }
}
