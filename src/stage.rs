use crate::action::ScrapeAction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FlowControl {
    Continue,
    Quit,
    Goto(String),
    Repeat { delay: Option<f64> },
}

impl FlowControl {
    pub fn goto<T: Into<String>>(stage: T) -> Self {
        FlowControl::Goto(stage.into())
    }

    pub fn repeat() -> Self {
        FlowControl::Repeat { delay: None }
    }

    pub fn repeat_with_delay(delay: f64) -> Self {
        FlowControl::Repeat { delay: Some(delay) }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScrapeStage {
    pub name: Option<String>,
    pub action: Box<dyn ScrapeAction>,
    pub on_complete: FlowControl,
    pub on_error: FlowControl,
}

impl ScrapeStage {
    pub fn with_name<T: Into<String>>(mut self, name: T) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn on_complete(mut self, on_complete: FlowControl) -> Self {
        self.on_complete = on_complete;
        self
    }

    pub fn on_any_error(mut self, on_error: FlowControl) -> Self {
        self.on_error = on_error;
        self
    }
}

impl<Action> From<Action> for ScrapeStage
where
    Action: 'static + ScrapeAction,
{
    fn from(action: Action) -> Self {
        ScrapeStage {
            name: None,
            action: Box::new(action),
            on_complete: FlowControl::Continue,
            on_error: FlowControl::Continue,
        }
    }
}

impl<Name, Action> Into<ScrapeStage> for (Name, Action)
where
    Name: Into<String>,
    Action: 'static + ScrapeAction,
{
    fn into(self) -> ScrapeStage {
        ScrapeStage {
            name: Some(self.0.into()),
            action: Box::new(self.1),
            on_complete: FlowControl::Continue,
            on_error: FlowControl::Continue,
        }
    }
}
