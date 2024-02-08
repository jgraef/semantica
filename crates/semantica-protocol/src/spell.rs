use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use uuid::Uuid;

use crate::user::{
    UserId,
    UserLink,
};

#[derive(Copy, Clone, Debug, Serialize, Deserialize, derive_more::From)]
pub struct RecipeId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Recipe {
    pub recipe_id: RecipeId,
    pub product: Option<SpellId>,
    pub ingredients: Vec<SpellId>,
    pub created_at: DateTime<Utc>,
    pub created_by: UserLink,
}

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
pub struct SpellId(pub Uuid);

impl SpellId {
    pub fn from_name(name: &str) -> Self {
        // todo: normalize (lower-case, trim), then hash (using murmur3) and convert
        // hash into uuid probably want to have that function in the server
        todo!();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spell {
    pub spell_id: SpellId,
    pub name: String,
    pub emoji: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: UserLink,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftingRequest {
    pub ingredients: Vec<SpellId>,
}

pub struct CraftingResponse {
    pub product: Spell,
    pub first_discovery: bool,
}
