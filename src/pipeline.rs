use crate::value::Value;
use fantoccini::{
    error::{CmdError, NewSessionError},
    Locator,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Serialize, Deserialize, Debug)]
pub enum Selector {
    Css,
    Id,
    LinkText,
}

impl Selector {
    pub fn get_locator<'a>(&'a self, query: &'a str) -> Locator<'a> {
        match self {
            Selector::Css => Locator::Css(query),
            Selector::Id => Locator::Id(query),
            Selector::LinkText => Locator::LinkText(query),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ElementSearchScope {
    Global,
    Scoped,
    Current,
}

pub type Pipeline = Vec<PipelineStage>;

#[derive(Serialize, Deserialize, Debug)]
pub enum PipelineStage {
    OpenUrl {
        url: Value,
    },
    FindElement {
        selector: Selector,
        query: Value,
        scope: ElementSearchScope,
    },
    FindElements {
        selector: Selector,
        query: Value,
        scope: ElementSearchScope,
        execute: Pipeline,
    },
    FillElement {
        value: Value,
    },
    ClickElement,
    StoreModel,
    SetModelAttribute {
        attribute: String,
        value: Value,
    },
}

impl fmt::Display for PipelineStage {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PipelineStage::OpenUrl { .. } => write!(fmt, "OpenUrl"),
            PipelineStage::FindElement { .. } => write!(fmt, "FindElement"),
            PipelineStage::FindElements { .. } => write!(fmt, "FindElements"),
            PipelineStage::FillElement { .. } => write!(fmt, "FillElement"),
            PipelineStage::ClickElement => write!(fmt, "ClickElement"),
            PipelineStage::StoreModel => write!(fmt, "StoreModel"),
            PipelineStage::SetModelAttribute { attribute, .. } => write!(fmt, "SetModelAttribute({})", attribute),
        }
    }
}

pub struct PipelineBuilder {
    pipeline: Vec<PipelineStage>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        PipelineBuilder { pipeline: Vec::new() }
    }

    pub fn open_url(mut self, url: Value) -> Self {
        self.pipeline.push(PipelineStage::OpenUrl { url });
        self
    }

    pub fn find_element(mut self, selector: Selector, query: Value) -> Self {
        self.pipeline.push(PipelineStage::FindElement {
            selector,
            query,
            scope: ElementSearchScope::Global,
        });

        self
    }

    pub fn find_elements(mut self, selector: Selector, query: Value, execute: Pipeline) -> Self {
        self.pipeline.push(PipelineStage::FindElements {
            selector,
            query,
            execute,
            scope: ElementSearchScope::Global,
        });

        self
    }

    pub fn fill_element(mut self, value: Value) -> Self {
        self.pipeline.push(PipelineStage::FillElement { value });
        self
    }

    pub fn click_element(mut self) -> Self {
        self.pipeline.push(PipelineStage::ClickElement);
        self
    }

    pub fn store_model(mut self) -> Self {
        self.pipeline.push(PipelineStage::StoreModel);
        self
    }

    pub fn set_model_attribute<T: Into<String>>(mut self, attribute: T, value: Value) -> Self {
        self.pipeline.push(PipelineStage::SetModelAttribute {
            attribute: attribute.into(),
            value,
        });

        self
    }

    pub fn build(self) -> Pipeline {
        self.pipeline
    }
}

#[derive(Debug)]
pub enum PipelineExecutionError {
    ValueResolveError,
    MissingElementLocator,
    MissingCurrentElement,
    MissingStageExecutor(PipelineStage),
    WebdriverConnectionError(NewSessionError),
    WebdriverCommandError(CmdError),
}

impl fmt::Display for PipelineExecutionError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PipelineExecutionError::ValueResolveError => {
                write!(fmt, "failed to resolve value")
            }

            PipelineExecutionError::MissingElementLocator => {
                write!(fmt, "missing element locator")
            }

            PipelineExecutionError::MissingCurrentElement => {
                write!(fmt, "current element is missing in the pipeline execution context")
            }

            PipelineExecutionError::MissingStageExecutor(stage) => {
                write!(fmt, "missing pipeline executor for stage {}", stage)
            }

            PipelineExecutionError::WebdriverConnectionError(error) => {
                write!(fmt, "webdriver connection error: {}", error)
            }

            PipelineExecutionError::WebdriverCommandError(error) => {
                write!(fmt, "webdriver command error: {}", error)
            }
        }
    }
}

impl Error for PipelineExecutionError {}
