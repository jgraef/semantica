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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpellId(pub Uuid);

#[derive(Clone, Debug, Serialize, Deserialize, derive_more::From)]
pub struct Spell {
    pub spell_id: SpellId,
    pub name: String,
    pub emoji: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: UserLink,
}
