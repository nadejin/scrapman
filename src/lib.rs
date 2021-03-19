mod pipeline;
mod value;

pub use pipeline::{Pipeline, PipelineBuilder, PipelineStage, Selector};
pub use value::{JsonValue, Value};

use fantoccini::{elements::Element, Client, ClientBuilder};
use serde_json::json;
use std::{error::Error, mem::swap};

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

    pub async fn execute(&self, pipeline: Pipeline) -> Result<(), Box<dyn Error>> {
        let mut client = ClientBuilder::native().connect(&self.webdriver_url).await?;
        let mut context = ScrapeContext::default();
        let mut result = Ok(());

        for stage in pipeline.into_iter() {
            result = self.execute_stage(stage, &mut client, &mut context).await;
            if result.is_err() {
                break;
            }
        }

        // std::thread::sleep(std::time::Duration::from_secs(5));

        client.close_window().await?;
        client.close().await?;
        result
    }

    async fn execute_stage(
        &self,
        stage: PipelineStage,
        client: &mut Client,
        context: &mut ScrapeContext,
    ) -> Result<(), Box<dyn Error>> {
        match stage {
            PipelineStage::OpenUrl { url } => match url.resolve(&context) {
                Some(url) => client.goto(&url).await.map_err(Into::into),
                None => Err("Missing url to open".into()),
            },

            PipelineStage::FindElement { selector } => match selector.get_locator(&context) {
                Some(locator) => client
                    .find(locator)
                    .await
                    .map_err(Into::into)
                    .and_then(|element| Ok(context.element = Some(element))),

                None => Err("Missing element selector".into()),
            },

            PipelineStage::FillElement { value } => {
                let value = value
                    .resolve(&context)
                    .ok_or("Missing value to fill the element")?
                    .to_owned();

                if let Some(ref mut element) = context.element {
                    element.send_keys(&value).await.map_err(Into::into)
                } else {
                    Err("Missing active element to fill in the current context".into())
                }
            }

            PipelineStage::ClickElement => match context.element.take() {
                Some(element) => element.click().await.map_err(Into::into).map(|_| ()),
                None => Err("".into()),
            },

            PipelineStage::StoreModel => {
                let mut model = json!({});
                swap(&mut model, &mut context.model);
                context.models.push(model);
                Ok(())
            }

            _ => Err(format!("Missing executor for pipeline stage {:?}", stage).into()),
        }
    }
}
