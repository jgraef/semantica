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
    user::{
        UserId,
        UserLink,
    },
    Links,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Spell<CreatedBy: Links<UserId>> {
    pub spell_id: SpellId,
    pub name: String,
    pub emoji: String,
    pub description: String,
    pub created_at: Option<DateTime<Utc>>,
    pub created_by: Option<CreatedBy>,
}

impl<CreatedBy: Links<UserId>> Links<SpellId> for Spell<CreatedBy> {
    fn id(&self) -> SpellId {
        self.spell_id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpellAmount<Spell> {
    pub spell: Spell,
    pub amount: usize,
}

impl<Spell: Links<SpellId>> Links<SpellId> for SpellAmount<Spell> {
    fn id(&self) -> SpellId {
        self.spell.id()
    }
}

pub type ResponseSpellAmount = SpellAmount<Spell<UserLink>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CraftingRequest {
    pub ingredients: Vec<SpellId>,
}

pub struct CraftingResponse {
    pub product: Spell<UserLink>,
    pub first_discovery: bool,
}
