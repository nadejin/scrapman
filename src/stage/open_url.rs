use crate::{
    pipeline::{ScrapePipelineStage, ScrapeResult},
    value::Value,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct OpenUrl {
    url: Value,
}

impl OpenUrl {
    pub fn new(url: Value) -> Self {
        OpenUrl { url }
    }
}

impl fmt::Display for OpenUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OpenUrl({})", self.url)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapePipelineStage for OpenUrl {
    async fn execute(
        &self,
        client: &mut fantoccini::Client,
        mut context: &mut crate::ScrapePipelineContext,
    ) -> Result<(), crate::ScrapeResult> {
        let url = self.url.resolve(&mut context).await?;
        client.goto(&url).await.map_err(ScrapeResult::WebdriverCommandError)
    }
}
