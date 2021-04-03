use crate::{
    action::ScrapeAction,
    pipeline::{ScrapeContext, ScrapeError, ScrapeResult},
};
use async_trait::async_trait;
use fantoccini::Client;
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
impl ScrapeAction for ClickElement {
    async fn execute(&self, _: &mut Client, context: &mut ScrapeContext) -> ScrapeResult {
        match context.current_element.take() {
            Some(element) => element
                .click()
                .await
                .map_err(ScrapeError::WebdriverCommandError)
                .map(|_| ()),

            None => Err(ScrapeError::MissingElement),
        }
    }
}
