use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::{ScrapeContext, ScrapeError},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClickElement;

impl Display for ClickElement {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "ClickElement")
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for ClickElement {
    async fn execute(&self, context: &mut ScrapeContext) -> ScrapeActionResult {
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