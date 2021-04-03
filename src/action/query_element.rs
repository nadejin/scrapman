use crate::{
    action::ScrapeAction,
    pipeline::{ScrapeContext, ScrapeError, ScrapePipeline, ScrapeResult},
    value::Value,
};
use async_trait::async_trait;
use fantoccini::{elements::Element, Client, Locator};
use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Serialize, Deserialize)]
pub struct QueryElement {
    selector: Selector,
    query: Value,
    scope: ElementScope,
    for_each: Option<ScrapePipeline>,
}

impl QueryElement {
    pub fn new(selector: Selector, query: Value) -> Self {
        QueryElement {
            selector,
            query,
            scope: ElementScope::Global,
            for_each: None,
        }
    }

    pub fn scoped(selector: Selector, query: Value) -> Self {
        QueryElement::new(selector, query).with_scope(ElementScope::Scoped)
    }

    pub fn current(selector: Selector, query: Value) -> Self {
        QueryElement::new(selector, query).with_scope(ElementScope::Current)
    }

    pub fn with_scope(mut self, scope: ElementScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn for_each(mut self, pipeline: ScrapePipeline) -> Self {
        self.for_each = Some(pipeline);
        self
    }
}

impl fmt::Display for QueryElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QueryElement({:?}, {}, {:?})", self.selector, self.query, self.scope)
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for QueryElement {
    async fn execute(&self, mut client: &mut Client, mut context: &mut ScrapeContext) -> ScrapeResult {
        let query = self.query.resolve(&mut context).await?;
        let locator = self.selector.get_locator(&query);
        let mut elements = match self.scope {
            ElementScope::Global => find_elements(&mut client, locator).await?,
            ElementScope::Scoped => find_child_elements(&mut context.scoped_element, locator).await?,
            ElementScope::Current => find_child_elements(&mut context.current_element, locator).await?,
        };

        if let Some(ref pipeline) = self.for_each {
            // Store current scoped element
            let current_scoped = context.scoped_element.take();

            let count = elements.len();
            for (idx, element) in elements.into_iter().enumerate() {
                context.scoped_element = Some(element.clone());
                context.current_element = Some(element.clone());

                // Nested pipeline execution launch
                let _res = pipeline.execute(client, context).await;
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

async fn find_elements<'a>(client: &mut Client, locator: Locator<'a>) -> Result<Vec<Element>, ScrapeError> {
    client
        .find_all(locator)
        .await
        .map_err(ScrapeError::WebdriverCommandError)
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
