//! Ops docs-portal adapter — projects kernel types onto the OpenAPI 3.2 wire
//! schema declared in `contracts/ops-docs-v1.openapi.yaml`.
//!
//! Pure std-only adapter per ADR-0015. Mirrors `#/components/schemas/*` 1:1
//! so contract drift breaks the build.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use console_docs_portal_kernel::{
    ExtractorClass, ExtractorId, ExtractorRecord, LiveFeedEvent, LiveFeedEventKind,
    ManifestSnapshot,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireRefreshExtractorOutcome {
    AppliedAndEventRecorded,
    AppliedButEventRecordingFailed,
}

impl WireRefreshExtractorOutcome {
    pub fn name(self) -> &'static str {
        match self {
            Self::AppliedAndEventRecorded => "applied-and-event-recorded",
            Self::AppliedButEventRecordingFailed => "applied-but-event-recording-failed",
        }
    }
}

/// Wire shape mirroring `#/components/schemas/RefreshExtractorResponse`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireRefreshExtractorResponse {
    pub extractor_id: String,
    pub outcome: WireRefreshExtractorOutcome,
    pub last_refreshed_unix_ms: u64,
    pub record_count: u64,
}

impl WireRefreshExtractorResponse {
    pub fn from_applied(
        extractor_id: ExtractorId,
        last_refreshed_unix_ms: u64,
        record_count: u64,
        outcome: WireRefreshExtractorOutcome,
    ) -> Self {
        Self {
            extractor_id: extractor_id.0,
            outcome,
            last_refreshed_unix_ms,
            record_count,
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
    use console_docs_portal_kernel::TenantScope;

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
    fn wire_live_feed_event_serializes_kinds() {
        let cases = [
            (LiveFeedEventKind::ExtractorRefreshed, "extractor-refreshed"),
            (
                LiveFeedEventKind::ManifestRowChanged,
                "manifest-row-changed",
            ),
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
    fn both_refresh_outcomes_map_without_error_detail_leak() {
        let recorded = WireRefreshExtractorResponse::from_applied(
            ExtractorId("a".into()),
            100,
            7,
            WireRefreshExtractorOutcome::AppliedAndEventRecorded,
        );
        let failed = WireRefreshExtractorResponse::from_applied(
            ExtractorId("a".into()),
            100,
            7,
            WireRefreshExtractorOutcome::AppliedButEventRecordingFailed,
        );
        assert_eq!(recorded.outcome.name(), "applied-and-event-recorded");
        assert_eq!(failed.outcome.name(), "applied-but-event-recording-failed");
        assert_eq!(recorded.extractor_id, "a");
        assert_eq!(failed.record_count, 7);
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
    fn wire_refresh_extractor_request_has_only_governed_input() {
        let req = WireRefreshExtractorRequest { unix_ms: 12345 };
        assert_eq!(req.unix_ms, 12345);
    }

    #[test]
    fn exported_openapi_matches_refresh_wire_contract() {
        let contract = std::fs::read_to_string(env!("OPS_DOCS_OPENAPI"))
            .expect("exported OpenAPI contract must be readable");
        for required in [
            "required: [extractor_id, outcome, last_refreshed_unix_ms, record_count]",
            "enum: [applied-and-event-recorded, applied-but-event-recording-failed]",
            "\"200\":",
        ] {
            assert!(
                contract.contains(required),
                "missing contract fragment: {required}"
            );
        }
        for unsupported in ["name: tenant", "force:", "refreshed:", "\"202\":"] {
            assert!(
                !contract.contains(unsupported),
                "unsupported contract fragment remains: {unsupported}"
            );
        }
    }
}
