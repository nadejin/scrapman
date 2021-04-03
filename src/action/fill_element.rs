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
pub struct FillElement(pub Value);

impl fmt::Display for FillElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FillElement({})", self.0)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for FillElement {
    async fn execute(&self, _: &mut Client, mut context: &mut ScrapeContext) -> ScrapeResult {
        let value = self.0.resolve(&mut context).await?;
        if let Some(ref mut element) = context.current_element {
            element
                .send_keys(&value)
                .await
                .map_err(ScrapeError::WebdriverCommandError)
        } else {
            Err(ScrapeError::MissingElement)
        }
    }
}
