use argon2::{
    password_hash::SaltString,
    Argon2,
    PasswordHash,
    PasswordHasher,
    PasswordVerifier,
};
use async_trait::async_trait;
use axum::{
    extract::{
        FromRequestParts,
        State,
    },
    http::request::Parts,
    Json,
};
use rand::thread_rng;
use semantica_protocol::{
    auth::{
        AuthRequest,
        AuthResponse,
        AuthSecret,
        NewUserRequest,
        NewUserResponse,
    },
    error::ApiError,
    user::UserId,
};
use tower_sessions::Session;

use crate::{
    context::{
        create_secret,
        Context,
        Transaction,
    },
    error::Error,
};

pub fn create_auth_secret() -> AuthSecret {
    const LENGTH: usize = 32;
    AuthSecret(create_secret(LENGTH))
}

async fn auth_with_secret<'a>(
    transaction: &mut Transaction<'a>,
    user_id: UserId,
    auth_secret: AuthSecret,
) -> Result<Option<UserId>, Error> {
    let Some(row) = sqlx::query!(
        "SELECT auth_secret FROM users WHERE user_id = $1",
        user_id.0,
    )
    .fetch_optional(transaction.db())
    .await?
    else {
        tracing::debug!("user not found");
        return Ok(None);
    };

    let password_ok = tokio::task::spawn_blocking(move || {
        let Ok(auth_secret_hash) = PasswordHash::new(&row.auth_secret)
        else {
            tracing::warn!("failed to hash auth_secret");
            return false;
        };
        Argon2::default()
            .verify_password(auth_secret.0 .0.as_bytes(), &auth_secret_hash)
            .is_ok()
    })
    .await
    .unwrap();

    tracing::debug!(?password_ok);

    Ok(password_ok.then_some(user_id))
}

pub async fn login(
    State(context): State<Context>,
    session: Session,
    Json(auth_request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, Error> {
    let mut transaction = context.transaction().await?;

    tracing::debug!(?auth_request, "authentication request");

    let auth_result = match auth_request {
        AuthRequest::Secret {
            user_id,
            auth_secret,
        } => auth_with_secret(&mut transaction, user_id, auth_secret).await?,
    };

    let result = if let Some(user_id) = auth_result {
        sqlx::query!("UPDATE users SET last_login = utc_now()")
            .execute(transaction.db())
            .await?;

        session.insert("user_id", user_id).await?;

        Ok(Json(AuthResponse { user_id }))
    }
    else {
        Err(ApiError::AuthenticationFailed.into())
    };

    transaction.commit().await?;

    result
}

pub async fn logout(session: Session) -> Result<(), Error> {
    session.remove::<UserId>("user_id").await?;
    Ok(())
}

pub async fn register(
    State(context): State<Context>,
    session: Session,
    Json(new_user_request): Json<NewUserRequest>,
) -> Result<Json<NewUserResponse>, Error> {
    let mut transaction = context.transaction().await?;

    let auth_secret = create_auth_secret();

    let auth_secret_hash = {
        let auth_secret = auth_secret.clone();
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut thread_rng());
            Argon2::default()
                .hash_password(auth_secret.0 .0.as_bytes(), &salt)
                .unwrap()
                .to_string()
        })
        .await
        .unwrap()
    };

    let response = sqlx::query!(
        "INSERT INTO users (name, auth_secret) VALUES ($1, $2) RETURNING user_id",
        &new_user_request.name,
        &auth_secret_hash,
    )
    .fetch_one(transaction.db())
    .await?;

    transaction.commit().await?;

    let user_id: UserId = response.user_id.into();
    session.insert("user_id", user_id).await?;

    Ok(Json(NewUserResponse {
        user_id,
        auth_secret,
    }))
}

/// Extracts the UserId from the signed session cookie.
pub struct Authenticated(pub UserId);

#[async_trait]
impl FromRequestParts<Context> for Authenticated {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &Context) -> Result<Self, Error> {
        let get_user_id = move || {
            async move {
                let session = Session::from_request_parts(parts, state).await.ok()?;
                session.get("user_id").await.ok().flatten()
            }
        };

        Ok(Self(
            get_user_id()
                .await
                .ok_or_else(|| ApiError::NotAuthenticated)?,
        ))
    }
}
