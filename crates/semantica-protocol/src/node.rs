use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

use crate::{
    spell::{
        Spell,
        SpellId,
    },
    user::{
        User,
        UserId,
        UserLink,
    },
    Links,
};

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    derive_more::From,
)]
#[serde(transparent)]
pub struct NodeId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node<CreatedBy: Links<UserId>, CreatedWith: Links<SpellId>> {
    pub node_id: NodeId,
    pub parent: Option<NodeId>,
    pub parent_position: Option<usize>,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by: Option<CreatedBy>,
    pub created_with: Option<CreatedWith>,
    pub content: Content,
}

pub type ResponseNode = Node<UserLink, Spell<UserLink>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Paragraph {
    pub text: String,
    pub atoms: Vec<Atom>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Atom {
    pub start: usize,
    pub length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodesParams {
    pub limit_paragraphs: Option<usize>,
    pub limit_nodes: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodesResponse {
    pub nodes: Vec<ResponseNode>,
}
