use crate::pipeline::ScrapePipelineStage;
use async_trait::async_trait;
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
impl ScrapePipelineStage for StoreModel {
    async fn execute(
        &self,
        _client: &mut fantoccini::Client,
        context: &mut crate::ScrapePipelineContext,
    ) -> Result<(), crate::ScrapeResult> {
        let mut model = json!({});
        swap(&mut model, &mut context.model);
        context.models.push(model);
        Ok(())
    }
}
