use crate::{
    action::{ScrapeAction, ScrapeActionResult},
    pipeline::{ScrapeContext, ScrapeError},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuccess;

impl Display for TestSuccess {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "TestSuccess")
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for TestSuccess {
    async fn execute(&self, _: &mut ScrapeContext) -> ScrapeActionResult {
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestError;

impl Display for TestError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        write!(fmt, "TestError")
    }
}

#[async_trait]
#[typetag::serde]
impl ScrapeAction for TestError {
    async fn execute(&self, _: &mut ScrapeContext) -> ScrapeActionResult {
        Err(ScrapeError::TestError)
    }
}
