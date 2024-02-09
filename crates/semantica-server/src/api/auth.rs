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
use rand::{
    distributions::Slice,
    thread_rng,
    Rng,
};
use semantica_protocol::{
    auth::{
        AuthRequest,
        AuthResponse,
        AuthSecret,
        NewUserRequest,
        NewUserResponse,
        Secret,
    },
    error::ApiError,
    node::NodeId,
    user::UserId,
};
use tower_sessions::Session;
use uuid::{
    uuid,
    Uuid,
};

use crate::{
    error::Error,
    game::{
        Game,
        Transaction,
    },
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

    let password_ok = verify_auth_secret(row.auth_secret, auth_secret).await;

    tracing::debug!(?password_ok);

    Ok(password_ok.then_some(user_id))
}

pub async fn login(
    State(game): State<Game>,
    session: Session,
    Json(auth_request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, Error> {
    let mut transaction = game.transaction().await?;

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
    State(game): State<Game>,
    session: Session,
    Json(new_user_request): Json<NewUserRequest>,
) -> Result<Json<NewUserResponse>, Error> {
    let mut transaction = game.transaction().await?;

    let auth_secret = create_auth_secret();
    let auth_secret_hash = hash_auth_secret(auth_secret.clone()).await;

    let user_id: UserId = Uuid::new_v4().into();

    // todo
    let start_node: NodeId = sqlx::query_scalar!("SELECT node_id FROM root_nodes LIMIT 1")
        .fetch_one(transaction.db())
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
        ) VALUES ($1, $2, $3, $4, $4, $5)"#,
        user_id.0,
        &new_user_request.name,
        &auth_secret_hash,
        transaction.now_naive(),
        start_node.0,
    )
    .fetch_one(transaction.db())
    .await?;

    transaction.commit().await?;

    session.insert("user_id", user_id).await?;

    Ok(Json(NewUserResponse {
        user_id,
        auth_secret,
    }))
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

/// Extracts the UserId from the signed session cookie.
pub struct Authenticated(pub UserId);

#[async_trait]
impl FromRequestParts<Game> for Authenticated {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &Game) -> Result<Self, Error> {
        let get_user_id = move || {
            async move {
                let session = Session::from_request_parts(parts, state).await.ok()?;
                session.get("user_id").await.ok().flatten()
            }
        };

        Ok(Self(
            get_user_id()
                .await
                //.ok_or_else(|| ApiError::NotAuthenticated)?,
                .unwrap_or_else(|| uuid!("43d65ac1-2778-49e8-b28d-65c7334cec32").into()),
        ))
    }
}

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
