use std::sync::Arc;

use rand::{
    distributions::Slice,
    thread_rng,
    Rng,
};
use semantica_protocol::auth::Secret;
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

pub fn create_secret(length: usize) -> Secret<String> {
    // this is url-safe
    #[rustfmt::skip]
    pub const ALPHABET: [char; 64] = [
        '_', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
        'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
        'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    // slice is not empty, so the unwrap will never fail.
    let dist = Slice::new(&ALPHABET).unwrap();

    thread_rng()
        .sample_iter(dist)
        .take(length)
        .collect::<String>()
        .into()
}

#[derive(Debug)]
struct Inner {
    pool: PgPool,
    ai: Ai,
}

#[derive(Clone, Debug)]
pub struct Context {
    inner: Arc<Inner>,
}

impl Context {
    pub async fn new(pool: PgPool, secrets: &SecretStore) -> Result<Self, Error> {
        sqlx::migrate!("../../migrations").run(&pool).await?;

        let ai = Ai::new(secrets);

        Ok(Self {
            inner: Arc::new(Inner { pool, ai }),
        })
    }

    pub async fn transaction(&self) -> Result<Transaction, Error> {
        let transaction = self.inner.pool.begin().await?;
        Ok(Transaction {
            context: self.clone(),
            transaction,
        })
    }

    pub fn ai(&self) -> &Ai {
        &self.inner.ai
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
