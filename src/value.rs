use crate::ScrapeContext;
use serde::{Deserialize, Serialize};

pub type JsonValue = serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    Constant(String),
    Context(String),
    ElementText,
}

impl Value {
    pub fn resolve<'a>(&'a self, _context: &'a ScrapeContext) -> Option<&'a str> {
        match self {
            Value::Constant(value) => Some(value),
            _ => None,
        }
    }

    pub fn constant<T: Into<String>>(value: T) -> Self {
        Value::Constant(value.into())
    }

    pub fn context<T: Into<String>>(value: T) -> Self {
        Value::Context(value.into())
    }
}
