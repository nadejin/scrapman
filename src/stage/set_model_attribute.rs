use crate::{
    pipeline::{ScrapePipelineStage, ScrapeResult},
    value::Value,
};
use async_trait::async_trait;
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
impl ScrapePipelineStage for SetModelAttribute {
    async fn execute(
        &self,
        _client: &mut fantoccini::Client,
        mut context: &mut crate::ScrapePipelineContext,
    ) -> Result<(), crate::ScrapeResult> {
        let value = self.value.resolve(&mut context).await?;
        context
            .model
            .dot_set(&self.attribute, value)
            .map_err(|_| ScrapeResult::SetModelAttributeError(self.attribute.clone()))
    }
}
