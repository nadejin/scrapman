use crate::value::JsonValue;
use async_trait::async_trait;
use fantoccini::{
    elements::Element,
    error::{CmdError, NewSessionError},
    Client,
};
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{error::Error, fmt};

#[async_trait]
#[typetag::serde(tag = "stage")]
pub trait ScrapePipelineStage: fmt::Display + Send + Sync {
    async fn execute(&self, client: &mut Client, context: &mut ScrapePipelineContext) -> Result<(), ScrapeResult>;
}

#[derive(Default, Serialize, Deserialize)]
pub struct ScrapePipeline {
    stages: Vec<Box<dyn ScrapePipelineStage>>,
}

impl ScrapePipeline {
    pub fn push<T: 'static + ScrapePipelineStage>(mut self, stage: T) -> Self {
        self.stages.push(Box::new(stage));
        self
    }

    pub fn execute<'a>(
        &'a self,
        mut client: &'a mut Client,
        mut context: &'a mut ScrapePipelineContext,
    ) -> BoxFuture<'a, Result<(), ScrapeResult>> {
        async move {
            let mut result = Ok(());
            for stage in self.stages.iter() {
                result = stage.execute(&mut client, &mut context).await;

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

pub struct ScrapePipelineContext {
    pub model: JsonValue,
    pub values: JsonValue,
    pub models: Vec<JsonValue>,
    pub scoped_element: Option<Element>,
    pub current_element: Option<Element>,
}

impl ScrapePipelineContext {
    pub fn with_values(values: JsonValue) -> Self {
        let mut context = ScrapePipelineContext::default();
        context.values = values;
        context
    }
}

impl Default for ScrapePipelineContext {
    fn default() -> Self {
        ScrapePipelineContext {
            current_element: None,
            scoped_element: None,
            model: json!({}),
            values: json!({}),
            models: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum ScrapeResult {
    ValueResolveError,
    MissingElement,
    SetModelAttributeError(String),
    WebdriverConnectionError(NewSessionError),
    WebdriverCommandError(CmdError),
}

impl fmt::Display for ScrapeResult {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScrapeResult::ValueResolveError => {
                write!(fmt, "failed to resolve value")
            }

            ScrapeResult::MissingElement => {
                write!(fmt, "required element is missing in the pipeline execution context")
            }

            ScrapeResult::SetModelAttributeError(attribute) => {
                write!(fmt, "failed to populate model attribute {}", attribute)
            }

            ScrapeResult::WebdriverConnectionError(error) => {
                write!(fmt, "webdriver connection error: {}", error)
            }

            ScrapeResult::WebdriverCommandError(error) => {
                write!(fmt, "webdriver command error: {}", error)
            }
        }
    }
}

impl Error for ScrapeResult {}
