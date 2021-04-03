use crate::{stage::ScrapeStage, value::JsonValue};
use fantoccini::{
    elements::Element,
    error::{CmdError, NewSessionError},
    Client,
};
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{error::Error, fmt};

#[derive(Default, Serialize, Deserialize)]
pub struct ScrapePipeline {
    stages: Vec<ScrapeStage>,
}

impl ScrapePipeline {
    pub fn push<T: Into<ScrapeStage>>(mut self, stage: T) -> Self {
        self.stages.push(stage.into());
        self
    }

    pub fn execute<'a>(
        &'a self,
        mut client: &'a mut Client,
        mut context: &'a mut ScrapeContext,
    ) -> BoxFuture<'a, Result<(), ScrapeError>> {
        async move {
            let mut result = Ok(());
            for stage in self.stages.iter() {
                result = stage.action.execute(&mut client, &mut context).await;

                // TODO: per-stage configuration for error handling
                if result.is_err() {
                    break;
                }
            }

            result
        }
        .boxed()
    }
}

pub struct ScrapeContext {
    pub model: JsonValue,
    pub values: JsonValue,
    pub models: Vec<JsonValue>,
    pub scoped_element: Option<Element>,
    pub current_element: Option<Element>,
}

impl ScrapeContext {
    pub fn with_values(values: JsonValue) -> Self {
        let mut context = ScrapeContext::default();
        context.values = values;
        context
    }
}

impl Default for ScrapeContext {
    fn default() -> Self {
        ScrapeContext {
            current_element: None,
            scoped_element: None,
            model: json!({}),
            values: json!({}),
            models: Vec::new(),
        }
    }
}

pub type ScrapeResult = Result<(), ScrapeError>;

#[derive(Debug)]
pub enum ScrapeError {
    ValueResolveError,
    MissingElement,
    SetModelAttributeError(String),
    WebdriverConnectionError(NewSessionError),
    WebdriverCommandError(CmdError),
}

impl fmt::Display for ScrapeError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScrapeError::ValueResolveError => {
                write!(fmt, "failed to resolve value")
            }

            ScrapeError::MissingElement => {
                write!(fmt, "required element is missing in the pipeline execution context")
            }

            ScrapeError::SetModelAttributeError(attribute) => {
                write!(fmt, "failed to populate model attribute {}", attribute)
            }

            ScrapeError::WebdriverConnectionError(error) => {
                write!(fmt, "webdriver connection error: {}", error)
            }

            ScrapeError::WebdriverCommandError(error) => {
                write!(fmt, "webdriver command error: {}", error)
            }
        }
    }
}

impl Error for ScrapeError {}
