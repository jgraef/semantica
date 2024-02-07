use argon2::{
    password_hash::SaltString,
    Argon2,
    PasswordHash,
    PasswordHasher,
    PasswordVerifier,
};
use axum::{
    extract::State,
    Json,
};
use axum_extra::extract::{
    cookie::Cookie,
    SignedCookieJar,
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
    user::UserId,
};

use crate::{
    context::{
        Context,
        Transaction,
    },
    error::Error,
};

fn create_secret(length: usize) -> Secret {
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

pub fn create_auth_secret() -> AuthSecret {
    const LENGTH: usize = 32;
    AuthSecret(create_secret(LENGTH))
}

async fn auth_with_secret<'a>(
    transaction: &mut Transaction<'a>,
    user_id: UserId,
    auth_secret: &AuthSecret,
) -> Result<Option<UserId>, Error> {
    let Some(row) = sqlx::query!(
        "SELECT auth_secret FROM users WHERE user_id = $1",
        user_id.0,
    )
    .fetch_optional(transaction.db())
    .await?
    else {
        return Ok(None);
    };

    let auth_secret_hash = PasswordHash::new(&row.auth_secret)?;

    if Argon2::default()
        .verify_password(auth_secret.0 .0.as_bytes(), &auth_secret_hash)
        .is_err()
    {
        return Ok(None);
    }

    Ok(Some(user_id))
}

pub async fn auth(
    State(context): State<Context>,
    Json(auth_request): Json<AuthRequest>,
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, AuthResponse), Error> {
    let mut transaction = context.transaction().await?;

    let auth_result = match auth_request {
        AuthRequest::Secret {
            user_id,
            auth_secret,
        } => auth_with_secret(&mut transaction, user_id, &auth_secret).await?,
    };

    if let Some(user_id) = auth_result {
        sqlx::query!("UPDATE users SET last_login = utc_now()")
            .execute(transaction.db())
            .await?;

        Ok((
            jar.add(Cookie::new("user_id", user_id.0.to_string())),
            AuthResponse { user_id },
        ))
    }
    else {
        Err(ApiError::AuthenticationFailed.into())
    }
}

pub async fn register(
    State(context): State<Context>,
    Json(new_user_request): Json<NewUserRequest>,
    jar: SignedCookieJar,
) -> Result<(SignedCookieJar, NewUserResponse), Error> {
    let mut transaction = context.transaction().await?;

    let auth_secret = create_auth_secret();

    let salt = SaltString::generate(&mut thread_rng());
    let auth_secret_hash = Argon2::default()
        .hash_password(auth_secret.0 .0.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let response = sqlx::query!(
        "INSERT INTO users (name, auth_secret) VALUES ($1, $2) RETURNING user_id",
        &new_user_request.name,
        &auth_secret_hash,
    )
    .fetch_one(transaction.db())
    .await?;

    transaction.commit().await?;

    let user_id: UserId = response.user_id.into();

    Ok((
        jar.add(Cookie::new("user_id", user_id.0.to_string())),
        NewUserResponse {
            user_id,
            auth_secret,
        },
    ))
}
