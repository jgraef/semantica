use axum::{
    extract::State,
    Json,
};
use semantica_protocol::user::InventoryResponse;

use super::auth::Authenticated;
use crate::{
    error::Error,
    game::Game,
};

pub async fn get_inventory(
    State(game): State<Game>,
    Authenticated(user_id): Authenticated,
) -> Result<Json<InventoryResponse>, Error> {
    let mut transaction = game.transaction().await?;
    let inventory = transaction.fetch_inventory(user_id).await?;
    transaction.commit().await?;
    Ok(Json(InventoryResponse { inventory }))
}
