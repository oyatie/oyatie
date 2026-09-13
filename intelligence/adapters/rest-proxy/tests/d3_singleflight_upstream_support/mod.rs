mod upstream;

pub use upstream::*;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use intelligence_rest_proxy::{
    AnthropicAdapter, RestAdapterError, SecretProviderFuture, SecretProviderStore,
    UpstreamOAuthSingleflight,
};
use scripted_http_server::ScriptedServer;
use tokio::sync::Semaphore;

pub type RefreshFuture =
    Pin<Box<dyn Future<Output = Result<String, RestAdapterError>> + Send + 'static>>;

#[derive(Default)]
pub struct StoreState {
    tokens: HashMap<String, String>,
    fetch_attempts: HashMap<String, usize>,
    store_attempts: HashMap<String, usize>,
    fetch_panics_remaining: usize,
    store_panics_remaining: usize,
    store_hangs_remaining: usize,
    transient_store_failure: Option<RestAdapterError>,
    fetch_error: Option<RestAdapterError>,
    store_error: Option<RestAdapterError>,
}

#[derive(Clone)]
pub struct RecordingStore {
    state: Arc<Mutex<StoreState>>,
    fetch_started: Option<Arc<Semaphore>>,
    fetch_release: Option<Arc<Semaphore>>,
}

impl RecordingStore {
    pub fn new(tokens: impl IntoIterator<Item = (&'static str, &'static str)>) -> Self {
        Self {
            state: Arc::new(Mutex::new(StoreState {
                tokens: tokens
                    .into_iter()
                    .map(|(handle, token)| (handle.to_string(), token.to_string()))
                    .collect(),
                ..StoreState::default()
            })),
            fetch_started: None,
            fetch_release: None,
        }
    }

    pub fn with_fetch_gate(
        tokens: impl IntoIterator<Item = (&'static str, &'static str)>,
    ) -> (Self, Arc<Semaphore>, Arc<Semaphore>) {
        let started = Arc::new(Semaphore::new(0));
        let release = Arc::new(Semaphore::new(0));
        let mut store = Self::new(tokens);
        store.fetch_started = Some(Arc::clone(&started));
        store.fetch_release = Some(Arc::clone(&release));
        (store, started, release)
    }

    pub fn fetch_attempts(&self, handle: &str) -> usize {
        self.state
            .lock()
            .unwrap()
            .fetch_attempts
            .get(handle)
            .copied()
            .unwrap_or_default()
    }

    pub fn store_attempts(&self, handle: &str) -> usize {
        self.state
            .lock()
            .unwrap()
            .store_attempts
            .get(handle)
            .copied()
            .unwrap_or_default()
    }

    pub fn token(&self, handle: &str) -> String {
        self.state.lock().unwrap().tokens[handle].clone()
    }

    pub fn set_fetch_error(&self, error: Option<RestAdapterError>) {
        self.state.lock().unwrap().fetch_error = error;
    }

    pub fn panic_next_fetch(&self) {
        self.state.lock().unwrap().fetch_panics_remaining += 1;
    }

    pub fn fail_next_store(&self, error: RestAdapterError) {
        self.state.lock().unwrap().transient_store_failure = Some(error);
    }

    pub fn panic_next_store(&self) {
        self.state.lock().unwrap().store_panics_remaining += 1;
    }

    pub fn hang_next_store(&self) {
        self.state.lock().unwrap().store_hangs_remaining += 1;
    }

    pub fn set_store_error(&self, error: Option<RestAdapterError>) {
        self.state.lock().unwrap().store_error = error;
    }
}

impl SecretProviderStore for RecordingStore {
    fn fetch_refresh_token<'a>(&'a self, handle: &'a str) -> SecretProviderFuture<'a, String> {
        Box::pin(async move {
            if let Some(started) = &self.fetch_started {
                started.add_permits(1);
            }
            if let Some(release) = &self.fetch_release {
                release.acquire().await.unwrap().forget();
            }

            let mut state = self.state.lock().unwrap();
            *state.fetch_attempts.entry(handle.to_string()).or_default() += 1;
            if state.fetch_panics_remaining > 0 {
                state.fetch_panics_remaining -= 1;
                drop(state);
                panic!("injected refresh-token fetch panic");
            }
            if let Some(error) = &state.fetch_error {
                return Err(error.clone());
            }
            state
                .tokens
                .get(handle)
                .cloned()
                .ok_or(RestAdapterError::SecretNotFound)
        })
    }

    fn store_refresh_token<'a>(
        &'a self,
        handle: &'a str,
        plaintext: &'a str,
    ) -> SecretProviderFuture<'a, ()> {
        Box::pin(async move {
            let should_hang = {
                let mut state = self.state.lock().unwrap();
                *state.store_attempts.entry(handle.to_string()).or_default() += 1;
                if state.store_panics_remaining > 0 {
                    state.store_panics_remaining -= 1;
                    drop(state);
                    panic!("injected refresh-token store panic");
                }
                if state.store_hangs_remaining > 0 {
                    state.store_hangs_remaining -= 1;
                    true
                } else {
                    if let Some(error) = state.transient_store_failure.take() {
                        return Err(error);
                    }
                    if let Some(error) = &state.store_error {
                        return Err(error.clone());
                    }
                    state
                        .tokens
                        .insert(handle.to_string(), plaintext.to_string());
                    false
                }
            };
            if should_hang {
                std::future::pending::<()>().await;
            }
            Ok(())
        })
    }
}

pub fn adapter(
    store: RecordingStore,
    server: &ScriptedServer,
    singleflight: Arc<UpstreamOAuthSingleflight>,
) -> Arc<AnthropicAdapter<RecordingStore>> {
    Arc::new(AnthropicAdapter::with_base_url_and_singleflight(
        store,
        server.base_url().to_owned(),
        singleflight,
    ))
}

pub fn refresh_future(
    adapter: Arc<AnthropicAdapter<RecordingStore>>,
    client: reqwest::Client,
    handle: &'static str,
) -> RefreshFuture {
    Box::pin(async move { adapter.refresh_token(&client, handle).await })
}

pub fn independent_adapter_calls(
    count: usize,
    store: &RecordingStore,
    server: &ScriptedServer,
    singleflight: &Arc<UpstreamOAuthSingleflight>,
    client: &reqwest::Client,
    handle: &'static str,
) -> Vec<RefreshFuture> {
    (0..count)
        .map(|_| {
            refresh_future(
                adapter(store.clone(), server, Arc::clone(singleflight)),
                client.clone(),
                handle,
            )
        })
        .collect()
}

pub fn admit_all(calls: &mut [RefreshFuture]) {
    let mut context = Context::from_waker(Waker::noop());
    for call in calls {
        assert!(
            matches!(call.as_mut().poll(&mut context), Poll::Pending),
            "a newly admitted flight must wait for its helper-owned worker"
        );
    }
}

pub async fn complete_admitted(
    mut calls: Vec<RefreshFuture>,
) -> Vec<Result<String, RestAdapterError>> {
    admit_all(&mut calls);
    let tasks: Vec<_> = calls.into_iter().map(tokio::spawn).collect();
    tokio::time::timeout(Duration::from_secs(5), async move {
        let mut results = Vec::with_capacity(tasks.len());
        for task in tasks {
            results.push(task.await.unwrap());
        }
        results
    })
    .await
    .expect("admitted refresh flights must finish within five seconds")
}
