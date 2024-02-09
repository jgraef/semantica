use futures::TryStreamExt;
use semantica_protocol::{
    spell::{
        Spell,
        SpellAmount,
    },
    user::{
        UserId,
        UserLink,
    },
};

use super::Transaction;
use crate::{
    error::Error,
    utils::convert::FromDb,
};

impl<'a> Transaction<'a> {
    pub async fn fetch_user_inventory(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<SpellAmount<Spell<UserLink>>>, Error> {
        let mut rows = sqlx::query!(
            r#"
            SELECT
                spells.spell_id AS spell_id,
                spells.name AS spell_name,
                spells.emoji AS spell_emoji,
                spells.description AS spell_description,
                spells.created_at AS spell_created_at,
                inventory_contents.amount AS amount,
                users.user_id AS "created_by?",
                users.name AS "created_by_name?"
            FROM inventory_contents
                INNER JOIN spells ON inventory_contents.spell_id = spells.spell_id
                LEFT OUTER JOIN users ON spells.created_by = users.user_id
            WHERE inventory_contents.user_id = $1
            "#,
            user_id.0,
        )
        .fetch(self.db());

        let mut inventory = vec![];
        while let Some(row) = rows.try_next().await? {
            inventory.push(SpellAmount {
                spell: Spell {
                    spell_id: row.spell_id.from_db()?,
                    name: row.spell_name,
                    emoji: row.spell_emoji,
                    description: row.spell_description,
                    created_at: row.spell_created_at.from_db()?,
                    created_by: match (row.created_by, row.created_by_name) {
                        (Some(user_id), Some(name)) => {
                            Some(UserLink {
                                user_id: user_id.from_db()?,
                                name,
                            })
                        }
                        _ => None,
                    },
                },
                amount: row.amount.from_db()?,
            });
        }

        Ok(inventory)
    }
}
