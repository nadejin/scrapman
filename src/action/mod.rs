mod click_element;
mod fill_element;
mod open_url;
mod pause;
mod query_element;
mod set_model_attribute;
mod store_model;

#[cfg(test)]
mod test;

pub use click_element::ClickElement;
pub use fill_element::FillElement;
pub use open_url::OpenUrl;
pub use pause::Pause;
pub use query_element::{ElementScope, QueryElement, Selector};
pub use set_model_attribute::SetModelAttribute;
pub use store_model::StoreModel;

#[cfg(test)]
pub use test::{TestError, TestSuccess};

use crate::pipeline::{ScrapeContext, ScrapeError};
use async_trait::async_trait;
use std::fmt::{Debug, Display};

pub type ScrapeActionResult = Result<(), ScrapeError>;

#[async_trait]
#[typetag::serde(tag = "type")]
pub trait ScrapeAction: Display + Send + Sync + Debug {
    async fn execute(&self, context: &mut ScrapeContext) -> ScrapeActionResult;
}
