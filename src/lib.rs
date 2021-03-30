mod pipeline;
mod value;

pub use pipeline::{
    ElementSearchScope, Pipeline, PipelineBuilder, PipelineExecutionContext, PipelineExecutionError, PipelineStage,
    Selector,
};
pub use value::{JsonValue, Value};

use fantoccini::{elements::Element, Client, ClientBuilder, Locator};
use futures::future::{BoxFuture, FutureExt};
use json_dotpath::DotPaths;
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
    ) -> Result<Vec<JsonValue>, PipelineExecutionError> {
        let mut client = ClientBuilder::native()
            .connect(&self.webdriver_url)
            .await
            .map_err(PipelineExecutionError::WebdriverConnectionError)?;

        let mut context = match values.into() {
            Some(values) => PipelineExecutionContext::with_values(values),
            None => PipelineExecutionContext::default(),
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

        result.map(|_| context.models)
    }

    fn execute_pipeline<'a>(
        &'a self,
        pipeline: Pipeline,
        mut client: &'a mut Client,
        mut context: &'a mut PipelineExecutionContext,
    ) -> BoxFuture<'a, Result<(), PipelineExecutionError>> {
        async move {
            let mut result = Ok(());
            for stage in pipeline.into_iter() {
                result = self.execute_pipeline_stage(stage, &mut client, &mut context).await;
                if result.is_err() {
                    break;
                }
            }

            result
        }
        .boxed()
    }

    async fn execute_pipeline_stage(
        &self,
        stage: PipelineStage,
        mut client: &mut Client,
        mut context: &mut PipelineExecutionContext,
    ) -> Result<(), PipelineExecutionError> {
        match stage {
            PipelineStage::OpenUrl { url } => {
                let url = url.resolve(&mut context).await?;
                client
                    .goto(&url)
                    .await
                    .map_err(PipelineExecutionError::WebdriverCommandError)
            }

            PipelineStage::QueryElement {
                selector,
                query,
                scope,
                execute,
            } => {
                let query = query.resolve(&mut context).await?;
                let locator = selector.get_locator(&query);
                let mut elements = match scope {
                    ElementSearchScope::Global => find_elements(&mut client, locator).await?,
                    ElementSearchScope::Scoped => find_child_elements(&mut context.scoped_element, locator).await?,
                    ElementSearchScope::Current => find_child_elements(&mut context.current_element, locator).await?,
                };

                if let Some(pipeline) = execute {
                    // Store current scoped element
                    let current_scoped = context.scoped_element.take();

                    let count = elements.len();
                    for (idx, element) in elements.into_iter().enumerate() {
                        context.scoped_element = Some(element.clone());
                        context.current_element = Some(element.clone());

                        // Nested pipeline execution launch
                        let _res = self.execute_pipeline(pipeline.clone(), client, context).await;
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

            PipelineStage::FillElement { value } => {
                let value = value.resolve(&mut context).await?;
                if let Some(ref mut element) = context.current_element {
                    element
                        .send_keys(&value)
                        .await
                        .map_err(PipelineExecutionError::WebdriverCommandError)
                } else {
                    Err(PipelineExecutionError::MissingContextElement)
                }
            }

            PipelineStage::ClickElement => match context.current_element.take() {
                Some(element) => element
                    .click()
                    .await
                    .map_err(PipelineExecutionError::WebdriverCommandError)
                    .map(|_| ()),

                None => Err(PipelineExecutionError::MissingContextElement),
            },

            PipelineStage::StoreModel => {
                let mut model = json!({});
                swap(&mut model, &mut context.model);
                context.models.push(model);
                Ok(())
            }

            PipelineStage::SetModelAttribute { attribute, value } => {
                let value = value.resolve(&mut context).await?;
                context
                    .model
                    .dot_set(&attribute, value)
                    .map_err(|_| PipelineExecutionError::SetModelAttributeError(attribute))
            }
        }
    }
}

async fn find_elements<'a>(client: &mut Client, locator: Locator<'a>) -> Result<Vec<Element>, PipelineExecutionError> {
    client
        .find_all(locator)
        .await
        .map_err(PipelineExecutionError::WebdriverCommandError)
}

async fn find_child_elements<'a>(
    element: &mut Option<Element>,
    locator: Locator<'a>,
) -> Result<Vec<Element>, PipelineExecutionError> {
    if let Some(element) = element {
        element
            .find_all(locator)
            .await
            .map_err(PipelineExecutionError::WebdriverCommandError)
    } else {
        Err(PipelineExecutionError::MissingContextElement)
    }
}
