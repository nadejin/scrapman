pub mod pipeline;
pub mod scrapman;
pub mod stage;
pub mod value;

pub use crate::{
    pipeline::{ScrapePipeline, ScrapePipelineContext, ScrapePipelineStage, ScrapeResult},
    scrapman::Scrapman,
    stage::{ClickElement, ElementScope, FillElement, OpenUrl, QueryElement, Selector, SetModelAttribute, StoreModel},
    value::{JsonValue, Value},
};
