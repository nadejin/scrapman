use crate::pipeline::{ScrapeContext, ScrapeError};
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
    pub async fn resolve(&self, context: &mut ScrapeContext) -> Result<String, ScrapeError> {
        match self {
            Value::Constant(value) => Ok(value.clone()),

            Value::Context(key) => context
                .values
                .dot_get::<JsonValue>(&key)
                .map_err(|_| ScrapeError::ValueResolveError)
                .and_then(to_string),

            Value::ElementText => match context.current_element {
                Some(ref mut element) => element.text().await.map_err(ScrapeError::WebdriverCommandError),
                None => Err(ScrapeError::MissingElement),
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

fn to_string(value: Option<JsonValue>) -> Result<String, ScrapeError> {
    match value {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(value) => Ok(value.to_string()),
        _ => Err(ScrapeError::ValueResolveError),
    }
}
