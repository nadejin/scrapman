mod context;
mod pipeline;
mod value;

pub use context::ScrapeContext;
pub use pipeline::{ElementSearchScope, Pipeline, PipelineBuilder, PipelineExecutionError, PipelineStage, Selector};
pub use value::{JsonValue, Value};

use fantoccini::{Client, ClientBuilder};
use serde_json::json;
use std::mem::swap;

pub struct Scrapman {
    webdriver_url: String,
}

impl Scrapman {
    pub fn new<T: Into<String>>(webdriver_url: T) -> Self {
        Scrapman {
            webdriver_url: webdriver_url.into(),
        }
    }

    pub async fn launch<T: Into<Option<JsonValue>>>(
        &self,
        pipeline: Pipeline,
        values: T,
    ) -> Result<(), PipelineExecutionError> {
        let mut client = ClientBuilder::native()
            .connect(&self.webdriver_url)
            .await
            .map_err(PipelineExecutionError::WebdriverConnectionError)?;

        let mut context = match values.into() {
            Some(values) => ScrapeContext::with_values(values),
            None => ScrapeContext::default(),
        };

        let result = self.execute_pipeline(pipeline, &mut client, &mut context).await;

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

    async fn execute_pipeline(
        &self,
        pipeline: Pipeline,
        mut client: &mut Client,
        mut context: &mut ScrapeContext,
    ) -> Result<(), PipelineExecutionError> {
        let mut result = Ok(());
        for stage in pipeline.into_iter() {
            result = self.execute_pipeline_stage(stage, &mut client, &mut context).await;
            if result.is_err() {
                break;
            }
        }

        result
    }

    async fn execute_pipeline_stage(
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

            PipelineStage::FindElement { selector, query, scope } => {
                let query = query
                    .resolve(&context)
                    .ok_or(PipelineExecutionError::MissingElementLocator)?;

                match scope {
                    ElementSearchScope::Global => client
                        .find(selector.get_locator(&query))
                        .await
                        .map_err(PipelineExecutionError::WebdriverCommandError)
                        .and_then(|element| Ok(context.current_element = Some(element))),

                    _ => Err(PipelineExecutionError::MissingElementLocator), // TODO support all locators
                }
            }

            PipelineStage::FillElement { value } => {
                let value = value
                    .resolve(&context)
                    .ok_or(PipelineExecutionError::ValueResolveError)?;

                if let Some(ref mut element) = context.current_element {
                    element
                        .send_keys(&value)
                        .await
                        .map_err(PipelineExecutionError::WebdriverCommandError)
                } else {
                    Err(PipelineExecutionError::MissingCurrentElement)
                }
            }

            PipelineStage::ClickElement => match context.current_element.take() {
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
