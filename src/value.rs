use crate::pipeline::{ScrapePipelineContext, ScrapeResult};
use json_dotpath::DotPaths;
use serde::{Deserialize, Serialize};
use std::fmt;

pub type JsonValue = serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Constant(String),
    Context(String),
    ElementText,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Value {
    pub async fn resolve(&self, context: &mut ScrapePipelineContext) -> Result<String, ScrapeResult> {
        match self {
            Value::Constant(value) => Ok(value.clone()),

            Value::Context(key) => context
                .values
                .dot_get::<JsonValue>(&key)
                .map_err(|_| ScrapeResult::ValueResolveError)
                .and_then(to_string),

            Value::ElementText => match context.current_element {
                Some(ref mut element) => element.text().await.map_err(ScrapeResult::WebdriverCommandError),
                None => Err(ScrapeResult::MissingElement),
            },
        }
    }

    pub fn constant<T: Into<String>>(value: T) -> Self {
        Value::Constant(value.into())
    }

    pub fn context<T: Into<String>>(value: T) -> Self {
        Value::Context(value.into())
    }
}

fn to_string(value: Option<JsonValue>) -> Result<String, ScrapeResult> {
    match value {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(value) => Ok(value.to_string()),
        _ => Err(ScrapeResult::ValueResolveError),
    }
}
