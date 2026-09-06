use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper::Response;

use crate::HttpResponse;
use crate::admission::Permit;

struct OwnedResponseBytes {
    bytes: Vec<u8>,
    _request: Arc<Permit>,
}

impl AsRef<[u8]> for OwnedResponseBytes {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

fn response_bytes(bytes: Vec<u8>, request: Option<Arc<Permit>>) -> Bytes {
    match request {
        Some(request) => Bytes::from_owner(OwnedResponseBytes {
            bytes,
            _request: request,
        }),
        None => Bytes::from(bytes),
    }
}

pub(crate) fn convert(
    response: HttpResponse,
    request: Option<Arc<Permit>>,
) -> Response<Full<Bytes>> {
    let mut builder = Response::builder().status(response.status);
    for (name, value) in &response.headers {
        builder = builder.header(name, value);
    }
    builder
        .body(Full::new(response_bytes(response.body, request.clone())))
        .unwrap_or_else(|_| {
            let mut fallback = Response::new(Full::new(response_bytes(
                b"response build failed".to_vec(),
                request,
            )));
            *fallback.status_mut() = hyper::StatusCode::INTERNAL_SERVER_ERROR;
            fallback
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServingLimits;
    use crate::admission::{Admission, Budget};
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn frame_handoff_and_byte_clones_retain_request_capacity() {
        let admission = Admission::new(ServingLimits::default());
        let permit = admission.acquire(Budget::Request).unwrap();
        let response = convert(
            HttpResponse::new(200).with_body(b"payload".to_vec()),
            Some(permit),
        );
        let mut body = response.into_body();
        let bytes = body.frame().await.unwrap().unwrap().into_data().unwrap();
        drop(body);
        assert_eq!(admission.snapshot().active, [0, 1, 0]);
        let slice = bytes.slice(1..3);
        drop(bytes);
        assert_eq!(admission.snapshot().active, [0, 1, 0]);
        assert_eq!(slice.as_ref(), b"ay");
        drop(slice);
        assert_eq!(admission.snapshot().active, [0; 3]);
    }

    #[tokio::test]
    async fn invalid_response_retains_capacity_through_fixed_fallback() {
        let admission = Admission::new(ServingLimits::default());
        let permit = admission.acquire(Budget::Request).unwrap();
        let mut original = HttpResponse::new(200);
        original
            .headers
            .insert("invalid header name".into(), "value".into());
        let response = convert(original, Some(permit));
        assert_eq!(response.status(), 500);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(bytes.as_ref(), b"response build failed");
        assert_eq!(admission.snapshot().active, [0, 1, 0]);
        drop(bytes);
        assert_eq!(admission.snapshot().active, [0; 3]);
    }

    #[test]
    fn dropping_unsent_response_releases_capacity() {
        let admission = Admission::new(ServingLimits::default());
        let response = convert(
            HttpResponse::new(200).with_body(b"body".to_vec()),
            Some(admission.acquire(Budget::Request).unwrap()),
        );
        assert_eq!(admission.snapshot().active, [0, 1, 0]);
        drop(response);
        assert_eq!(admission.snapshot().active, [0; 3]);
    }
}
