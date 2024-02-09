use axum::{
    extract::{
        Path,
        Query,
        State,
    },
    Json,
};
use semantica_protocol::node::{
    NodeId,
    NodesParams,
    NodesResponse,
};

use super::auth::Authenticated;
use crate::{
    error::Error,
    game::Game,
};

const LIMIT_PARAGRAPHS_MAX: usize = 100;
const LIMIT_PARAGRAPHS_DEFAULT: usize = 20;
const LIMIT_NODES_MAX: usize = 5;
const LIMIT_NODES_DEFAULT: usize = 2;

pub async fn current_nodes(
    State(game): State<Game>,
    Authenticated(user_id): Authenticated,
    Query(query): Query<NodesParams>,
) -> Result<Json<NodesResponse>, Error> {
    let mut transaction = game.transaction().await?;

    let limit_paragraphs = std::cmp::min(
        LIMIT_PARAGRAPHS_MAX,
        query.limit_paragraphs.unwrap_or(LIMIT_PARAGRAPHS_DEFAULT),
    );
    let limit_nodes = std::cmp::min(
        LIMIT_NODES_MAX,
        query.limit_nodes.unwrap_or(LIMIT_NODES_DEFAULT),
    );

    let node = transaction.fetch_current_user_node(user_id).await?;
    let mut next_node = node.parent;
    let mut num_paragraphs = node.content.paragraphs.len();
    let mut num_nodes = 1;
    let mut nodes = Vec::with_capacity(limit_nodes);
    nodes.push(node);

    while num_paragraphs < limit_paragraphs && num_nodes < limit_nodes {
        let Some(node_id) = next_node
        else {
            break;
        };
        let node = transaction.fetch_node(node_id).await?;
        num_paragraphs += node.content.paragraphs.len();
        num_nodes += 1;
        next_node = node.parent;
        nodes.push(node);
    }

    transaction.commit().await?;

    Ok(Json(NodesResponse { nodes }))
}

pub async fn get_nodes(
    State(game): State<Game>,
    Path(node_id): Path<NodeId>,
    Query(query): Query<NodesParams>,
) -> Result<Json<NodesResponse>, Error> {
    let mut transaction = game.transaction().await?;

    let limit_paragraphs = std::cmp::min(
        LIMIT_PARAGRAPHS_MAX,
        query.limit_paragraphs.unwrap_or(LIMIT_PARAGRAPHS_DEFAULT),
    );
    let limit_nodes = std::cmp::min(
        LIMIT_NODES_MAX,
        query.limit_nodes.unwrap_or(LIMIT_NODES_DEFAULT),
    );

    let mut next_node = Some(node_id);
    let mut num_paragraphs = 0;
    let mut num_nodes = 0;
    let mut nodes = Vec::with_capacity(limit_nodes);

    while num_paragraphs < limit_paragraphs && num_nodes < limit_nodes {
        let Some(node_id) = next_node
        else {
            break;
        };
        let node = transaction.fetch_node(node_id).await?;
        num_paragraphs += node.content.paragraphs.len();
        num_nodes += 1;
        next_node = node.parent;
        nodes.push(node);
    }

    transaction.commit().await?;

    Ok(Json(NodesResponse { nodes }))
}
