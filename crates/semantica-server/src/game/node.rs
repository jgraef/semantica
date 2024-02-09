use chrono::NaiveDateTime;
use lazy_static::lazy_static;
use regex::Regex;
use semantica_protocol::{
    node::{
        Atom,
        Content,
        Node,
        NodeId,
        Paragraph,
        ResponseNode,
    },
    spell::{
        Spell,
        SpellId,
    },
    user::{
        UserId,
        UserLink,
    },
};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    error::Error,
    game::Transaction,
    utils::{
        bug,
        convert::{
            DbConversionError,
            FromDb,
            ToDb,
        },
    },
};

type CreateNode = Node<UserId, SpellId>;

pub fn create_root_node(content: Content) -> CreateNode {
    CreateNode {
        node_id: Uuid::new_v4().into(),
        parent: None,
        parent_position: None,
        created_at: None,
        created_by: None,
        created_with: None,
        content: content,
    }
}

pub fn create_node_content(text: &str) -> Content {
    lazy_static! {
        static ref WORD: Regex = r"\b\w+\b".parse().unwrap();
    }

    let mut paragraphs = vec![];

    for line in text.lines() {
        let line = line.trim();

        if !line.is_empty() {
            let mut atoms = vec![];

            for m in WORD.find_iter(&text) {
                atoms.push(Atom {
                    start: m.start(),
                    length: m.len(),
                });
            }

            paragraphs.push(Paragraph {
                text: line.to_owned(),
                atoms,
            });
        }
    }

    Content { paragraphs }
}

#[derive(FromRow)]
struct NodeRow {
    node_id: Uuid,
    content: serde_json::Value,
    parent_id: Option<Uuid>,
    parent_position: Option<i32>,
    created_at: Option<NaiveDateTime>,

    created_by_user_id: Option<Uuid>,
    created_by_name: Option<String>,

    created_with_spell_id: Option<Uuid>,
    created_with_name: Option<String>,
    created_with_emoji: Option<String>,
    created_with_description: Option<String>,
    created_with_created_at: Option<NaiveDateTime>,

    created_with_created_by_user_id: Option<Uuid>,
    created_with_created_by_name: Option<String>,
}

impl FromDb<ResponseNode> for NodeRow {
    fn from_db(self) -> Result<ResponseNode, DbConversionError> {
        Ok(ResponseNode {
            node_id: self.node_id.from_db()?,
            parent: self.parent_id.from_db()?,
            parent_position: self.parent_position.from_db()?,
            created_at: self.created_at.from_db()?,
            created_by: match (self.created_by_user_id, self.created_by_name) {
                (Some(user_id), Some(name)) => {
                    Some(UserLink {
                        user_id: user_id.from_db()?,
                        name,
                    })
                }
                (None, None) => None,
                _ => bug!(),
            },
            created_with: match (
                self.created_with_spell_id,
                self.created_with_name,
                self.created_with_emoji,
                self.created_with_description,
            ) {
                (Some(spell_id), Some(name), Some(emoji), Some(description)) => {
                    let created_by = match (
                        self.created_with_created_by_user_id,
                        self.created_with_created_by_name,
                    ) {
                        (Some(user_id), Some(name)) => {
                            Some(UserLink {
                                user_id: user_id.from_db()?,
                                name,
                            })
                        }
                        (None, None) => None,
                        _ => bug!(),
                    };
                    Some(Spell {
                        spell_id: spell_id.from_db()?,
                        name,
                        emoji,
                        description,
                        created_at: self.created_with_created_at.from_db()?,
                        created_by,
                    })
                }
                (None, None, None, None) => None,
                _ => bug!(),
            },
            content: self.content.from_db()?,
        })
    }
}

impl<'a> Transaction<'a> {
    pub async fn insert_node(&mut self, node: &CreateNode) -> Result<(), Error> {
        sqlx::query!(
            r#"
            INSERT INTO nodes (
                node_id,
                content,
                parent_id,
                parent_position,
                created_at,
                created_by,
                created_with
            ) VALUES (
                $1,
                $2,
                $3,
                $4,
                $5,
                $6,
                $7
            )
            "#,
            ToDb::<Uuid>::to_db(&node.node_id)?,
            ToDb::<serde_json::Value>::to_db(&node.content)?,
            ToDb::<Option<Uuid>>::to_db(&node.parent)?,
            ToDb::<Option<i32>>::to_db(&node.parent_position)?,
            ToDb::<Option<NaiveDateTime>>::to_db(&node.created_at)?,
            ToDb::<Option<Uuid>>::to_db(&node.created_by)?,
            ToDb::<Option<Uuid>>::to_db(&node.created_with)?,
        )
        .execute(self.db())
        .await?;

        if node.parent.is_none() {
            sqlx::query!(
                "INSERT INTO root_nodes (node_id) VALUES ($1)",
                node.node_id.0
            )
            .execute(self.db())
            .await?;
        }

        Ok(())
    }

    pub async fn fetch_current_user_node(
        &mut self,
        user_id: UserId,
    ) -> Result<ResponseNode, Error> {
        Ok(sqlx::query_as!(
            NodeRow,
            r#"
            SELECT
                nodes.node_id AS node_id,
                nodes.content AS content,
                nodes.parent_id AS parent_id,
                nodes.parent_position AS parent_position,
                nodes.created_at AS created_at,
    
                users_created_by.user_id AS created_by_user_id,
                users_created_by.name AS created_by_name,
    
                spells_created_with.spell_id AS created_with_spell_id,
                spells_created_with.name AS created_with_name,
                spells_created_with.emoji AS created_with_emoji,
                spells_created_with.description AS created_with_description,
                spells_created_with.created_at AS created_with_created_at,
    
                users_created_with_created_by.user_id AS created_with_created_by_user_id,
                users_created_with_created_by.name AS created_with_created_by_name
            FROM users
                INNER JOIN nodes ON users.in_node = nodes.node_id
                LEFT OUTER JOIN users AS users_created_by ON nodes.created_by = users_created_by.user_id
                LEFT OUTER JOIN spells AS spells_created_with ON nodes.created_with = spells_created_with.spell_id
                LEFT OUTER JOIN users AS users_created_with_created_by ON spells_created_with.created_by = users_created_with_created_by.user_id
            WHERE users.user_id = $1
            LIMIT 1
            "#,
            user_id.0,
        ).fetch_one(self.db()).await?.from_db()?)
    }

    pub async fn fetch_node(&mut self, node_id: NodeId) -> Result<ResponseNode, Error> {
        Ok(sqlx::query_as!(
            NodeRow,
            r#"
            SELECT
                nodes.node_id AS node_id,
                nodes.content AS content,
                nodes.parent_id AS parent_id,
                nodes.parent_position AS parent_position,
                nodes.created_at AS created_at,
    
                users_created_by.user_id AS created_by_user_id,
                users_created_by.name AS created_by_name,
    
                spells_created_with.spell_id AS created_with_spell_id,
                spells_created_with.name AS created_with_name,
                spells_created_with.emoji AS created_with_emoji,
                spells_created_with.description AS created_with_description,
                spells_created_with.created_at AS created_with_created_at,
    
                users_created_with_created_by.user_id AS created_with_created_by_user_id,
                users_created_with_created_by.name AS created_with_created_by_name
            FROM nodes
                LEFT OUTER JOIN users AS users_created_by ON nodes.created_by = users_created_by.user_id
                LEFT OUTER JOIN spells AS spells_created_with ON nodes.created_with = spells_created_with.spell_id
                LEFT OUTER JOIN users AS users_created_with_created_by ON spells_created_with.created_by = users_created_with_created_by.user_id
            WHERE nodes.node_id = $1
            LIMIT 1
            "#,
            node_id.0,
        ).fetch_one(self.db()).await?.from_db()?)
    }
}
