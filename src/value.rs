use crate::pipeline::{PipelineExecutionContext, PipelineExecutionError};
use fantoccini::elements::Element;
use json_dotpath::DotPaths;
use serde::{Deserialize, Serialize};

pub type JsonValue = serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Value {
    Constant(String),
    Context(String),
    CurrentElementText,
    ScopedElementText,
}

impl Value {
    pub async fn resolve(&self, context: &mut PipelineExecutionContext) -> Result<String, PipelineExecutionError> {
        match self {
            Value::Constant(value) => Ok(value.clone()),

            Value::Context(key) => context
                .values
                .dot_get::<JsonValue>(&key)
                .map_err(|_| PipelineExecutionError::ValueResolveError)
                .and_then(to_string),

            Value::CurrentElementText => get_text(&mut context.current_element).await,

            Value::ScopedElementText => get_text(&mut context.scoped_element).await,
        }
    }

    pub fn constant<T: Into<String>>(value: T) -> Self {
        Value::Constant(value.into())
    }

    pub fn context<T: Into<String>>(value: T) -> Self {
        Value::Context(value.into())
    }
}

fn to_string(value: Option<serde_json::Value>) -> Result<String, PipelineExecutionError> {
    match value {
        Some(serde_json::Value::String(value)) => Ok(value.clone()),
        Some(value) => Ok(value.to_string()),
        _ => Err(PipelineExecutionError::ValueResolveError),
    }
}

async fn get_text(element: &mut Option<Element>) -> Result<String, PipelineExecutionError> {
    if let Some(ref mut element) = element {
        element
            .text()
            .await
            .map_err(PipelineExecutionError::WebdriverCommandError)
    } else {
        Err(PipelineExecutionError::MissingContextElement)
    }
}
