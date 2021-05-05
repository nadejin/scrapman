use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::ScrapeContext,
    value::Value,
    ScrapeError,
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
        write!(fmt, "open URL with the value from {}", self.url)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for OpenUrl {
    async fn execute(&self, mut context: &mut ScrapeContext) -> ScrapeActionResult {
        match self.url.resolve(&mut context).await? {
            Some(url) => context.client.goto(&url).await,
            None => Err(ScrapeError::MissingUrl),
        }
    }
}
