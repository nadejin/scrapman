use crate::{
    action::ScrapeAction,
    pipeline::{ScrapeContext, ScrapeResult},
};
use async_trait::async_trait;
use fantoccini::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fmt, mem::swap};

#[derive(Serialize, Deserialize)]
pub struct StoreModel;

impl fmt::Display for StoreModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StoreModel")
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for StoreModel {
    async fn execute(&self, _: &mut Client, context: &mut ScrapeContext) -> ScrapeResult {
        let mut model = json!({});
        swap(&mut model, &mut context.model);
        context.models.push(model);
        Ok(())
    }
}
