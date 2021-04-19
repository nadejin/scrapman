use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::{ScrapeContext, ScrapeError},
    value::Value,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct FillElement {
    pub value: Value,
}

impl FillElement {
    pub fn new(value: Value) -> FillElement {
        FillElement { value }
    }
}

impl Display for FillElement {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "FillElement({})", self.value)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for FillElement {
    async fn execute(&self, mut context: &mut ScrapeContext) -> ScrapeActionResult {
        let value = self.value.resolve(&mut context).await?;
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
