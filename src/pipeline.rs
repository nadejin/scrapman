use crate::{
    client::ScrapeClient,
    stage::{FlowControl, ScrapeStage},
    value::JsonValue,
};
use fantoccini::{
    elements::Element,
    error::{CmdError, NewSessionError},
};
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FormatResult},
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScrapePipeline {
    stages: Vec<ScrapeStage>,
}

pub type ScrapePipelineResult = Result<(), ScrapeError>;

impl ScrapePipeline {
    pub fn push<T: Into<ScrapeStage>>(mut self, stage: T) -> Self {
        self.stages.push(stage.into());
        self
    }

    pub fn execute<'a>(&'a self, context: &'a mut ScrapeContext) -> BoxFuture<'a, ScrapePipelineResult> {
        async move {
            let mut idx = 0;
            loop {
                match self.stages.get(idx) {
                    Some(stage) => {
                        let result = stage.action.execute(context).await;

                        // TODO log error
                        dbg!(&result);

                        let flow = match result {
                            Ok(_) => &stage.on_complete,
                            Err(_) => &stage.on_error,
                        };

                        match flow {
                            FlowControl::Continue => idx += 1,
                            FlowControl::Quit => break,
                            FlowControl::Goto(next_stage) => {
                                match self.stages.iter().position(|stage| match &stage.name {
                                    Some(name) => name == next_stage,
                                    _ => false,
                                }) {
                                    Some(pos) => idx = pos,
                                    None => return Err(ScrapeError::MissingPipelineStage(next_stage.clone())),
                                }
                            }
                        };
                    }

                    None => break,
                }
            }

            Ok(())
        }
        .boxed()
    }
}

pub struct ScrapeContext {
    pub client: Box<dyn ScrapeClient>,
    pub model: JsonValue,
    pub values: JsonValue,
    pub models: Vec<JsonValue>,
    pub scoped_element: Option<Element>,
    pub current_element: Option<Element>,
}

impl ScrapeContext {
    pub fn new<C: ScrapeClient + 'static, V: Into<Option<JsonValue>>>(client: C, values: V) -> Self {
        ScrapeContext {
            client: Box::new(client),
            model: json!({}),
            values: values.into().unwrap_or(json!({})),
            models: Vec::new(),
            current_element: None,
            scoped_element: None,
        }
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    ValueResolveError,
    MissingElement,
    MissingPipelineStage(String),
    SetModelAttributeError(String),
    WebdriverConnectionError(NewSessionError),
    WebdriverCommandError(CmdError),
    TestError,
}

impl Display for ScrapeError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        match self {
            ScrapeError::ValueResolveError => {
                write!(fmt, "failed to resolve value")
            }

            ScrapeError::MissingElement => {
                write!(fmt, "required element is missing in the pipeline execution context")
            }

            ScrapeError::MissingPipelineStage(stage) => {
                write!(fmt, "missing pipeline stage {}", stage)
            }

            ScrapeError::SetModelAttributeError(attribute) => {
                write!(fmt, "failed to populate model attribute {}", attribute)
            }

            ScrapeError::WebdriverConnectionError(error) => {
                write!(fmt, "webdriver connection error: {}", error)
            }

            ScrapeError::WebdriverCommandError(error) => {
                write!(fmt, "webdriver command error: {}", error)
            }

            ScrapeError::TestError => {
                write!(fmt, "test error")
            }
        }
    }
}

impl Error for ScrapeError {}

#[cfg(test)]
mod test {
    use crate::{
        action::{OpenUrl, ScrapeAction, TestError, TestSuccess},
        client::MockScrapeClient,
        pipeline::{ScrapeContext, ScrapeError, ScrapePipeline},
        scrapman::Scrapman,
        stage::{FlowControl, ScrapeStage},
        value::Value,
        StoreModel,
    };
    use futures::future;
    use mockall::predicate;

    #[tokio::test]
    async fn test_plain_pipeline() {
        let url = "http://localhost";
        let pipeline = ScrapePipeline::default().push(OpenUrl::new(Value::constant(url)));

        let mut client = MockScrapeClient::new();
        client
            .expect_goto()
            .with(predicate::eq(url))
            .times(1)
            .returning(|_| Box::pin(future::ok(())));

        client
            .expect_disconnect()
            .times(1)
            .returning(|| Box::pin(future::ok(())));

        let scrapman = Scrapman::new(url);

        let result = scrapman.launch_with_client(pipeline, None, client).await;
        assert_eq!(true, result.is_ok());
    }

    #[tokio::test]
    async fn test_quit_on_error() {
        let pipeline = ScrapePipeline::default()
            .push(ScrapeStage::from(TestError).on_error(FlowControl::Quit))
            .push(StoreModel);

        let mut client = MockScrapeClient::new();
        client
            .expect_disconnect()
            .times(1)
            .returning(|| Box::pin(future::ok(())));

        let scrapman = Scrapman::new("");
        let result = scrapman.launch_with_client(pipeline, None, client).await;
        assert_eq!(true, result.is_ok());

        let ctx = result.unwrap();
        assert_eq!(0, ctx.models.len());
    }

    #[tokio::test]
    async fn test_conditional_pipeline_1() {
        let result = test_conditional_pipeline(TestSuccess).await;
        assert_eq!(true, result.is_ok());

        let ctx = result.unwrap();
        assert_eq!(0, ctx.models.len());
    }

    #[tokio::test]
    async fn test_conditional_pipeline_2() {
        let result = test_conditional_pipeline(TestError).await;
        assert_eq!(true, result.is_ok());

        let ctx = result.unwrap();
        assert_eq!(1, ctx.models.len());
    }

    async fn test_conditional_pipeline<T: ScrapeAction + 'static>(f: T) -> Result<ScrapeContext, ScrapeError> {
        let pipeline = ScrapePipeline::default()
            .push(ScrapeStage::from(f).on_complete(FlowControl::Goto("Complete".into())))
            .push(StoreModel)
            .push(ScrapeStage::from(TestSuccess).with_name("Complete"));

        let mut client = MockScrapeClient::new();
        client
            .expect_disconnect()
            .times(1)
            .returning(|| Box::pin(future::ok(())));

        let scrapman = Scrapman::new("");
        scrapman.launch_with_client(pipeline, None, client).await
    }
}
