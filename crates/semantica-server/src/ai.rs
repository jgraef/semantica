use askama::Template;
use hf_textgen::{
    Api,
    TextGeneration,
};
use serde::{
    Deserialize,
    Serialize,
};
use shuttle_secrets::SecretStore;

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct Ai {
    api: Api,
    crafting_model: TextGeneration,
    world_model: TextGeneration,
}

impl Ai {
    pub fn new(secrets: &SecretStore) -> Self {
        let mut builder = Api::builder();
        if let Some(hf_token) = secrets.get("HF_TOKEN") {
            builder = builder.with_hf_token(hf_token)
        }
        else {
            tracing::warn!("HF_TOKEN not set");
        }
        let api = builder.build();

        let crafting_model = api.text_generation("NousResearch/Nous-Hermes-2-Mixtral-8x7B-DPO");
        let world_model = api.text_generation("todo");

        Self {
            api,
            crafting_model,
            world_model,
        }
    }

    pub async fn craft(&self, ingredients: &[&str]) -> Result<CraftingProduct, Error> {
        #[derive(Debug, Template)]
        #[template(path = "crafting_prompt.txt")]
        struct CraftingPrompt<'a> {
            ingredients: &'a [&'a str],
        }

        let prompt = CraftingPrompt { ingredients }.render()?;

        let response = self.crafting_model.generate(&prompt).await?;

        let product: CraftingProduct = serde_json::from_str(&response)?;

        Ok(product)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CraftingProduct {
    pub thing: String,
    pub emoji: String,
    pub description: String,
}
