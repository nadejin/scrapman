use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::{ScrapeContext, ScrapeError, ScrapePipeline},
    value::Value,
};
use async_trait::async_trait;
use fantoccini::{elements::Element, Locator};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ElementScope {
    Global,
    Scoped,
    Current,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryElement {
    selector: Selector,
    query: Value,
    scope: ElementScope,
    for_each: Option<ScrapePipeline>,
}

impl QueryElement {
    pub fn new(selector: Selector, query: Value, scope: ElementScope) -> Self {
        QueryElement {
            selector,
            query,
            scope,
            for_each: None,
        }
    }

    pub fn global(selector: Selector, query: Value) -> Self {
        QueryElement::new(selector, query, ElementScope::Global)
    }

    pub fn scoped(selector: Selector, query: Value) -> Self {
        QueryElement::new(selector, query, ElementScope::Scoped)
    }

    pub fn current(selector: Selector, query: Value) -> Self {
        QueryElement::new(selector, query, ElementScope::Current)
    }

    pub fn for_each(mut self, pipeline: ScrapePipeline) -> Self {
        self.for_each = Some(pipeline);
        self
    }
}

impl Display for QueryElement {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(
            fmt,
            "QueryElement({:?}, {}, {:?})",
            self.selector, self.query, self.scope
        )
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for QueryElement {
    async fn execute(&self, mut context: &mut ScrapeContext) -> ScrapeActionResult {
        let query = self
            .query
            .resolve(&mut context)
            .await?
            .ok_or(ScrapeError::MissingQuery)?;

        let locator = self.selector.get_locator(&query);
        let mut elements = match self.scope {
            ElementScope::Global => context.client.find_all(locator).await?,
            ElementScope::Scoped => find_child_elements(&mut context.scoped_element, locator).await?,
            ElementScope::Current => find_child_elements(&mut context.current_element, locator).await?,
        };

        if elements.is_empty() {
            return Err(ScrapeError::ElementQueryEmptyResult);
        }

        if let Some(ref pipeline) = self.for_each {
            // Store current scoped element
            let current_scoped = context.scoped_element.take();

            let count = elements.len();
            for (idx, element) in elements.into_iter().enumerate() {
                context.scoped_element = Some(element.clone());
                context.current_element = Some(element.clone());

                // Nested pipeline execution launch
                let _res = pipeline.execute(&mut context).await;
                // TODO: process errors in nested pipeline
                if let Err(error) = _res {
                    println!("!! Nested pipeline error: {}", error);
                }

                // Current element set to the last element in the sequence
                if idx == count - 1 {
                    context.current_element = Some(element);
                }
            }

            // Restore original scoped element
            context.scoped_element = current_scoped;
        } else {
            context.current_element = elements.pop();
        }

        Ok(())
    }
}

async fn find_child_elements<'a>(
    element: &mut Option<Element>,
    locator: Locator<'a>,
) -> Result<Vec<Element>, ScrapeError> {
    if let Some(element) = element {
        element
            .find_all(locator)
            .await
            .map_err(ScrapeError::WebdriverCommandError)
    } else {
        Err(ScrapeError::MissingElement)
    }
}
