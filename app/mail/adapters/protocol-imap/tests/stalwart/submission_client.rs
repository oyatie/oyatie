use super::client::{Client, Response};
use serde_json::{Value, json};
use std::fmt::Display;

impl Client {
    pub async fn jmap_create(
        &self,
        object: &str,
        items: impl IntoIterator<Item = Value>,
        arguments: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> Response {
        let create = items
            .into_iter()
            .enumerate()
            .map(|(n, item)| (format!("i{n}"), item))
            .collect::<serde_json::Map<_, _>>();
        let mut args = json!({"accountId":"a","create":create});
        for (key, value) in arguments {
            args[key.to_string()] = value;
        }
        self.jmap_method_call(&format!("{object}/set"), args).await
    }

    pub async fn jmap_destroy(
        &self,
        object: &str,
        ids: impl IntoIterator<Item = impl Display>,
        arguments: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> Response {
        let ids = ids.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
        let mut args = json!({"accountId":"a","destroy":ids});
        for (key, value) in arguments {
            args[key.to_string()] = value;
        }
        self.jmap_method_call(&format!("{object}/set"), args).await
    }
}

impl Response {
    pub fn pointer(&self, pointer: &str) -> Option<&Value> {
        self.0.pointer(pointer)
    }
    pub fn created(&self, index: usize) -> &Value {
        &self.method_response()["created"][format!("i{index}")]
    }
    pub fn not_created(&self, index: usize) -> &Value {
        &self.method_response()["notCreated"][format!("i{index}")]
    }
    pub fn not_destroyed(&self, id: &str) -> &Value {
        &self.method_response()["notDestroyed"][id]
    }
    pub fn num_responses(&self) -> usize {
        self.0["methodResponses"].as_array().map_or(0, Vec::len)
    }
    pub fn name_at(&self, index: usize) -> &str {
        self.0["methodResponses"][index][0].as_str().unwrap_or("")
    }
}

pub trait JmapUtils {
    fn id(&self) -> &str;
    fn typ(&self) -> &str;
}
impl JmapUtils for Value {
    fn id(&self) -> &str {
        self["id"].as_str().expect("string property")
    }
    fn typ(&self) -> &str {
        self["type"].as_str().expect("string property")
    }
}
