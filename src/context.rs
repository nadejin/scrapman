use crate::value::JsonValue;
use fantoccini::elements::Element;
use serde_json::json;

pub struct ScrapeContext {
    pub model: JsonValue,
    pub values: JsonValue,
    pub models: Vec<JsonValue>,
    pub scope_element: Option<Element>,
    pub current_element: Option<Element>,
}

impl ScrapeContext {
    pub fn with_values(values: JsonValue) -> Self {
        let mut context = ScrapeContext::default();
        context.values = values;
        context
    }
}

impl Default for ScrapeContext {
    fn default() -> Self {
        ScrapeContext {
            current_element: None,
            scope_element: None,
            model: json!({}),
            values: json!({}),
            models: Vec::new(),
        }
    }
}
