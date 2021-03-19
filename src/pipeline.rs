use crate::{value::Value, ScrapeContext};
use fantoccini::Locator;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Selector {
    Css(Value),
    Id(Value),
    LinkText(Value),
}

impl Selector {
    pub fn get_locator<'a>(&'a self, context: &'a ScrapeContext) -> Option<Locator<'a>> {
        match self {
            Selector::Css(value) => value.resolve(context).map(Locator::Css),
            Selector::Id(value) => value.resolve(context).map(Locator::Id),
            Selector::LinkText(value) => value.resolve(context).map(Locator::LinkText),
        }
    }
}

pub type Pipeline = Vec<PipelineStage>;

#[derive(Serialize, Deserialize, Debug)]
pub enum PipelineStage {
    OpenUrl { url: Value },
    FindElement { selector: Selector },
    FindElements { selector: Selector, execute: Pipeline },
    FillElement { value: Value },
    ClickElement,
    StoreModel,
    SetModelAttribute { attribute: String, value: Value },
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

    pub fn find_element(mut self, selector: Selector) -> Self {
        self.pipeline.push(PipelineStage::FindElement { selector });
        self
    }

    pub fn find_elements(mut self, selector: Selector, execute: Pipeline) -> Self {
        self.pipeline.push(PipelineStage::FindElements { selector, execute });

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
