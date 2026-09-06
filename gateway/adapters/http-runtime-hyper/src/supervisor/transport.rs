use super::*;

impl Context {
    async fn request(&self, request: Request<Incoming>) -> Response<Full<Bytes>> {
        let is_http1 = request.version() != hyper::Version::HTTP_2;
        let permit = match self.control.admission.acquire(Budget::Request) {
            Ok(permit) => permit,
            Err(error) => {
                self.control.admission.record(RuntimeEvent::RequestRefused);
                if error == AdmissionRefusal::Poisoned {
                    self.control.request_drain();
                }
                return refusal(503, is_http1, None);
            }
        };
        let deadline = self.control.snapshot().limits.body_deadline;
        let parsed = match tokio::time::timeout(
            deadline,
            collect_hyper_request(request, self.config.max_body_bytes),
        )
        .await
        {
            Ok(Ok(parsed)) => parsed,
            Ok(Err(error)) => {
                if matches!(error, HyperRuntimeError::BodyTooLarge { .. }) {
                    self.control.admission.record(RuntimeEvent::BodyLimit);
                }
                return refusal(error.status_code(), is_http1, Some(permit));
            }
            Err(_) => {
                self.control.admission.record(RuntimeEvent::BodyTimeout);
                return refusal(408, is_http1, Some(permit));
            }
        };
        let response = match self.execution.submit(
            parsed,
            self.router.clone(),
            self.chain.clone(),
            permit.clone(),
        ) {
            Ok(response) => response,
            Err(error) => {
                self.control.admission.record(RuntimeEvent::RequestRefused);
                if !matches!(
                    error,
                    ExecutionFailure::Admission(
                        AdmissionRefusal::Capacity(_) | AdmissionRefusal::Draining
                    )
                ) {
                    self.control.request_drain();
                    return refusal(500, is_http1, Some(permit));
                }
                return refusal(503, is_http1, Some(permit));
            }
        };
        match response.await {
            Ok(response) => crate::response::convert(response.response, Some(response.request)),
            Err(_) => refusal(500, is_http1, Some(permit)),
        }
    }
}

fn refusal(status: u16, close: bool, permit: Option<Arc<Permit>>) -> Response<Full<Bytes>> {
    let mut response = HttpResponse::new(status).with_body(b"request refused".to_vec());
    if close {
        response = response.with_header("connection", "close");
    }
    crate::response::convert(response, permit)
}

pub(super) async fn connection(stream: TcpStream, context: Arc<Context>) -> Result<(), String> {
    let mut drain = context.control.drain.subscribe();
    let service_context = context.clone();
    let service = service_fn(move |request| {
        let context = service_context.clone();
        async move { Ok::<_, std::convert::Infallible>(context.request(request).await) }
    });
    let mut builder = Builder::new(TokioExecutor::new());
    builder
        .http1()
        .header_read_timeout(context.config.header_read_timeout)
        .keep_alive(true)
        .timer(TokioTimer::new());
    let requests = context.control.snapshot().limits.capacity(Budget::Request);
    let streams = u32::try_from(requests).unwrap_or(u32::MAX);
    builder
        .http2()
        .max_concurrent_streams(streams)
        .keep_alive_interval(Some(context.config.keepalive_timeout / 2))
        .keep_alive_timeout(context.config.keepalive_timeout)
        .timer(TokioTimer::new());
    let connection = builder.serve_connection(TokioIo::new(stream), service);
    tokio::pin!(connection);
    if !*drain.borrow_and_update() {
        tokio::select! {
            result = &mut connection => return result.map_err(|error| error.to_string()),
            _ = drain.changed() => {},
        }
    }
    connection.as_mut().graceful_shutdown();
    connection.await.map_err(|error| error.to_string())
}
