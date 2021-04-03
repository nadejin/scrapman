use crate::{
    action::ScrapeAction,
    pipeline::{ScrapeContext, ScrapeError, ScrapeResult},
    value::Value,
};
use async_trait::async_trait;
use fantoccini::Client;
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
impl ScrapeAction for OpenUrl {
    async fn execute(&self, client: &mut Client, mut context: &mut ScrapeContext) -> ScrapeResult {
        let url = self.url.resolve(&mut context).await?;
        client.goto(&url).await.map_err(ScrapeError::WebdriverCommandError)
    }
}
