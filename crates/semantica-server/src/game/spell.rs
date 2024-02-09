use chrono::NaiveDateTime;
use murmur3::Murmur3x64x128;
use semantica_protocol::{
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
use sqlx::prelude::FromRow;
use uuid::Uuid;

use super::Transaction;
use crate::{
    error::Error,
    utils::convert::{
        DbConversionError,
        FromDb,
        ToDb,
    },
};

pub fn get_spell_id_for_name(name: &str) -> SpellId {
    const SEED: u32 = 1;
    let hash = murmur3::hash::<Murmur3x64x128, _>(SEED, name.as_bytes());
    Uuid::from_u128(hash).into()
}

pub fn create_spell(name: String, emoji: String, description: String) -> Spell<UserId> {
    Spell {
        spell_id: get_spell_id_for_name(&name),
        name,
        emoji,
        description,
        created_at: None,
        created_by: None,
    }
}

impl<'a> Transaction<'a> {
    pub async fn insert_spell<CreatedBy: Links<UserId>>(
        &mut self,
        spell: &Spell<CreatedBy>,
    ) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO spells (
                spell_id,
                name,
                emoji,
                description,
                created_at,
                created_by
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            ToDb::<Uuid>::to_db(&spell.spell_id)?,
            &spell.name,
            &spell.emoji,
            &spell.description,
            ToDb::<Option<NaiveDateTime>>::to_db(&spell.created_at)?,
            ToDb::<Option<Uuid>>::to_db(&spell.created_by.as_ref().map(|link| link.id()))?,
        )
        .execute(self.db())
        .await?;
        Ok(())
    }

    pub async fn fetch_spell(&mut self, spell_id: SpellId) -> Result<Spell<UserLink>, Error> {
        // todo: join to get created_by.name
        let spell = sqlx::query_as!(
            SpellRow,
            "SELECT * FROM spells WHERE spell_id = $1",
            ToDb::<Uuid>::to_db(&spell_id)?
        )
        .fetch_one(self.db())
        .await?
        .from_db()?;
        Ok(spell)
    }
}

#[derive(FromRow)]
struct SpellRow {
    spell_id: Uuid,
    name: String,
    emoji: String,
    description: String,
    created_at: Option<NaiveDateTime>,
    created_by: Option<Uuid>,
}

impl FromDb<Spell<UserLink>> for SpellRow {
    fn from_db(self) -> Result<Spell<UserLink>, DbConversionError> {
        todo!();
    }
}
