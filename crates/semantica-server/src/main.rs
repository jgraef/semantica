pub mod ai;
pub mod api;
pub mod context;
pub mod error;

use axum::{
    routing::get,
    Router,
};
use context::Context;
use shuttle_axum::ShuttleAxum;
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://semantica:{secrets.DB_PASSWORD}@localhost/semantica"
    )]
    pool: PgPool,
    #[shuttle_secrets::Secrets] secrets: SecretStore,
) -> ShuttleAxum {
    let context = Context::new(pool, &secrets)
        .await
        .map_err(CustomError::new)?;

    let api = Router::new().route("/events", get(api::events::subscribe));

    let router = Router::new()
        .route("/", get(|| async { "Hello World" }))
        .nest("/api/v1", api)
        .with_state(context);

    Ok(router.into())
}
