use crate::{
    client::ScrapeClient,
    stage::{FlowControl, ScrapeStage},
    value::JsonValue,
};
use fantoccini::{
    elements::Element,
    error::{CmdError, NewSessionError},
};
use futures::future::{BoxFuture, FutureExt};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FormatResult},
};

lazy_static! {
    static ref EMPTY: JsonValue = json!({});
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScrapePipeline {
    stages: Vec<ScrapeStage>,
}

impl ScrapePipeline {
    pub fn push<T: Into<ScrapeStage>>(mut self, stage: T) -> Self {
        self.stages.push(stage.into());
        self
    }

    pub fn execute<'a>(&'a self, context: &'a mut ScrapeContext) -> BoxFuture<'a, ScrapeResult> {
        async move {
            let mut idx = 0;
            loop {
                match self.stages.get(idx) {
                    Some(stage) => {
                        let result = stage.action.execute(context).await;

                        // TODO log error

                        let flow = match result {
                            Ok(_) => &stage.on_complete,
                            Err(_) => &stage.on_error,
                        };

                        match flow {
                            FlowControl::Continue => idx += 1,
                            FlowControl::Quit => break,
                            FlowControl::Goto(next_stage) => {
                                match self.stages.iter().position(|stage| match &stage.name {
                                    Some(name) => name == next_stage,
                                    _ => false,
                                }) {
                                    Some(pos) => idx = pos,
                                    None => return Err(ScrapeError::MissingPipelineStage(next_stage.clone())),
                                }
                            }
                        };
                    }

                    None => break,
                }
            }

            Ok(())
        }
        .boxed()
    }
}

pub struct ScrapeContext {
    pub client: Box<dyn ScrapeClient>,
    pub model: JsonValue,
    pub values: JsonValue,
    pub models: Vec<JsonValue>,
    pub scoped_element: Option<Element>,
    pub current_element: Option<Element>,
}

impl ScrapeContext {
    pub fn new<C: ScrapeClient + 'static, V: Into<Option<JsonValue>>>(client: C, values: V) -> Self {
        ScrapeContext {
            client: Box::new(client),
            model: EMPTY.clone(),
            values: values.into().unwrap_or(EMPTY.clone()),
            models: Vec::new(),
            current_element: None,
            scoped_element: None,
        }
    }
}

pub type ScrapeResult = Result<(), ScrapeError>;

#[derive(Debug)]
pub enum ScrapeError {
    ValueResolveError,
    MissingElement,
    MissingPipelineStage(String),
    SetModelAttributeError(String),
    WebdriverConnectionError(NewSessionError),
    WebdriverCommandError(CmdError),
}

impl Display for ScrapeError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        match self {
            ScrapeError::ValueResolveError => {
                write!(fmt, "failed to resolve value")
            }

            ScrapeError::MissingElement => {
                write!(fmt, "required element is missing in the pipeline execution context")
            }

            ScrapeError::MissingPipelineStage(stage) => {
                write!(fmt, "missing pipeline stage {}", stage)
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
