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
    derive_more::FromStr,
    derive_more::Display,
)]
#[serde(transparent)]
pub struct NodeId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node<CreatedBy: Links<UserId>, ForkSpell: Links<SpellId>> {
    pub node_id: NodeId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<ParentLink<ForkSpell>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub natural_child: Option<NodeId>,

    #[serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty")]
    pub fork_children: Vec<ForkLink<ForkSpell>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<CreatedBy>,

    pub content: Content,
}

impl<CreatedBy: Links<UserId>, ForkSpell: Links<SpellId>> Node<CreatedBy, ForkSpell> {
    pub fn parent_id(&self) -> Option<NodeId> {
        self.parent.as_ref().map(|p| p.node_id)
    }

    pub fn parent_fork(&self) -> Option<&Fork<ForkSpell>> {
        self.parent.as_ref().and_then(|p| p.fork.as_ref())
    }

    pub fn parent_position(&self) -> Option<usize> {
        self.parent_fork().map(|f| f.position)
    }

    pub fn created_with(&self) -> Option<&ForkSpell> {
        self.parent_fork().map(|f| &f.spell)
    }
}

pub type ResponseNode = Node<UserLink, Spell<UserLink>>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ParentLink<ForkSpell: Links<SpellId>> {
    pub node_id: NodeId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork: Option<Fork<ForkSpell>>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ForkLink<ForkSpell: Links<SpellId>> {
    pub node_id: NodeId,
    pub fork: Fork<ForkSpell>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Fork<Spell: Links<SpellId>> {
    pub position: usize,
    pub spell: Spell,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Paragraph {
    pub text: String,

    #[serde(default)]
    pub atoms: Vec<Atom>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Atom {
    pub start: usize,
    pub length: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeResponse {
    pub node: ResponseNode,
}
