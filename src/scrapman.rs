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
        let mut client = ClientBuilder::native()
            .connect(&self.webdriver_url)
            .await
            .map_err(ScrapeError::WebdriverConnectionError)?;

        let mut context = match values.into() {
            Some(values) => ScrapeContext::with_values(values),
            None => ScrapeContext::default(),
        };

        let result = pipeline.execute(&mut client, &mut context).await;

        client
            .close_window()
            .await
            .map_err(ScrapeError::WebdriverCommandError)?;

        client.close().await.map_err(ScrapeError::WebdriverCommandError)?;

        result.map(|_| context.models)
    }
}
