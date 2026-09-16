use super::TOKEN;
use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Request},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

pub struct Client {
    pub router: Router,
}
impl Client {
    pub async fn jmap_method_call(&self, name: &str, args: Value) -> JmapResponse {
        self.jmap_request(
            &["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            json!([[name, args, "c0"]]),
        )
        .await
    }
    pub async fn jmap_request(&self, using: &[&str], calls: Value) -> JmapResponse {
        let response = self
            .jmap_raw_post(
                json!({"using":using,"methodCalls":calls}).to_string(),
                "application/json",
            )
            .await;
        JmapResponse(serde_json::from_slice(&response.body).expect("JSON response"))
    }
    pub async fn jmap_raw_post(&self, body: impl Into<Vec<u8>>, content_type: &str) -> RawResponse {
        let response = self
            .router
            .clone()
            .oneshot(
                Request::post("/jmap")
                    .header("authorization", format!("Bearer {TOKEN}"))
                    .header("content-type", content_type)
                    .body(Body::from(body.into()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        RawResponse {
            status,
            headers,
            body: response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        }
    }
}
pub struct RawResponse {
    headers: HeaderMap,
    pub status: u16,
    pub body: Vec<u8>,
}
impl RawResponse {
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }
}
#[derive(Debug)]
pub struct JmapResponse(pub Value);
impl JmapResponse {
    pub fn method_response(&self) -> &Value {
        self.response_at(0)
    }
    pub fn response_at(&self, n: usize) -> &Value {
        &self.0["methodResponses"][n][1]
    }
    pub fn name_at(&self, n: usize) -> &str {
        self.0["methodResponses"][n][0].as_str().unwrap_or("")
    }
    pub fn error_type_at(&self, n: usize) -> Option<&str> {
        self.response_at(n)["type"].as_str()
    }
    pub fn num_responses(&self) -> usize {
        self.0["methodResponses"].as_array().map_or(0, Vec::len)
    }
    pub fn call_id_at(&self, n: usize) -> &str {
        self.0["methodResponses"][n][2].as_str().unwrap_or("")
    }
    pub fn session_state(&self) -> Option<&str> {
        self.0["sessionState"].as_str()
    }
}

impl Client {
    pub async fn http_raw(&self, request: Request<Body>) -> RawResponse {
        let response = self.router.clone().oneshot(request).await.unwrap();
        RawResponse {
            status: response.status().as_u16(),
            headers: response.headers().clone(),
            body: response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        }
    }
    pub async fn http_get_raw(&self, url: &str, _: Option<&str>) -> RawResponse {
        self.http_raw(
            Request::get(url)
                .header("authorization", format!("Bearer {TOKEN}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
    }
    pub async fn jmap_get(
        &self,
        object: &str,
        properties: impl IntoIterator<Item = impl std::fmt::Display>,
        ids: impl IntoIterator<Item = impl std::fmt::Display>,
    ) -> JmapResponse {
        let properties: Vec<_> = properties.into_iter().map(|p| p.to_string()).collect();
        let ids: Vec<_> = ids.into_iter().map(|p| p.to_string()).collect();
        self.jmap_method_call(
            &format!("{object}/get"),
            json!({"accountId":"a",
            "properties":if properties.is_empty(){None}else{Some(properties)},
            "ids":if ids.is_empty(){None}else{Some(ids)}}),
        )
        .await
    }
}
impl RawResponse {
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type")?.to_str().ok()
    }
}
impl JmapResponse {
    pub fn is_error_at(&self, n: usize) -> bool {
        self.name_at(n) == "error"
    }
    pub fn list(&self) -> &[Value] {
        self.method_response()["list"].as_array().expect("get list")
    }
}
pub trait JmapUtils {
    fn id(&self) -> &str {
        self.text_field("id")
    }
    fn text_field(&self, field: &str) -> &str;
    fn integer_field(&self, field: &str) -> i64;
    fn blob_id(&self) -> &str {
        self.text_field("blobId")
    }
    fn typ(&self) -> &str {
        self.text_field("type")
    }
}
impl JmapUtils for Value {
    fn text_field(&self, field: &str) -> &str {
        self[field].as_str().expect("string property")
    }
    fn integer_field(&self, field: &str) -> i64 {
        self[field].as_i64().expect("integer property")
    }
}
