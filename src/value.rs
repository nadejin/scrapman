use crate::ScrapeContext;
use serde::{Deserialize, Serialize};

pub type JsonValue = serde_json::Value;

const KEY_SEPARATOR: &'static str = ".";

#[derive(Serialize, Deserialize, Debug)]
pub enum Value {
    Constant(String),
    Context(String),
    CurrentElementText,
    ScopeElementText,
}

impl Value {
    pub fn resolve(&self, ctx: &ScrapeContext) -> Option<String> {
        match self {
            Value::Constant(value) => Some(value.clone()),
            Value::Context(key) => get_value_by_key(&ctx.values, key),
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

fn get_value_by_key(value: &serde_json::Value, key: &str) -> Option<String> {
    let mut split = key.split(KEY_SEPARATOR);
    let head = split.next()?;
    let tail = split.collect::<Vec<&str>>().join(KEY_SEPARATOR);

    match value {
        serde_json::Value::Object(object) => object.get(head).and_then(|value| {
            if tail.len() > 0 {
                get_value_by_key(value, &tail)
            } else {
                Some(value_to_string(value))
            }
        }),

        serde_json::Value::Array(array) => {
            let idx = head.parse::<usize>().ok()?;
            array.get(idx).and_then(|item| {
                if tail.len() > 0 {
                    get_value_by_key(item, &tail)
                } else {
                    Some(value_to_string(value))
                }
            })
        }

        _ => None,
    }
}

fn value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod test {

    use serde_json::json;

    #[test]
    fn test_get_value_by_key() {
        use super::get_value_by_key;

        let value = json!({
            "key": "value",
            "nested": {
                "key": "value"
            },
            "array": [
                { "key": "value" },
                { "key": "value" },
            ],
            "number": 123,
            "bool": false,
            "null": null,
        });

        let expected = Some(String::from("value"));

        assert_eq!(expected, get_value_by_key(&value, "key"));
        assert_eq!(expected, get_value_by_key(&value, "nested.key"));
        assert_eq!(None, get_value_by_key(&value, "missing.key"));

        assert_eq!(expected, get_value_by_key(&value, "array.0.key"));
        assert_eq!(expected, get_value_by_key(&value, "array.1.key"));
        assert_eq!(None, get_value_by_key(&value, "array.2.key"));

        assert_eq!(Some(String::from("123")), get_value_by_key(&value, "number"));
        assert_eq!(Some(String::from("false")), get_value_by_key(&value, "bool"));
        assert_eq!(Some(String::from("null")), get_value_by_key(&value, "null"));
    }
}
