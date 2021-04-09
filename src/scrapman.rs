use crate::pipeline::{ScrapeContext, ScrapeError, ScrapePipeline};
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
    ) -> Result<Vec<JsonValue>, ScrapeError> {
        let client = ClientBuilder::native()
            .connect(&self.webdriver_url)
            .await
            .map_err(ScrapeError::WebdriverConnectionError)?;

        let mut context = ScrapeContext::new(client, values);
        pipeline.execute(&mut context).await?;
        context.client.disconnect().await?;
        Ok(context.models)
    }
}
