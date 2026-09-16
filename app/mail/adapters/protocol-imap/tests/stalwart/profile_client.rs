use axum::{Router, body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use std::fmt::Display;
use tower::ServiceExt;
pub struct Client {
    pub router: Router,
}
pub struct Response(pub(super) Value);
impl Client {
    pub async fn jmap_method_call(&self, name: &str, args: Value) -> Response {
        self.jmap_method_calls(json!([[name, args, "c"]])).await
    }
    pub async fn jmap_method_calls(&self, calls: Value) -> Response {
        let mut using = vec!["urn:ietf:params:jmap:core"];
        for call in calls.as_array().expect("methodCalls") {
            let name = call[0].as_str().expect("method name");
            let capability =
                if name.starts_with("Identity/") || name.starts_with("EmailSubmission/") {
                    "urn:ietf:params:jmap:submission"
                } else if name.starts_with("VacationResponse/") {
                    "urn:ietf:params:jmap:vacationresponse"
                } else {
                    "urn:ietf:params:jmap:mail"
                };
            if !using.contains(&capability) {
                using.push(capability);
            }
        }
        let request = Request::post("/jmap")
            .header("authorization", format!("Bearer {}", super::TOKEN))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"using":using,"methodCalls":calls}).to_string(),
            ))
            .unwrap();
        let response = self.router.clone().oneshot(request).await.unwrap();
        assert!(response.status().is_success());
        let value: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        Response(value)
    }
    pub async fn jmap_get(
        &self,
        object: &str,
        properties: impl IntoIterator<Item = impl Display>,
        ids: impl IntoIterator<Item = impl Display>,
    ) -> Response {
        let properties: Vec<_> = properties.into_iter().map(|p| p.to_string()).collect();
        let ids: Vec<_> = ids.into_iter().map(|id| id.to_string()).collect();
        self.jmap_method_call(&format!("{object}/get"), json!({"accountId":"a","properties":if properties.is_empty(){None}else{Some(properties)},"ids":if ids.is_empty(){None}else{Some(ids)}})).await
    }
    pub async fn jmap_changes(&self, object: &str, state: &str) -> Response {
        self.jmap_method_call(
            &format!("{object}/changes"),
            json!({"accountId":"a","sinceState":state}),
        )
        .await
    }
    pub async fn jmap_update(
        &self,
        object: &str,
        updates: impl IntoIterator<Item = (impl Display, Value)>,
        args: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> Response {
        let updates = updates
            .into_iter()
            .map(|(id, value)| (id.to_string(), value))
            .collect::<serde_json::Map<_, _>>();
        let mut value = json!({"accountId":"a","update":updates});
        for (key, arg) in args {
            value[key.to_string()] = arg;
        }
        self.jmap_method_call(&format!("{object}/set"), value).await
    }
    pub async fn jmap_query(
        &self,
        object: &str,
        filter: impl IntoIterator<Item = (impl Display, Value)>,
        sort: impl IntoIterator<Item = impl Display>,
        args: impl IntoIterator<Item = (impl Display, Value)>,
    ) -> Response {
        let filter = filter
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect::<serde_json::Map<_, _>>();
        let sort = sort
            .into_iter()
            .map(|p| json!({"property":p.to_string()}))
            .collect::<Vec<_>>();
        let mut value = json!({"accountId":"a","filter":filter,"sort":sort});
        for (key, arg) in args {
            value[key.to_string()] = arg;
        }
        self.jmap_method_call(&format!("{object}/query"), value)
            .await
    }
}
impl Response {
    pub fn method_response(&self) -> &Value {
        &self.0["methodResponses"][0][1]
    }
    pub fn list(&self) -> &[Value] {
        self.method_response()["list"].as_array().expect("get list")
    }
    pub fn state(&self) -> &str {
        self.method_response()["state"].as_str().expect("state")
    }
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.method_response()["ids"]
            .as_array()
            .expect("query IDs")
            .iter()
            .map(|id| id.as_str().unwrap())
    }
    pub fn not_found(&self) -> impl Iterator<Item = &str> {
        self.method_response()["notFound"]
            .as_array()
            .expect("notFound")
            .iter()
            .map(|id| id.as_str().unwrap())
    }
}
