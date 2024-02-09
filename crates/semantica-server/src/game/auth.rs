use argon2::{
    password_hash::SaltString,
    Argon2,
    PasswordHash,
    PasswordHasher,
    PasswordVerifier,
};
use rand::thread_rng;
use semantica_protocol::{
    auth::{
        AuthRequest,
        AuthSecret,
    },
    node::NodeId,
    user::UserId,
};

use super::Transaction;
use crate::{
    api::auth::create_secret,
    error::Error,
};

impl<'a> Transaction<'a> {
    pub async fn insert_user(
        &mut self,
        user_id: UserId,
        name: &str,
        auth_secret: AuthSecret,
    ) -> Result<(), Error> {
        let auth_secret_hash = hash_auth_secret(auth_secret).await;

        // todo
        let start_node: NodeId = sqlx::query_scalar!("SELECT node_id FROM root_nodes LIMIT 1")
            .fetch_one(self.db())
            .await?
            .into();

        sqlx::query!(
            r#"
            INSERT INTO users (
                user_id,
                name,
                auth_secret,
                created_at,
                last_login,
                in_node
            ) VALUES ($1, $2, $3, utc_now(), utc_now(), $4)"#,
            user_id.0,
            name,
            &auth_secret_hash,
            start_node.0,
        )
        .execute(self.db())
        .await?;

        Ok(())
    }

    async fn authenticate_user_with_secret(
        &mut self,
        user_id: UserId,
        auth_secret: AuthSecret,
    ) -> Result<bool, Error> {
        let Some(row) = sqlx::query!(
            "SELECT auth_secret FROM users WHERE user_id = $1",
            user_id.0,
        )
        .fetch_optional(self.db())
        .await?
        else {
            tracing::debug!("user not found");
            return Ok(false);
        };

        let password_ok = verify_auth_secret(row.auth_secret, auth_secret).await;

        tracing::debug!(?password_ok);

        Ok(password_ok)
    }

    pub async fn authenticate_user(
        &mut self,
        auth_request: AuthRequest,
    ) -> Result<Option<UserId>, Error> {
        let auth_result = match auth_request {
            AuthRequest::Secret {
                user_id,
                auth_secret,
            } => {
                self.authenticate_user_with_secret(user_id, auth_secret)
                    .await?
                    .then_some(user_id)
            }
        };

        if let Some(user_id) = auth_result {
            sqlx::query!("UPDATE users SET last_login = utc_now()")
                .execute(self.db())
                .await?;

            Ok(Some(user_id))
        }
        else {
            Ok(None)
        }
    }
}

pub fn create_auth_secret() -> AuthSecret {
    const LENGTH: usize = 32;
    AuthSecret(create_secret(LENGTH))
}

pub async fn hash_auth_secret(auth_secret: AuthSecret) -> String {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut thread_rng());
        Argon2::default()
            .hash_password(auth_secret.0 .0.as_bytes(), &salt)
            .unwrap()
            .to_string()
    })
    .await
    .unwrap()
}

pub async fn verify_auth_secret(auth_secret_hash: String, auth_secret: AuthSecret) -> bool {
    tokio::task::spawn_blocking(move || {
        let Ok(auth_secret_hash) = PasswordHash::new(&auth_secret_hash)
        else {
            tracing::warn!("failed to hash auth_secret");
            return false;
        };
        Argon2::default()
            .verify_password(auth_secret.0 .0.as_bytes(), &auth_secret_hash)
            .is_ok()
    })
    .await
    .unwrap()
}
