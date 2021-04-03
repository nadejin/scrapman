use crate::action::ScrapeAction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ErrorBehavior {
    Continue,
    Quit,
    Goto(String),
}

#[derive(Serialize, Deserialize)]
pub struct ScrapeStage {
    pub name: Option<String>,
    pub action: Box<dyn ScrapeAction>,
    pub on_error: ErrorBehavior,
}

impl<Name, Action> Into<ScrapeStage> for (Name, Action, ErrorBehavior)
where
    Name: Into<String>,
    Action: 'static + ScrapeAction,
{
    fn into(self) -> ScrapeStage {
        ScrapeStage {
            name: Some(self.0.into()),
            action: Box::new(self.1),
            on_error: self.2,
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
            on_error: ErrorBehavior::Continue,
        }
    }
}

impl<Action> Into<ScrapeStage> for (Action, ErrorBehavior)
where
    Action: 'static + ScrapeAction,
{
    fn into(self) -> ScrapeStage {
        ScrapeStage {
            name: None,
            action: Box::new(self.0),
            on_error: self.1,
        }
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
            on_error: ErrorBehavior::Continue,
        }
    }
}
