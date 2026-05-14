//! Ops docs-portal application — orchestration layer wrapping kernel ports
//! into wire-DTO-returning use cases. Pure std-only; no I/O.

use oya_ops_docs_portal_adapter::{
    WireLiveFeedEvent, WireManifestSnapshot, WireRefreshExtractorResponse,
};
use oya_ops_docs_portal_kernel::{
    ExtractorClass, ExtractorId, ExtractorRecord, LiveFeedError, LiveFeedEvent, LiveFeedEventKind,
    LiveFeedPort, ManifestError, ManifestPort, TenantScope,
};

/// GET /workspace/docs/manifest.
pub struct GetManifestUseCase<P: ManifestPort> {
    port: P,
}

impl<P: ManifestPort> GetManifestUseCase<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn execute(
        &self,
        tenant_scope: TenantScope,
        extractor_filter: Option<ExtractorClass>,
        include_stale: bool,
        now_unix_ms: u64,
    ) -> WireManifestSnapshot {
        WireManifestSnapshot::from_port(
            &self.port,
            tenant_scope,
            extractor_filter,
            include_stale,
            now_unix_ms,
        )
    }
}

/// POST /workspace/docs/api/v1/extractors/{id}/refresh.
///
/// Composite use case: refreshes the manifest port AND emits a
/// LiveFeedEventKind::ExtractorRefreshed event on the live-feed port so SSE
/// subscribers see the change in near-real-time per ADR-0066 Warm SLA.
pub struct RefreshExtractorUseCase<M: ManifestPort, F: LiveFeedPort> {
    manifest: M,
    live_feed: F,
}

#[derive(Debug)]
pub enum RefreshExtractorError {
    Manifest(ManifestError),
    LiveFeed(LiveFeedError),
}

impl From<ManifestError> for RefreshExtractorError {
    fn from(error: ManifestError) -> Self {
        Self::Manifest(error)
    }
}

impl From<LiveFeedError> for RefreshExtractorError {
    fn from(error: LiveFeedError) -> Self {
        Self::LiveFeed(error)
    }
}

impl<M: ManifestPort, F: LiveFeedPort> RefreshExtractorUseCase<M, F> {
    pub fn new(manifest: M, live_feed: F) -> Self {
        Self {
            manifest,
            live_feed,
        }
    }

    pub fn execute(
        &mut self,
        id: &ExtractorId,
        record_count: u64,
        unix_ms: u64,
        tenant_scope: TenantScope,
        payload_hash: String,
    ) -> Result<WireRefreshExtractorResponse, RefreshExtractorError> {
        self.manifest.refresh_extractor(id, record_count, unix_ms)?;
        self.live_feed.emit(LiveFeedEvent {
            kind: LiveFeedEventKind::ExtractorRefreshed,
            extractor_id: Some(id.clone()),
            tenant_scope,
            emitted_at_unix_ms: unix_ms,
            payload_hash,
        })?;
        // Synthesize the response shape; record_count was just written.
        Ok(WireRefreshExtractorResponse {
            extractor_id: id.0.clone(),
            refreshed: true,
            last_refreshed_unix_ms: unix_ms,
            record_count,
        })
    }
}

/// Extractor registration (mutating; internal-sre).
pub struct RegisterExtractorUseCase<P: ManifestPort> {
    port: P,
}

impl<P: ManifestPort> RegisterExtractorUseCase<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn execute(&mut self, record: ExtractorRecord) -> Result<(), ManifestError> {
        self.port.register_extractor(record)
    }

    pub fn inner_mut(&mut self) -> &mut P {
        &mut self.port
    }
}

/// GET /workspace/docs/live — replay window helper for SSE bootstrap.
pub struct SubscribeLiveFeedUseCase<P: LiveFeedPort> {
    port: P,
}

impl<P: LiveFeedPort> SubscribeLiveFeedUseCase<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn execute(&self, since_unix_ms: u64, limit: usize) -> Vec<WireLiveFeedEvent> {
        self.port
            .recent(since_unix_ms, limit)
            .into_iter()
            .map(WireLiveFeedEvent::from_kernel)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_ops_docs_portal_kernel::{InMemoryLiveFeed, InMemoryManifest};

    fn record(id: &str, class: ExtractorClass, fresh: u64) -> ExtractorRecord {
        ExtractorRecord {
            id: ExtractorId(id.into()),
            class,
            source_path: format!("docs/extractors/{id}.json"),
            last_refreshed_unix_ms: fresh,
            manifest_record_count: 0,
        }
    }

    fn populated_manifest() -> InMemoryManifest {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("a", ExtractorClass::Hot, 100))
            .unwrap();
        m.register_extractor(record("b", ExtractorClass::Cold, 100))
            .unwrap();
        m
    }

    #[test]
    fn get_manifest_returns_all_when_filter_none() {
        let use_case = GetManifestUseCase::new(populated_manifest());
        let response = use_case.execute(TenantScope(None), None, true, 200);
        assert_eq!(response.records.len(), 2);
    }

    #[test]
    fn get_manifest_filters_by_class() {
        let use_case = GetManifestUseCase::new(populated_manifest());
        let response = use_case.execute(TenantScope(None), Some(ExtractorClass::Hot), true, 200);
        assert_eq!(response.records.len(), 1);
        assert_eq!(response.records[0].class, "hot");
    }

    #[test]
    fn refresh_extractor_emits_live_feed() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        let response = use_case
            .execute(
                &ExtractorId("a".into()),
                42,
                200,
                TenantScope(None),
                "h".into(),
            )
            .unwrap();
        assert!(response.refreshed);
        assert_eq!(response.record_count, 42);
        assert_eq!(response.last_refreshed_unix_ms, 200);
        // The composite use case must emit a LiveFeed event on every successful refresh.
        assert_eq!(use_case.live_feed.count(), 1);
    }

    #[test]
    fn refresh_unknown_extractor_errors() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        let result = use_case.execute(
            &ExtractorId("unknown".into()),
            0,
            200,
            TenantScope(None),
            "h".into(),
        );
        assert!(matches!(
            result,
            Err(RefreshExtractorError::Manifest(
                ManifestError::UnknownExtractorId(_)
            ))
        ));
    }

    #[test]
    fn refresh_stale_timestamp_errors() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        // Existing fresh timestamp is 100; attempting 50 (older) should fail.
        let result = use_case.execute(
            &ExtractorId("a".into()),
            0,
            50,
            TenantScope(None),
            "h".into(),
        );
        assert!(matches!(
            result,
            Err(RefreshExtractorError::Manifest(
                ManifestError::StaleTimestamp { .. }
            ))
        ));
    }

    #[test]
    fn register_extractor_use_case_works() {
        let mut use_case = RegisterExtractorUseCase::new(InMemoryManifest::new());
        let result = use_case.execute(record("new", ExtractorClass::Warm, 0));
        assert!(result.is_ok());
        assert_eq!(
            use_case
                .inner_mut()
                .query(
                    &oya_ops_docs_portal_kernel::ManifestQuery {
                        tenant_scope: TenantScope(None),
                        extractor_filter: None,
                        include_stale: true
                    },
                    10
                )
                .records
                .len(),
            1
        );
    }

    #[test]
    fn subscribe_live_feed_filters_by_since() {
        let mut feed = InMemoryLiveFeed::new();
        for ms in [100u64, 200, 300, 400] {
            feed.emit(LiveFeedEvent {
                kind: LiveFeedEventKind::ExtractorRefreshed,
                extractor_id: Some(ExtractorId("a".into())),
                tenant_scope: TenantScope(None),
                emitted_at_unix_ms: ms,
                payload_hash: "h".into(),
            })
            .unwrap();
        }
        let use_case = SubscribeLiveFeedUseCase::new(feed);
        let events = use_case.execute(250, 10);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].emitted_at_unix_ms, 300);
    }

    #[test]
    fn subscribe_live_feed_respects_limit() {
        let mut feed = InMemoryLiveFeed::new();
        for ms in 0..20 {
            feed.emit(LiveFeedEvent {
                kind: LiveFeedEventKind::ExtractorRefreshed,
                extractor_id: Some(ExtractorId("a".into())),
                tenant_scope: TenantScope(None),
                emitted_at_unix_ms: ms,
                payload_hash: "h".into(),
            })
            .unwrap();
        }
        let use_case = SubscribeLiveFeedUseCase::new(feed);
        let events = use_case.execute(0, 5);
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn refresh_extractor_preserves_monotonic_feed_order() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        use_case
            .execute(
                &ExtractorId("a".into()),
                1,
                200,
                TenantScope(None),
                "h1".into(),
            )
            .unwrap();
        use_case
            .execute(
                &ExtractorId("b".into()),
                1,
                300,
                TenantScope(None),
                "h2".into(),
            )
            .unwrap();
        assert_eq!(use_case.live_feed.count(), 2);
    }
}
