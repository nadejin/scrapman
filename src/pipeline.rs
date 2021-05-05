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
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FormatResult},
};
use tokio::time::{sleep, Duration};

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
                        match stage.name {
                            Some(ref name) => info!("Executing {}: {}", name, stage.action),
                            None => info!("Executing: {}", stage.action),
                        }

                        // Stage action execution, flow control evaluation based on the result
                        let flow: &FlowControl;
                        match stage.action.execute(context).await {
                            // "On complete" branch is executed
                            Ok(_) => flow = &stage.on_complete,

                            // Internal client error - pipeline execution is stopped, the error is propagated
                            Err(e @ ScrapeError::WebdriverCommandError(_)) => return Err(e),

                            // Stage action execution failure - "on error" branch is executed
                            Err(error) => {
                                warn!("Action execution failure: {}", error);
                                flow = &stage.on_error;
                            }
                        }

                        match flow {
                            // Following pipeline stage is executed
                            FlowControl::Continue => idx += 1,

                            // Pipeline execution is stopped
                            FlowControl::Quit => {
                                info!("Flow control quit, stopping the pipeline execution");
                                break;
                            }

                            // Pipeline execution is redirected to a named stage
                            FlowControl::Goto(next_stage) => {
                                info!("Flow control redirection to stage \"{}\"", next_stage);
                                match self.stages.iter().position(|stage| match &stage.name {
                                    Some(name) => next_stage.eq(name),
                                    _ => false,
                                }) {
                                    Some(pos) => idx = pos,
                                    None => return Err(ScrapeError::MissingPipelineStage),
                                }
                            }

                            // Current pipeline stage execution is repeated after an optional delay
                            FlowControl::Repeat { delay } => {
                                match delay {
                                    Some(x) => info!("Repeating stage after {} seconds", x),
                                    None => info!("Repeating stage immediately"),
                                };

                                if let Some(delay) = *delay {
                                    sleep(Duration::from_secs_f64(delay)).await;
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
    ElementQueryEmptyResult,
    MissingElement,
    MissingUrl,
    MissingQuery,
    MissingPipelineStage,
    SetModelAttributeError,
    TestError,
    WebdriverConnectionError(NewSessionError),
    WebdriverCommandError(CmdError),
}

impl Display for ScrapeError {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> FormatResult {
        match self {
            ScrapeError::ValueResolveError => {
                write!(fmt, "failed to resolve value")
            }

            ScrapeError::ElementQueryEmptyResult => {
                write!(fmt, "element query empty result")
            }

            ScrapeError::MissingElement => {
                write!(fmt, "required element is missing in the pipeline execution context")
            }

            ScrapeError::MissingUrl => {
                write!(fmt, "missing URL to open")
            }

            ScrapeError::MissingQuery => {
                write!(fmt, "missing element query")
            }

            ScrapeError::MissingPipelineStage => {
                write!(fmt, "missing specified pipeline stage")
            }

            ScrapeError::SetModelAttributeError => {
                write!(fmt, "failed to populate model attribute")
            }

            ScrapeError::TestError => {
                write!(fmt, "test error")
            }

            ScrapeError::WebdriverConnectionError(error) => {
                write!(fmt, "webdriver connection error: {}", error)
            }

            ScrapeError::WebdriverCommandError(error) => {
                write!(fmt, "webdriver command error: {}", error)
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
        let pipeline = ScrapePipeline::default().push(OpenUrl::new(Value::Constant(url.into())));

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
            .push(ScrapeStage::from(TestError).on_any_error(FlowControl::Quit))
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
