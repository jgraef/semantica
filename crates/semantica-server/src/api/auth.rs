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
        NewUserRequest,
        NewUserResponse,
        Secret,
    },
    error::ApiError,
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
        auth::create_auth_secret,
        Game,
    },
};

pub async fn login(
    State(game): State<Game>,
    session: Session,
    Json(auth_request): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, Error> {
    let mut transaction = game.transaction().await?;
    let auth_result = transaction.authenticate_user(auth_request).await?;
    transaction.commit().await?;

    if let Some(user_id) = auth_result {
        session.insert("user_id", user_id).await?;
        Ok(Json(AuthResponse { user_id }))
    }
    else {
        Err(ApiError::AuthenticationFailed.into())
    }
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
    let user_id: UserId = Uuid::new_v4().into();
    transaction
        .insert_user(user_id, &new_user_request.name, auth_secret.clone())
        .await?;
    transaction.commit().await?;
    session.insert("user_id", user_id).await?;

    Ok(Json(NewUserResponse {
        user_id,
        auth_secret,
    }))
}

/// extracts the UserId from the session
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
