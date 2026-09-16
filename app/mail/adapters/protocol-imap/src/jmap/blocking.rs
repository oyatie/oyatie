use super::Jmap;
use axum::http::StatusCode;

impl Jmap {
    pub(super) async fn blocking<T: Send + 'static>(
        &self,
        work: impl FnOnce(Jmap) -> Result<T, StatusCode> + Send + 'static,
    ) -> Result<T, StatusCode> {
        // Bound queued and executing work, including unauthenticated calls.
        // The worker retains admission if its HTTP caller disconnects.
        let permit = self
            .workers
            .clone()
            .try_acquire_owned()
            .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
        let state = self.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            work(state)
        })
        .await
        .unwrap_or(Err(StatusCode::SERVICE_UNAVAILABLE))
    }
}
