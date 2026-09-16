use super::client::{Client, JmapResponse};
use serde_json::{Value, json};
use std::fmt::Display;

impl Client {
    pub async fn jmap_destroy(
        &self,
        object: impl Display,
        ids: impl IntoIterator<Item = impl Display>,
        args: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> JmapResponse {
        let ids: Vec<_> = ids.into_iter().map(|s| s.to_string()).collect();
        let mut value = json!({"accountId":"a","destroy":ids});
        for (key, v) in args {
            value[key.to_string()] = v;
        }
        self.jmap_method_call(&format!("{object}/set"), value).await
    }
    pub async fn jmap_method_calls(&self, calls: Value) -> JmapResponse {
        self.jmap_request(
            &["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            calls,
        )
        .await
    }
    pub async fn jmap_create(
        &self,
        object: impl Display,
        items: impl IntoIterator<Item = Value>,
        args: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> JmapResponse {
        let items = items
            .into_iter()
            .enumerate()
            .map(|(i, v)| (format!("i{i}"), v))
            .collect::<serde_json::Map<_, _>>();
        let mut value = json!({"accountId":"a","create":items});
        for (key, v) in args {
            value[key.to_string()] = v;
        }
        self.jmap_method_call(&format!("{object}/set"), value).await
    }
    pub async fn jmap_update(
        &self,
        object: impl Display,
        items: impl IntoIterator<Item = (impl Display, Value)>,
        args: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> JmapResponse {
        let items = items
            .into_iter()
            .map(|(i, v)| (i.to_string(), v))
            .collect::<serde_json::Map<_, _>>();
        let mut value = json!({"accountId":"a","update":items});
        for (key, v) in args {
            value[key.to_string()] = v;
        }
        self.jmap_method_call(&format!("{object}/set"), value).await
    }
    pub async fn jmap_changes(&self, object: &str, state: &str) -> JmapResponse {
        self.jmap_method_call(
            &format!("{object}/changes"),
            json!({"accountId":"a","sinceState":state}),
        )
        .await
    }
    pub async fn jmap_query(
        &self,
        object: impl Display,
        filter: impl IntoIterator<Item = (impl Display, Value)>,
        sort: impl IntoIterator<Item = impl Display>,
        args: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> JmapResponse {
        let filter = filter
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<serde_json::Map<_, _>>();
        let sort = sort
            .into_iter()
            .map(|s| json!({"property":s.to_string()}))
            .collect::<Vec<_>>();
        let mut value = json!({"accountId":"a","filter":filter,"sort":sort});
        for (key, v) in args {
            value[key.to_string()] = v;
        }
        self.jmap_method_call(&format!("{object}/query"), value)
            .await
    }
}
pub enum ChangeType<'a> {
    Created(&'a str),
    Updated(&'a str),
    Destroyed(&'a str),
}
impl JmapResponse {
    pub fn created(&self, n: u32) -> &Value {
        &self.method_response()["created"][format!("i{n}")]
    }
    pub fn not_updated(&self, id: &str) -> &Value {
        &self.method_response()["notUpdated"][id]
    }
    pub fn not_destroyed(&self, id: &str) -> &Value {
        &self.method_response()["notDestroyed"][id]
    }
    pub fn state(&self) -> &str {
        self.method_response()["state"].as_str().expect("state")
    }
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.method_response()["ids"]
            .as_array()
            .expect("ids")
            .iter()
            .map(|v| v.as_str().expect("id"))
    }
    pub fn not_found(&self) -> impl Iterator<Item = &str> {
        self.method_response()["notFound"]
            .as_array()
            .expect("notFound")
            .iter()
            .map(|v| v.as_str().expect("id"))
    }
    pub fn destroyed(&self) -> impl Iterator<Item = &str> {
        self.method_response()["destroyed"]
            .as_array()
            .expect("destroyed")
            .iter()
            .map(|v| v.as_str().expect("id"))
    }
    pub fn changes(&self) -> impl Iterator<Item = ChangeType<'_>> {
        ["created", "updated", "destroyed"]
            .into_iter()
            .flat_map(|key| {
                self.method_response()[key]
                    .as_array()
                    .expect("changes")
                    .iter()
                    .map(move |v| {
                        let id = v.as_str().expect("id");
                        match key {
                            "created" => ChangeType::Created(id),
                            "updated" => ChangeType::Updated(id),
                            _ => ChangeType::Destroyed(id),
                        }
                    })
            })
    }
}
