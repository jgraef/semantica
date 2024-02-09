pub mod api;
pub mod error;
pub mod game;
pub mod utils;

use game::Game;
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
) -> Result<Game, shuttle_runtime::Error> {
    Game::new(pool, secrets)
        .await
        .map_err(CustomError::new)
        .map_err(Into::into)
}
