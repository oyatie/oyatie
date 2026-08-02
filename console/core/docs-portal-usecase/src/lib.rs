//! Ops docs-portal application — use-case orchestration layer.
//!
//! Per ADR-0056 this crate depends only inward on the kernel. REST/OpenAPI
//! wire projection stays in the presentation/adapter boundary, not here.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use console_docs_portal_kernel::{
    ExtractorClass, ExtractorId, ExtractorRecord, LiveFeedError, LiveFeedEvent, LiveFeedEventKind,
    LiveFeedPort, ManifestError, ManifestPort, ManifestQuery, ManifestSnapshot, TenantScope,
};

/// Manifest mutation details returned after a successful refresh.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedRefresh {
    pub extractor_id: ExtractorId,   // data_class: INTERNAL_ONLY
    pub last_refreshed_unix_ms: u64, // data_class: INTERNAL_ONLY
    pub record_count: u64,           // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefreshExtractorOutcome {
    AppliedAndEventRecorded {
        applied: AppliedRefresh,
    },
    AppliedButEventRecordingFailed {
        applied: AppliedRefresh,
        cause: LiveFeedError,
    },
}

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
    ) -> Result<ManifestSnapshot, ManifestError> {
        self.port.query(
            &ManifestQuery {
                tenant_scope,
                extractor_filter,
                include_stale,
            },
            now_unix_ms,
        )
    }
}

/// POST /workspace/docs/api/v1/extractors/{id}/refresh.
///
/// Composite use case: refreshes the manifest port and records an
/// `LiveFeedEventKind::ExtractorRefreshed` event through the live-feed port.
pub struct RefreshExtractorUseCase<M: ManifestPort, F: LiveFeedPort> {
    manifest: M,
    live_feed: F,
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
        payload_hash: String,
    ) -> Result<RefreshExtractorOutcome, ManifestError> {
        self.manifest.refresh_extractor(id, record_count, unix_ms)?;
        let applied = AppliedRefresh {
            extractor_id: id.clone(),
            last_refreshed_unix_ms: unix_ms,
            record_count,
        };
        let event = LiveFeedEvent {
            kind: LiveFeedEventKind::ExtractorRefreshed,
            extractor_id: Some(id.clone()),
            tenant_scope: TenantScope(None),
            emitted_at_unix_ms: unix_ms,
            payload_hash,
        };
        match self.live_feed.emit(event) {
            Ok(()) => Ok(RefreshExtractorOutcome::AppliedAndEventRecorded { applied }),
            Err(cause) => {
                Ok(RefreshExtractorOutcome::AppliedButEventRecordingFailed { applied, cause })
            }
        }
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

/// GET /workspace/docs/live — fallible replay-window query.
pub struct SubscribeLiveFeedUseCase<P: LiveFeedPort> {
    port: P,
}

impl<P: LiveFeedPort> SubscribeLiveFeedUseCase<P> {
    pub fn new(port: P) -> Self {
        Self { port }
    }

    pub fn execute(
        &self,
        tenant_scope: TenantScope,
        since_unix_ms: u64,
        limit: usize,
    ) -> Result<Vec<LiveFeedEvent>, LiveFeedError> {
        Ok(self
            .port
            .recent(&tenant_scope, since_unix_ms, limit)?
            .into_iter()
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console_docs_portal_kernel::{InMemoryLiveFeed, InMemoryManifest};

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
        let response = use_case
            .execute(TenantScope(None), None, true, 200)
            .unwrap();
        assert_eq!(response.records.len(), 2);
    }

    #[test]
    fn get_manifest_filters_by_class() {
        let use_case = GetManifestUseCase::new(populated_manifest());
        let response = use_case
            .execute(TenantScope(None), Some(ExtractorClass::Hot), true, 200)
            .unwrap();
        assert_eq!(response.records.len(), 1);
        assert_eq!(response.records[0].class, ExtractorClass::Hot);
    }

    #[derive(Default)]
    struct RejectingLiveFeed;

    impl LiveFeedPort for RejectingLiveFeed {
        fn emit(&mut self, _event: LiveFeedEvent) -> Result<(), LiveFeedError> {
            Err(LiveFeedError::NonMonotonicTimestamp {
                prior: 300,
                attempted: 200,
            })
        }

        fn recent(
            &self,
            _tenant_scope: &TenantScope,
            _since_unix_ms: u64,
            _limit: usize,
        ) -> Result<Vec<&LiveFeedEvent>, LiveFeedError> {
            Ok(Vec::new())
        }

        fn count(&self) -> usize {
            0
        }
    }

    #[test]
    fn feed_failure_after_mutation_is_typed_applied_outcome() {
        let mut use_case = RefreshExtractorUseCase::new(populated_manifest(), RejectingLiveFeed);
        let outcome = use_case
            .execute(&ExtractorId("a".into()), 42, 200, "h".into())
            .unwrap();
        assert!(matches!(
            outcome,
            RefreshExtractorOutcome::AppliedButEventRecordingFailed {
                applied: AppliedRefresh {
                    extractor_id: ExtractorId(ref id),
                    last_refreshed_unix_ms: 200,
                    record_count: 42,
                },
                cause: LiveFeedError::NonMonotonicTimestamp { .. },
            } if id == "a"
        ));
        let snapshot = use_case
            .manifest
            .query(
                &ManifestQuery {
                    tenant_scope: TenantScope(None),
                    extractor_filter: None,
                    include_stale: true,
                },
                200,
            )
            .unwrap();
        assert_eq!(snapshot.records[0].last_refreshed_unix_ms, 200);
        assert_eq!(snapshot.records[0].manifest_record_count, 42);
    }

    #[test]
    fn refresh_extractor_emits_live_feed() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        let response = use_case
            .execute(&ExtractorId("a".into()), 42, 200, "h".into())
            .unwrap();
        assert!(matches!(
            response,
            RefreshExtractorOutcome::AppliedAndEventRecorded {
                applied: AppliedRefresh {
                    extractor_id: ExtractorId(ref id),
                    last_refreshed_unix_ms: 200,
                    record_count: 42,
                }
            } if id == "a"
        ));
        assert_eq!(use_case.live_feed.count(), 1);
    }

    #[test]
    fn refresh_unknown_extractor_errors() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        let result = use_case.execute(&ExtractorId("unknown".into()), 0, 200, "h".into());
        assert!(matches!(result, Err(ManifestError::UnknownExtractorId(_))));
        assert_eq!(use_case.live_feed.count(), 0);
    }

    #[test]
    fn refresh_stale_timestamp_errors() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        let result = use_case.execute(&ExtractorId("a".into()), 0, 50, "h".into());
        assert!(matches!(result, Err(ManifestError::StaleTimestamp { .. })));
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
                    &ManifestQuery {
                        tenant_scope: TenantScope(None),
                        extractor_filter: None,
                        include_stale: true
                    },
                    10
                )
                .unwrap()
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
        let events = use_case.execute(TenantScope(None), 250, 10).unwrap();
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
        let events = use_case.execute(TenantScope(None), 0, 5).unwrap();
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn refresh_extractor_preserves_monotonic_feed_order() {
        let mut use_case =
            RefreshExtractorUseCase::new(populated_manifest(), InMemoryLiveFeed::new());
        use_case
            .execute(&ExtractorId("a".into()), 1, 200, "h1".into())
            .unwrap();
        use_case
            .execute(&ExtractorId("b".into()), 1, 300, "h2".into())
            .unwrap();
        assert_eq!(use_case.live_feed.count(), 2);
    }
}
