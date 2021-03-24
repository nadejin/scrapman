mod pipeline;
mod value;

pub use pipeline::{ElementSearchScope, Pipeline, PipelineBuilder, PipelineExecutionError, PipelineStage, Selector};
pub use value::{JsonValue, Value};

use fantoccini::{elements::Element, Client, ClientBuilder};
use serde_json::json;
use std::mem::swap;

pub struct Scrapman {
    webdriver_url: String,
}

pub struct ScrapeContext {
    pub element: Option<Element>,
    pub model: JsonValue,
    pub models: Vec<JsonValue>,
}

impl Default for ScrapeContext {
    fn default() -> Self {
        ScrapeContext {
            element: None,
            model: json!({}),
            models: Vec::new(),
        }
    }
}

impl Scrapman {
    pub fn new<T: Into<String>>(webdriver_url: T) -> Self {
        Scrapman {
            webdriver_url: webdriver_url.into(),
        }
    }

    pub async fn execute(&self, pipeline: Pipeline) -> Result<(), PipelineExecutionError> {
        let mut client = ClientBuilder::native()
            .connect(&self.webdriver_url)
            .await
            .map_err(PipelineExecutionError::WebdriverConnectionError)?;

        let mut context = ScrapeContext::default();

        let mut result = Ok(());
        for stage in pipeline.into_iter() {
            result = self.execute_stage(stage, &mut client, &mut context).await;
            if result.is_err() {
                break;
            }
        }

        client
            .close_window()
            .await
            .map_err(PipelineExecutionError::WebdriverCommandError)?;

        client
            .close()
            .await
            .map_err(PipelineExecutionError::WebdriverCommandError)?;

        result
    }

    async fn execute_stage(
        &self,
        stage: PipelineStage,
        client: &mut Client,
        context: &mut ScrapeContext,
    ) -> Result<(), PipelineExecutionError> {
        match stage {
            PipelineStage::OpenUrl { url } => match url.resolve(&context) {
                Some(url) => client
                    .goto(&url)
                    .await
                    .map_err(PipelineExecutionError::WebdriverCommandError),

                None => Err(PipelineExecutionError::ValueResolveError),
            },

            PipelineStage::FindElement { selector, scope } => match selector.get_locator(&context) {
                Some(locator) => match scope {
                    ElementSearchScope::Global => client
                        .find(locator)
                        .await
                        .map_err(PipelineExecutionError::WebdriverCommandError)
                        .and_then(|element| Ok(context.element = Some(element))),

                    _ => Err(PipelineExecutionError::MissingElementLocator), // TODO support all locators
                },

                None => Err(PipelineExecutionError::MissingElementLocator),
            },

            PipelineStage::FillElement { value } => {
                let value = value
                    .resolve(&context)
                    .ok_or(PipelineExecutionError::ValueResolveError)?
                    .to_owned();

                if let Some(ref mut element) = context.element {
                    element
                        .send_keys(&value)
                        .await
                        .map_err(PipelineExecutionError::WebdriverCommandError)
                } else {
                    Err(PipelineExecutionError::MissingCurrentElement)
                }
            }

            PipelineStage::ClickElement => match context.element.take() {
                Some(element) => element
                    .click()
                    .await
                    .map_err(PipelineExecutionError::WebdriverCommandError)
                    .map(|_| ()),

                None => Err(PipelineExecutionError::MissingCurrentElement),
            },

            PipelineStage::StoreModel => {
                let mut model = json!({});
                swap(&mut model, &mut context.model);
                context.models.push(model);
                Ok(())
            }

            _ => Err(PipelineExecutionError::MissingStageExecutor(stage)),
        }
    }
}
