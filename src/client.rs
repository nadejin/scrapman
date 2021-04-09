use crate::ScrapeError;
use async_trait::async_trait;
use fantoccini::{elements::Element, Client, Locator};

#[cfg(test)]
use mockall::automock;

#[async_trait]
#[cfg_attr(test, automock)]
pub trait ScrapeClient: Send + Sync {
    async fn goto(&mut self, url: &str) -> Result<(), ScrapeError>;
    async fn find_all(&mut self, search: Locator<'_>) -> Result<Vec<Element>, ScrapeError>;
    async fn disconnect(&mut self) -> Result<(), ScrapeError>;
}

#[async_trait]
impl ScrapeClient for Client {
    async fn goto(&mut self, url: &str) -> Result<(), ScrapeError> {
        self.goto(url).await.map_err(ScrapeError::WebdriverCommandError)
    }

    async fn find_all(&mut self, search: Locator<'_>) -> Result<Vec<Element>, ScrapeError> {
        self.find_all(search).await.map_err(ScrapeError::WebdriverCommandError)
    }

    async fn disconnect(&mut self) -> Result<(), ScrapeError> {
        self.close_window().await.map_err(ScrapeError::WebdriverCommandError)?;
        self.close().await.map_err(ScrapeError::WebdriverCommandError)
    }
}
