use crate::pipeline::{ScrapeContext, ScrapeError};
use json_dotpath::DotPaths;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};

pub type JsonValue = serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Constant(String),
    Context(String),
    ElementText,
    ElementAttribute(String),
}

impl Display for Value {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "{:?}", self)
    }
}

impl Value {
    pub async fn resolve(&self, context: &mut ScrapeContext) -> Result<Option<String>, ScrapeError> {
        match self {
            Value::Constant(value) => Ok(Some(value.to_owned())),

            Value::Context(key) => context
                .values
                .dot_get::<JsonValue>(&key)
                .map(to_string)
                .map_err(|_| ScrapeError::ValueResolveError),

            Value::ElementText => {
                let element = context.current_element.as_mut().ok_or(ScrapeError::MissingElement)?;
                element
                    .text()
                    .await
                    .map(Option::Some)
                    .map_err(ScrapeError::WebdriverCommandError)
            }

            Value::ElementAttribute(attribute) => {
                let element = context.current_element.as_mut().ok_or(ScrapeError::MissingElement)?;
                element
                    .attr(attribute)
                    .await
                    .map_err(ScrapeError::WebdriverCommandError)
            }
        }
    }
}

fn to_string(value: Option<JsonValue>) -> Option<String> {
    value.map(|value| match value {
        JsonValue::String(value) => value.clone(),
        value => value.to_string(),
    })
}
