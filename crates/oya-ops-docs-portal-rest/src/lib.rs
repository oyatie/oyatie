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

use oya_ops_docs_portal_adapter::{
    WireLiveFeedEvent, WireManifestSnapshot, WireRefreshExtractorResponse,
};
use oya_ops_docs_portal_usecase::{
    GetManifestUseCase, RefreshExtractorError, RefreshExtractorUseCase, SubscribeLiveFeedUseCase,
};
use oya_ops_docs_portal_kernel::{
    ExtractorClass, ExtractorId, LiveFeedPort, ManifestPort, TenantScope,
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
    pub tenant_scope: TenantScope,
    pub class: Option<ExtractorClass>,
    pub include_stale: bool,
    pub now_unix_ms: u64,
}

/// Request shape for GET /workspace/docs/live.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SubscribeLiveFeedRequest {
    pub tenant_scope: TenantScope,
    pub since_unix_ms: u64,
    pub limit: usize,
}

/// Request shape for POST /workspace/docs/api/v1/extractors/{id}/refresh.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RefreshExtractorRequest {
    pub extractor_id: ExtractorId,
    pub unix_ms: u64,
    pub record_count: u64,
    pub tenant_scope: TenantScope,
    pub payload_hash: String,
    pub force: bool,
}

/// REST handler: GET /workspace/docs/manifest.
pub fn get_manifest<P: ManifestPort>(port: P, request: GetManifestRequest) -> WireManifestSnapshot {
    let snapshot = GetManifestUseCase::new(port).execute(
        request.tenant_scope,
        request.class,
        request.include_stale,
        request.now_unix_ms,
    );
    WireManifestSnapshot::from_kernel(&snapshot)
}

/// REST handler: GET /workspace/docs/live.
pub fn subscribe_live_feed<P: LiveFeedPort>(
    port: P,
    request: SubscribeLiveFeedRequest,
) -> Vec<WireLiveFeedEvent> {
    SubscribeLiveFeedUseCase::new(port)
        .execute(request.since_unix_ms, request.limit)
        .iter()
        .map(WireLiveFeedEvent::from_kernel)
        .collect()
}

/// REST handler: POST /workspace/docs/api/v1/extractors/{id}/refresh.
pub fn refresh_extractor<M: ManifestPort, F: LiveFeedPort>(
    manifest: M,
    live_feed: F,
    request: RefreshExtractorRequest,
) -> Result<WireRefreshExtractorResponse, RefreshExtractorError> {
    let result = RefreshExtractorUseCase::new(manifest, live_feed).execute(
        &request.extractor_id,
        request.record_count,
        request.unix_ms,
        request.tenant_scope,
        request.payload_hash,
    )?;
    Ok(WireRefreshExtractorResponse {
        extractor_id: result.extractor_id.0,
        refreshed: result.refreshed,
        last_refreshed_unix_ms: result.last_refreshed_unix_ms,
        record_count: result.record_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_ops_docs_portal_kernel::{
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
                tenant_scope: TenantScope(None),
                class: None,
                include_stale: true,
                now_unix_ms: 200,
            },
        );
        assert_eq!(response.records.len(), 2);
    }

    #[test]
    fn get_manifest_filters_by_class() {
        let response = get_manifest(
            populated_manifest(),
            GetManifestRequest {
                tenant_scope: TenantScope(None),
                class: Some(ExtractorClass::Hot),
                include_stale: true,
                now_unix_ms: 200,
            },
        );
        assert_eq!(response.records.len(), 1);
        assert_eq!(response.records[0].class, "hot");
    }

    #[test]
    fn subscribe_live_feed_empty_when_no_events() {
        let events = subscribe_live_feed(
            InMemoryLiveFeed::new(),
            SubscribeLiveFeedRequest {
                tenant_scope: TenantScope(None),
                since_unix_ms: 0,
                limit: 10,
            },
        );
        assert!(events.is_empty());
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
                tenant_scope: TenantScope(None),
                payload_hash: "h".into(),
                force: false,
            },
        )
        .unwrap();
        assert!(response.refreshed);
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
                tenant_scope: TenantScope(None),
                payload_hash: "h".into(),
                force: false,
            },
        );
        assert!(matches!(
            result,
            Err(RefreshExtractorError::Manifest(
                ManifestError::UnknownExtractorId(_)
            ))
        ));
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
                tenant_scope: TenantScope(None),
                payload_hash: "h".into(),
                force: false,
            },
        );
        assert!(matches!(
            result,
            Err(RefreshExtractorError::Manifest(
                ManifestError::StaleTimestamp { .. }
            ))
        ));
    }

    #[test]
    fn subscribe_live_feed_respects_limit() {
        // Author 5 events into the feed; request limit=3; expect 3 back.
        let mut feed = InMemoryLiveFeed::new();
        for ms in [10u64, 20, 30, 40, 50] {
            feed.emit(oya_ops_docs_portal_kernel::LiveFeedEvent {
                kind: oya_ops_docs_portal_kernel::LiveFeedEventKind::ExtractorRefreshed,
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
                tenant_scope: TenantScope(None),
                since_unix_ms: 0,
                limit: 3,
            },
        );
        assert_eq!(events.len(), 3);
    }
}
