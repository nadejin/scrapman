use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::ScrapeContext,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt, mem::swap};

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreModel;

impl fmt::Display for StoreModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "store model")
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for StoreModel {
    async fn execute(&self, context: &mut ScrapeContext) -> ScrapeActionResult {
        let mut model = json!({});
        swap(&mut model, &mut context.model);
        Ok(context.models.push(model))
    }
}
