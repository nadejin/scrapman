use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::{ScrapeContext, ScrapeError},
    value::Value,
};
use async_trait::async_trait;
use json_dotpath::DotPaths;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct SetModelAttribute {
    pub attribute: String,
    pub value: Value,
}

impl SetModelAttribute {
    pub fn new<T: Into<String>>(attribute: T, value: Value) -> Self {
        SetModelAttribute {
            attribute: attribute.into(),
            value,
        }
    }
}

impl Display for SetModelAttribute {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "SetModelAttribute({}, {})", self.attribute, self.value)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for SetModelAttribute {
    async fn execute(&self, mut context: &mut ScrapeContext) -> ScrapeActionResult {
        let value = self.value.resolve(&mut context).await?;
        context
            .model
            .dot_set(&self.attribute, value)
            .map_err(|_| ScrapeError::SetModelAttributeError(self.attribute.clone()))
    }
}
