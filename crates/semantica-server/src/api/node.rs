use axum::{
    extract::{
        Path,
        State,
    },
    Json,
};
use semantica_protocol::node::{
    NodeId,
    NodeResponse,
};

use super::auth::Authenticated;
use crate::{
    error::Error,
    game::Game,
};

pub async fn current_node(
    State(game): State<Game>,
    Authenticated(user_id): Authenticated,
) -> Result<Json<NodeResponse>, Error> {
    let mut transaction = game.transaction().await?;
    let node = transaction.fetch_current_user_node(user_id).await?;
    transaction.commit().await?;
    Ok(Json(NodeResponse { node }))
}

pub async fn get_node(
    State(game): State<Game>,
    Path(node_id): Path<NodeId>,
) -> Result<Json<NodeResponse>, Error> {
    let mut transaction = game.transaction().await?;
    let node = transaction.fetch_node(node_id).await?;
    transaction.commit().await?;
    Ok(Json(NodeResponse { node }))
}
