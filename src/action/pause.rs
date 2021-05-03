use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::ScrapeContext,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};
use tokio::time::{sleep, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Pause {
    duration: f64,
}

impl Pause {
    pub fn new(duration: f64) -> Self {
        Pause { duration }
    }
}

impl Display for Pause {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "Pause")
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for Pause {
    async fn execute(&self, _: &mut ScrapeContext) -> ScrapeActionResult {
        sleep(Duration::from_secs_f64(self.duration)).await;
        Ok(())
    }
}
