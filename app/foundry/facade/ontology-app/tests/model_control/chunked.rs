use axum::body::{Body, Bytes, HttpBody};
use http_body_util::Full;
use std::{
    collections::VecDeque,
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

type NextFrame = <http_body_util::combinators::Frame<'static, Full<Bytes>> as Future>::Output;

struct Chunks {
    chunks: VecDeque<Full<Bytes>>,
}
impl HttpBody for Chunks {
    type Data = Bytes;
    type Error = Infallible;
    fn poll_frame(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<NextFrame> {
        while let Some(chunk) = self.chunks.front_mut() {
            match Pin::new(chunk).poll_frame(context) {
                Poll::Ready(None) => {
                    self.chunks.pop_front();
                }
                result => return result,
            }
        }
        Poll::Ready(None)
    }
}

pub(crate) fn body(bytes: Vec<u8>) -> Body {
    let middle = bytes.len() / 2;
    Body::new(Chunks {
        chunks: VecDeque::from([
            Full::new(Bytes::from(bytes[..middle].to_vec())),
            Full::new(Bytes::from(bytes[middle..].to_vec())),
        ]),
    })
}
