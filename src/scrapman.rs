use crate::{
    client::ScrapeClient,
    pipeline::{ScrapeContext, ScrapeError, ScrapePipeline},
    value::JsonValue,
};
use fantoccini::ClientBuilder;
use log::info;

pub type ScrapeResult = Result<ScrapeContext, ScrapeError>;

pub struct Scrapman {
    webdriver_url: String,
}

impl Scrapman {
    pub fn new<T: Into<String>>(webdriver_url: T) -> Self {
        Scrapman {
            webdriver_url: webdriver_url.into(),
        }
    }

    pub async fn launch<T>(&self, pipeline: ScrapePipeline, values: T) -> ScrapeResult
    where
        T: Into<Option<JsonValue>>,
    {
        let client = ClientBuilder::native()
            .connect(&self.webdriver_url)
            .await
            .map_err(ScrapeError::WebdriverConnectionError)?;

        self.launch_with_client(pipeline, values, client).await
    }

    pub async fn launch_with_client<Values, Client>(
        &self,
        pipeline: ScrapePipeline,
        values: Values,
        client: Client,
    ) -> ScrapeResult
    where
        Values: Into<Option<JsonValue>>,
        Client: ScrapeClient + 'static,
    {
        let mut context = ScrapeContext::new(client, values);
        info!("Launching pipeline execution");
        pipeline.execute(&mut context).await?;
        context.client.disconnect().await?;
        Ok(context)
    }
}
