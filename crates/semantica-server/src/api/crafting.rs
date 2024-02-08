use axum::{
    extract::State,
    Json,
};
use chrono::{
    TimeZone,
    Utc,
};
use semantica_protocol::{
    spell::{
        CraftingRequest,
        CraftingResponse,
        Spell,
        SpellId,
    },
    user::UserLink,
};

use super::auth::Authenticated;
use crate::{
    context::Context,
    error::Error,
};

pub async fn craft(
    State(context): State<Context>,
    Authenticated(user_id): Authenticated,
    Json(crafting_request): Json<CraftingRequest>,
) -> Result<Json<CraftingResponse>, Error> {
    let mut transaction = context.transaction().await?;

    let mut ingredients = crafting_request
        .ingredients
        .into_iter()
        .map(|spell_id| spell_id.0)
        .collect::<Vec<_>>();
    ingredients.sort();

    let row = sqlx::query!(
        r#"
        SELECT
            spells.spell_id AS spell_id,
            spells.name AS spell_name,
            spells.emoji AS spell_emoji,
            spells.description AS spell_description,
            spells.created_at AS spell_created_at,
            spells.created_by AS spell_created_by,
            users.user_id AS created_by,
            users.name AS created_by_name
        FROM recipes
            INNER JOIN spells ON recipes.product = spells.spell_id
            INNER JOIN users ON spells.created_by = users.user_id
        WHERE recipes.ingredients = $1
        "#,
        &ingredients
    )
    .fetch_optional(transaction.db())
    .await?;

    if let Some(row) = row {
        Ok(Json(CraftingResponse {
            product: Spell {
                spell_id: row.spell_id.into(),
                name: row.spell_name,
                emoji: row.spell_emoji,
                description: row.spell_description,
                created_at: Utc.from_utc_datetime(&row.spell_created_at),
                created_by: UserLink {
                    user_id: row.created_by.into(),
                    name: row.created_by_name,
                },
            },
            first_discovery: false,
        }))
    }
    else {
        let mut rows = sqlx::query!(
            "SELECT spell_id, name FROM spells WHERE spell_id = ANY($1)",
            &ingredients
        )
        .fetch_all(transaction.db())
        .await?;
        rows.sort_by_key(|row| row.spell_id);
        let ingredient_names = rows.iter().map(|row| row.name.as_str()).collect::<Vec<_>>();

        let crafting_result = context.ai().craft(&ingredient_names).await?;

        let created_at = Utc::now();

        let row = sqlx::query!(
            r#"
            INSERT INTO spells
                (name, emoji, description, created_at, created_by)
            VALUES
                ($1, $2, $3, $4, $5)
            RETURNING spell_id
            "#,
            crafting_result.name,
            crafting_result.emoji,
            crafting_result.description,
            created_at.naive_utc(),
            user_id.0,
        )
        .fetch_one(transaction.db())
        .await?;
        let spell_id: SpellId = row.spell_id.into();

        sqlx::query!(
            r#"
            INSERT INTO recipes
                (product, ingredients)
            VALUES
                ($1, $2)
            "#,
            spell_id.0,
            &ingredients
        )
        .execute(transaction.db())
        .await?;

        let row = sqlx::query!("SELECT name FROM users WHERE user_id = $1", user_id.0)
            .fetch_one(transaction.db())
            .await?;
        let user_name = row.name;

        Ok(Json(CraftingResponse {
            product: Spell {
                spell_id,
                name: crafting_result.name,
                emoji: crafting_result.emoji,
                description: crafting_result.description,
                created_at,
                created_by: UserLink {
                    user_id,
                    name: user_name,
                },
            },
            first_discovery: true,
        }))
    }
}
