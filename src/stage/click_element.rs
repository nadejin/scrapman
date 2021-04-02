use crate::pipeline::{ScrapePipelineStage, ScrapeResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct ClickElement;

impl fmt::Display for ClickElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClickElement")
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapePipelineStage for ClickElement {
    async fn execute(
        &self,
        _client: &mut fantoccini::Client,
        context: &mut crate::ScrapePipelineContext,
    ) -> Result<(), crate::ScrapeResult> {
        match context.current_element.take() {
            Some(element) => element
                .click()
                .await
                .map_err(ScrapeResult::WebdriverCommandError)
                .map(|_| ()),

            None => Err(ScrapeResult::MissingElement),
        }
    }
}
