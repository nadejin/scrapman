use crate::{
    action::ScrapeAction,
    pipeline::{ScrapeContext, ScrapeError, ScrapeResult},
    value::Value,
};
use async_trait::async_trait;
use fantoccini::Client;
use json_dotpath::DotPaths;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct SetModelAttribute {
    attribute: String,
    value: Value,
}

impl SetModelAttribute {
    pub fn new<T: Into<String>>(attribute: T, value: Value) -> Self {
        SetModelAttribute {
            attribute: attribute.into(),
            value,
        }
    }
}

impl fmt::Display for SetModelAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SetModelAttribute({}, {})", self.attribute, self.value)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for SetModelAttribute {
    async fn execute(&self, _: &mut Client, mut context: &mut ScrapeContext) -> ScrapeResult {
        let value = self.value.resolve(&mut context).await?;
        context
            .model
            .dot_set(&self.attribute, value)
            .map_err(|_| ScrapeError::SetModelAttributeError(self.attribute.clone()))
    }
}
