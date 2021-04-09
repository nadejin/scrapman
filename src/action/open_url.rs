use crate::{
    action::ScrapeAction,
    pipeline::{ScrapeContext, ScrapeResult},
    value::Value,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenUrl {
    pub url: Value,
}

impl OpenUrl {
    pub fn new(url: Value) -> OpenUrl {
        OpenUrl { url }
    }
}

impl Display for OpenUrl {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "OpenUrl({})", self.url)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for OpenUrl {
    async fn execute(&self, mut context: &mut ScrapeContext) -> ScrapeResult {
        let url = self.url.resolve(&mut context).await?;
        context.client.goto(&url).await
    }
}
