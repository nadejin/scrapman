use crate::pipeline::{ScrapePipeline, ScrapePipelineContext, ScrapeResult};
use crate::value::JsonValue;
use fantoccini::ClientBuilder;

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
        pipeline: ScrapePipeline,
        values: T,
    ) -> Result<Vec<JsonValue>, ScrapeResult> {
        let mut client = ClientBuilder::native()
            .connect(&self.webdriver_url)
            .await
            .map_err(ScrapeResult::WebdriverConnectionError)?;

        let mut context = match values.into() {
            Some(values) => ScrapePipelineContext::with_values(values),
            None => ScrapePipelineContext::default(),
        };

        let result = pipeline.execute(&mut client, &mut context).await;

        client
            .close_window()
            .await
            .map_err(ScrapeResult::WebdriverCommandError)?;

        client.close().await.map_err(ScrapeResult::WebdriverCommandError)?;

        result.map(|_| context.models)
    }
}
