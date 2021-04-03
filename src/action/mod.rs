mod click_element;
mod fill_element;
mod open_url;
mod query_element;
mod set_model_attribute;
mod store_model;

pub use click_element::ClickElement;
pub use fill_element::FillElement;
pub use open_url::OpenUrl;
pub use query_element::{ElementScope, QueryElement, Selector};
pub use set_model_attribute::SetModelAttribute;
pub use store_model::StoreModel;

use crate::pipeline::{ScrapeContext, ScrapeResult};
use async_trait::async_trait;
use fantoccini::Client;
use std::fmt;

#[async_trait]
#[typetag::serde(tag = "type")]
pub trait ScrapeAction: fmt::Display + Send + Sync {
    async fn execute(&self, client: &mut Client, context: &mut ScrapeContext) -> ScrapeResult;
}
