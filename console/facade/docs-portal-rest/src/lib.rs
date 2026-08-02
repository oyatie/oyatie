//! Framework-free REST boundary for the ops docs-portal BC.
//!
//! Hyper service bindings deferred to the runtime composition root per the
//! LTS-dependency-enforcement directive. Hyper is the canonical workspace HTTP
//! backbone (user-issued 2026-05-14). Owns OpenAPI-aligned request /
//! response shapes + handler functions today.
//!
//! Routes here MUST stay 1:1 with paths in `contracts/ops-docs-v1.openapi.yaml`.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use console_docs_portal_adapter::{
    WireLiveFeedEvent, WireManifestSnapshot, WireRefreshExtractorOutcome,
    WireRefreshExtractorResponse,
};
use console_docs_portal_kernel::{
    ExtractorClass, ExtractorId, LiveFeedError, LiveFeedPort, ManifestError, ManifestPort,
    TenantScope,
};
use console_docs_portal_usecase::{
    GetManifestUseCase, RefreshExtractorOutcome, RefreshExtractorUseCase, SubscribeLiveFeedUseCase,
};

pub const GET_MANIFEST_ROUTE: &str = "/workspace/docs/manifest";
pub const SUBSCRIBE_LIVE_FEED_ROUTE: &str = "/workspace/docs/live";
pub const REFRESH_EXTRACTOR_ROUTE: &str =
    "/workspace/docs/api/v1/extractors/{extractor_id}/refresh";

pub const GET_MANIFEST_METHOD: &str = "GET";
pub const SUBSCRIBE_LIVE_FEED_METHOD: &str = "GET";
pub const REFRESH_EXTRACTOR_METHOD: &str = "POST";

/// Request shape for GET /workspace/docs/manifest.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct GetManifestRequest {
    pub class: Option<ExtractorClass>,
    pub include_stale: bool,
    pub now_unix_ms: u64,
}

/// Request shape for GET /workspace/docs/live.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SubscribeLiveFeedRequest {
    pub since_unix_ms: u64,
    pub limit: usize,
}

/// Request shape for POST /workspace/docs/api/v1/extractors/{id}/refresh.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshExtractorRequest {
    pub extractor_id: ExtractorId,
    pub unix_ms: u64,
    pub record_count: u64,
    pub payload_hash: String,
}

/// REST handler: GET /workspace/docs/manifest.
pub fn get_manifest<P: ManifestPort>(
    port: P,
    request: GetManifestRequest,
) -> Result<WireManifestSnapshot, ManifestError> {
    let snapshot = GetManifestUseCase::new(port).execute(
        TenantScope(None),
        request.class,
        request.include_stale,
        request.now_unix_ms,
    )?;
    Ok(WireManifestSnapshot::from_kernel(&snapshot))
}

/// REST handler: GET /workspace/docs/live.
pub fn subscribe_live_feed<P: LiveFeedPort>(
    port: P,
    request: SubscribeLiveFeedRequest,
) -> Result<Vec<WireLiveFeedEvent>, LiveFeedError> {
    Ok(SubscribeLiveFeedUseCase::new(port)
        .execute(TenantScope(None), request.since_unix_ms, request.limit)?
        .iter()
        .map(WireLiveFeedEvent::from_kernel)
        .collect())
}

/// REST handler: POST /workspace/docs/api/v1/extractors/{id}/refresh.
pub fn refresh_extractor<M: ManifestPort, F: LiveFeedPort>(
    manifest: M,
    live_feed: F,
    request: RefreshExtractorRequest,
) -> Result<WireRefreshExtractorResponse, ManifestError> {
    let outcome = RefreshExtractorUseCase::new(manifest, live_feed).execute(
        &request.extractor_id,
        request.record_count,
        request.unix_ms,
        request.payload_hash,
    )?;
    match outcome {
        RefreshExtractorOutcome::AppliedAndEventRecorded { applied } => {
            Ok(WireRefreshExtractorResponse::from_applied(
                applied.extractor_id,
                applied.last_refreshed_unix_ms,
                applied.record_count,
                WireRefreshExtractorOutcome::AppliedAndEventRecorded,
            ))
        }
        RefreshExtractorOutcome::AppliedButEventRecordingFailed { applied, .. } => {
            Ok(WireRefreshExtractorResponse::from_applied(
                applied.extractor_id,
                applied.last_refreshed_unix_ms,
                applied.record_count,
                WireRefreshExtractorOutcome::AppliedButEventRecordingFailed,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use console_docs_portal_kernel::{
        ExtractorRecord, InMemoryLiveFeed, InMemoryManifest, ManifestError,
    };

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
    fn routes_match_openapi_paths() {
        assert_eq!(GET_MANIFEST_ROUTE, "/workspace/docs/manifest");
        assert_eq!(SUBSCRIBE_LIVE_FEED_ROUTE, "/workspace/docs/live");
        assert_eq!(
            REFRESH_EXTRACTOR_ROUTE,
            "/workspace/docs/api/v1/extractors/{extractor_id}/refresh"
        );
    }

    #[test]
    fn http_methods_match_openapi() {
        assert_eq!(GET_MANIFEST_METHOD, "GET");
        assert_eq!(SUBSCRIBE_LIVE_FEED_METHOD, "GET");
        assert_eq!(REFRESH_EXTRACTOR_METHOD, "POST");
    }

    #[test]
    fn get_manifest_returns_all_records() {
        let response = get_manifest(
            populated_manifest(),
            GetManifestRequest {
                class: None,
                include_stale: true,
                now_unix_ms: 200,
            },
        )
        .unwrap();
        assert_eq!(response.records.len(), 2);
    }

    #[test]
    fn get_manifest_filters_by_class() {
        let response = get_manifest(
            populated_manifest(),
            GetManifestRequest {
                class: Some(ExtractorClass::Hot),
                include_stale: true,
                now_unix_ms: 200,
            },
        )
        .unwrap();
        assert_eq!(response.records.len(), 1);
        assert_eq!(response.records[0].class, "hot");
    }

    #[test]
    fn subscribe_live_feed_empty_when_no_events() {
        let events = subscribe_live_feed(
            InMemoryLiveFeed::new(),
            SubscribeLiveFeedRequest {
                since_unix_ms: 0,
                limit: 10,
            },
        )
        .unwrap();
        assert!(events.is_empty());
    }

    #[derive(Default)]
    struct RejectingLiveFeed;

    impl LiveFeedPort for RejectingLiveFeed {
        fn emit(
            &mut self,
            _event: console_docs_portal_kernel::LiveFeedEvent,
        ) -> Result<(), console_docs_portal_kernel::LiveFeedError> {
            Err(
                console_docs_portal_kernel::LiveFeedError::NonMonotonicTimestamp {
                    prior: 300,
                    attempted: 200,
                },
            )
        }

        fn recent(
            &self,
            _tenant_scope: &TenantScope,
            _since_unix_ms: u64,
            _limit: usize,
        ) -> Result<
            Vec<&console_docs_portal_kernel::LiveFeedEvent>,
            console_docs_portal_kernel::LiveFeedError,
        > {
            Ok(Vec::new())
        }

        fn count(&self) -> usize {
            0
        }
    }

    #[test]
    fn feed_recording_failure_is_normal_typed_applied_response() {
        let response = refresh_extractor(
            populated_manifest(),
            RejectingLiveFeed,
            RefreshExtractorRequest {
                extractor_id: ExtractorId("a".into()),
                unix_ms: 200,
                record_count: 42,
                payload_hash: "h".into(),
            },
        )
        .unwrap();
        assert_eq!(
            response.outcome.name(),
            "applied-but-event-recording-failed"
        );
        assert_eq!(response.record_count, 42);
    }

    #[test]
    fn refresh_extractor_emits_live_feed_event() {
        let manifest = populated_manifest();
        let live_feed = InMemoryLiveFeed::new();
        let response = refresh_extractor(
            manifest,
            live_feed,
            RefreshExtractorRequest {
                extractor_id: ExtractorId("a".into()),
                unix_ms: 200,
                record_count: 42,
                payload_hash: "h".into(),
            },
        )
        .unwrap();
        assert_eq!(response.outcome.name(), "applied-and-event-recorded");
        assert_eq!(response.record_count, 42);
        assert_eq!(response.last_refreshed_unix_ms, 200);
    }

    #[test]
    fn refresh_extractor_unknown_id_errors() {
        let result = refresh_extractor(
            populated_manifest(),
            InMemoryLiveFeed::new(),
            RefreshExtractorRequest {
                extractor_id: ExtractorId("unknown".into()),
                unix_ms: 200,
                record_count: 0,
                payload_hash: "h".into(),
            },
        );
        assert!(matches!(result, Err(ManifestError::UnknownExtractorId(_))));
    }

    #[test]
    fn refresh_extractor_stale_timestamp_errors() {
        let result = refresh_extractor(
            populated_manifest(),
            InMemoryLiveFeed::new(),
            RefreshExtractorRequest {
                extractor_id: ExtractorId("a".into()),
                unix_ms: 50,
                record_count: 0,
                payload_hash: "h".into(),
            },
        );
        assert!(matches!(result, Err(ManifestError::StaleTimestamp { .. })));
    }

    #[test]
    fn subscribe_live_feed_respects_limit() {
        // Author 5 events into the feed; request limit=3; expect 3 back.
        let mut feed = InMemoryLiveFeed::new();
        for ms in [10u64, 20, 30, 40, 50] {
            feed.emit(console_docs_portal_kernel::LiveFeedEvent {
                kind: console_docs_portal_kernel::LiveFeedEventKind::ExtractorRefreshed,
                extractor_id: Some(ExtractorId("a".into())),
                tenant_scope: TenantScope(None),
                emitted_at_unix_ms: ms,
                payload_hash: "h".into(),
            })
            .unwrap();
        }
        let events = subscribe_live_feed(
            feed,
            SubscribeLiveFeedRequest {
                since_unix_ms: 0,
                limit: 3,
            },
        )
        .unwrap();
        assert_eq!(events.len(), 3);
    }
}
