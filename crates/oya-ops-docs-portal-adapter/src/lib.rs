//! Ops docs-portal adapter — projects kernel types onto the OpenAPI 3.2 wire
//! schema declared in `contracts/ops-docs.openapi.yaml`.
//!
//! Pure std-only adapter per ADR-0015. Mirrors `#/components/schemas/*` 1:1
//! so contract drift breaks the build.

use oya_ops_docs_portal_kernel::{
    ExtractorClass, ExtractorId, ExtractorRecord, LiveFeedEvent, LiveFeedEventKind,
    ManifestPort, ManifestQuery, ManifestSnapshot, TenantScope,
};

/// Wire shape mirroring `#/components/schemas/ExtractorRecord`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireExtractorRecord {
    pub id: String,
    pub class: String,
    pub source_path: String,
    pub last_refreshed_unix_ms: u64,
    pub manifest_record_count: u64,
    pub is_stale: bool,
}

impl WireExtractorRecord {
    pub fn from_kernel(record: &ExtractorRecord, now_unix_ms: u64) -> Self {
        Self {
            id: record.id.0.clone(),
            class: record.class.name().to_string(),
            source_path: record.source_path.clone(),
            last_refreshed_unix_ms: record.last_refreshed_unix_ms,
            manifest_record_count: record.manifest_record_count,
            is_stale: record.is_stale(now_unix_ms),
        }
    }
}

/// Wire shape mirroring `#/components/schemas/ManifestSnapshot`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireManifestSnapshot {
    pub records: Vec<WireExtractorRecord>,
    pub freshness_unix_ms: u64,
    pub stale_record_count: usize,
}

impl WireManifestSnapshot {
    pub fn from_kernel(snapshot: &ManifestSnapshot) -> Self {
        let records = snapshot
            .records
            .iter()
            .map(|r| WireExtractorRecord::from_kernel(r, snapshot.freshness_unix_ms))
            .collect();
        Self {
            records,
            freshness_unix_ms: snapshot.freshness_unix_ms,
            stale_record_count: snapshot.stale_record_count,
        }
    }

    /// Convenience: project the GET /workspace/docs/manifest response by
    /// applying the kernel query against a manifest port.
    pub fn from_port<P: ManifestPort>(
        port: &P,
        tenant_scope: TenantScope,
        extractor_filter: Option<ExtractorClass>,
        include_stale: bool,
        now_unix_ms: u64,
    ) -> Self {
        let snapshot = port.query(
            &ManifestQuery {
                tenant_scope,
                extractor_filter,
                include_stale,
            },
            now_unix_ms,
        );
        Self::from_kernel(&snapshot)
    }
}

/// Wire shape mirroring `#/components/schemas/LiveFeedEvent`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireLiveFeedEvent {
    pub kind: String,
    pub extractor_id: Option<String>,
    pub tenant_scope: Option<String>,
    pub emitted_at_unix_ms: u64,
    pub payload_hash: String,
}

impl WireLiveFeedEvent {
    pub fn from_kernel(event: &LiveFeedEvent) -> Self {
        Self {
            kind: kind_name(event.kind).to_string(),
            extractor_id: event.extractor_id.as_ref().map(|id| id.0.clone()),
            tenant_scope: event.tenant_scope.0.clone(),
            emitted_at_unix_ms: event.emitted_at_unix_ms,
            payload_hash: event.payload_hash.clone(),
        }
    }
}

fn kind_name(kind: LiveFeedEventKind) -> &'static str {
    match kind {
        LiveFeedEventKind::ExtractorRefreshed => "extractor-refreshed",
        LiveFeedEventKind::ManifestRowChanged => "manifest-row-changed",
        LiveFeedEventKind::DeadCodeDetected => "dead-code-detected",
        LiveFeedEventKind::LinkBroken => "link-broken",
    }
}

/// Wire shape mirroring `#/components/schemas/RefreshExtractorRequest`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireRefreshExtractorRequest {
    pub unix_ms: u64,
    pub force: bool,
}

/// Wire shape mirroring `#/components/schemas/RefreshExtractorResponse`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireRefreshExtractorResponse {
    pub extractor_id: String,
    pub refreshed: bool,
    pub last_refreshed_unix_ms: u64,
    pub record_count: u64,
}

impl WireRefreshExtractorResponse {
    pub fn from_record(record: &ExtractorRecord, refreshed: bool) -> Self {
        Self {
            extractor_id: record.id.0.clone(),
            refreshed,
            last_refreshed_unix_ms: record.last_refreshed_unix_ms,
            record_count: record.manifest_record_count,
        }
    }
}

/// Wire shape mirroring `#/components/schemas/CedarDenyResponse`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireCedarDenyResponse {
    pub error: String,
    pub principal_role: String,
    pub resource: String,
    pub cedar_fragment_denied_by: Option<String>,
}

impl WireCedarDenyResponse {
    pub fn for_extractor(
        id: &ExtractorId,
        principal_role: impl Into<String>,
        cedar_fragment_denied_by: Option<String>,
    ) -> Self {
        Self {
            error: "cedar-deny".into(),
            principal_role: principal_role.into(),
            resource: format!("extractor:{}", id.0),
            cedar_fragment_denied_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oya_ops_docs_portal_kernel::InMemoryManifest;

    fn record(id: &str, class: ExtractorClass, fresh: u64, count: u64) -> ExtractorRecord {
        ExtractorRecord {
            id: ExtractorId(id.into()),
            class,
            source_path: format!("docs/extractors/{id}.json"),
            last_refreshed_unix_ms: fresh,
            manifest_record_count: count,
        }
    }

    #[test]
    fn wire_extractor_record_serializes_classes() {
        let cases = [
            (ExtractorClass::Hot, "hot"),
            (ExtractorClass::Warm, "warm"),
            (ExtractorClass::Cold, "cold"),
        ];
        for (class, expected) in cases {
            let r = record("a", class, 0, 0);
            assert_eq!(WireExtractorRecord::from_kernel(&r, 0).class, expected);
        }
    }

    #[test]
    fn wire_extractor_record_propagates_staleness() {
        let r = record("a", ExtractorClass::Hot, 0, 0);
        let wire = WireExtractorRecord::from_kernel(&r, 600);
        assert!(wire.is_stale);
        let wire_fresh = WireExtractorRecord::from_kernel(&r, 100);
        assert!(!wire_fresh.is_stale);
    }

    #[test]
    fn wire_manifest_snapshot_from_port() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("a", ExtractorClass::Hot, 100, 5))
            .unwrap();
        m.register_extractor(record("b", ExtractorClass::Cold, 100, 10))
            .unwrap();
        let snap = WireManifestSnapshot::from_port(&m, TenantScope(None), None, true, 200);
        assert_eq!(snap.records.len(), 2);
        assert_eq!(snap.freshness_unix_ms, 200);
        assert_eq!(snap.records[0].id, "a");
        assert_eq!(snap.records[1].id, "b");
    }

    #[test]
    fn wire_manifest_snapshot_filter_by_class() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("a", ExtractorClass::Hot, 100, 5))
            .unwrap();
        m.register_extractor(record("b", ExtractorClass::Cold, 100, 10))
            .unwrap();
        let snap = WireManifestSnapshot::from_port(
            &m,
            TenantScope(None),
            Some(ExtractorClass::Hot),
            true,
            200,
        );
        assert_eq!(snap.records.len(), 1);
        assert_eq!(snap.records[0].class, "hot");
    }

    #[test]
    fn wire_live_feed_event_serializes_kinds() {
        let cases = [
            (LiveFeedEventKind::ExtractorRefreshed, "extractor-refreshed"),
            (LiveFeedEventKind::ManifestRowChanged, "manifest-row-changed"),
            (LiveFeedEventKind::DeadCodeDetected, "dead-code-detected"),
            (LiveFeedEventKind::LinkBroken, "link-broken"),
        ];
        for (kind, expected) in cases {
            let event = LiveFeedEvent {
                kind,
                extractor_id: Some(ExtractorId("a".into())),
                tenant_scope: TenantScope(None),
                emitted_at_unix_ms: 100,
                payload_hash: "deadbeef".into(),
            };
            assert_eq!(WireLiveFeedEvent::from_kernel(&event).kind, expected);
        }
    }

    #[test]
    fn wire_live_feed_event_tenant_scope_passthrough() {
        let event = LiveFeedEvent {
            kind: LiveFeedEventKind::ExtractorRefreshed,
            extractor_id: None,
            tenant_scope: TenantScope(Some("tenant-hash-abc".into())),
            emitted_at_unix_ms: 100,
            payload_hash: "h".into(),
        };
        let wire = WireLiveFeedEvent::from_kernel(&event);
        assert_eq!(wire.tenant_scope.as_deref(), Some("tenant-hash-abc"));
        assert!(wire.extractor_id.is_none());
    }

    #[test]
    fn wire_refresh_extractor_response_shape() {
        let r = record("a", ExtractorClass::Hot, 100, 7);
        let resp = WireRefreshExtractorResponse::from_record(&r, true);
        assert_eq!(resp.extractor_id, "a");
        assert!(resp.refreshed);
        assert_eq!(resp.last_refreshed_unix_ms, 100);
        assert_eq!(resp.record_count, 7);
    }

    #[test]
    fn wire_cedar_deny_for_extractor() {
        let resp = WireCedarDenyResponse::for_extractor(
            &ExtractorId("watch".into()),
            "tenant-user",
            Some("ops-manifest-tenant-filter".into()),
        );
        assert_eq!(resp.error, "cedar-deny");
        assert_eq!(resp.principal_role, "tenant-user");
        assert_eq!(resp.resource, "extractor:watch");
        assert_eq!(
            resp.cedar_fragment_denied_by.as_deref(),
            Some("ops-manifest-tenant-filter")
        );
    }

    #[test]
    fn wire_refresh_extractor_request_shape() {
        let req = WireRefreshExtractorRequest {
            unix_ms: 12345,
            force: false,
        };
        assert_eq!(req.unix_ms, 12345);
        assert!(!req.force);
    }

    #[test]
    fn wire_manifest_snapshot_propagates_stale_count() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("hot-stale", ExtractorClass::Hot, 0, 0))
            .unwrap();
        // include_stale=true means stale rows appear, and stale_record_count is N.
        let snap = WireManifestSnapshot::from_port(&m, TenantScope(None), None, true, 10_000);
        assert_eq!(snap.records.len(), 1);
        assert_eq!(snap.stale_record_count, 1);
    }
}
