//! Ops docs-portal kernel — port traits + types for the ops/docs-portal BC
//! per ralplan-docs-portal v7 + ADR-0066 live code-introspection docs portal.
//!
//! Defines the hot/warm/cold extractor classes (ADR-0066 perf SLAs) +
//! fallible ManifestPort and LiveFeedPort boundaries.
//!
//! Pure std-only kernel layer per ADR-0015: no outbound I/O, no framework deps.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;

/// Extractor freshness class per ADR-0066 §5. Each class declares an SLA budget.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ExtractorClass {
    /// Read budget ≤500ms.
    Hot,
    /// Refresh budget ≤2s.
    Warm,
    /// Background-refreshed; budget ≤10min scheduled; cold cache.
    Cold,
}

impl ExtractorClass {
    pub fn name(self) -> &'static str {
        match self {
            ExtractorClass::Hot => "hot",
            ExtractorClass::Warm => "warm",
            ExtractorClass::Cold => "cold",
        }
    }

    /// SLA budget in milliseconds for this class per ADR-0066 + ADR-0067 §5.
    pub fn sla_budget_ms(self) -> u64 {
        match self {
            ExtractorClass::Hot => 500,
            ExtractorClass::Warm => 2_000,
            ExtractorClass::Cold => 600_000, // 10 min
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExtractorId(pub String); // data_class: INTERNAL_ONLY

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtractorRecord {
    pub id: ExtractorId,             // data_class: INTERNAL_ONLY
    pub class: ExtractorClass,       // data_class: INTERNAL_ONLY
    pub source_path: String,         // data_class: INTERNAL_ONLY
    pub last_refreshed_unix_ms: u64, // data_class: INTERNAL_ONLY
    pub manifest_record_count: u64,  // data_class: INTERNAL_ONLY
}

impl ExtractorRecord {
    pub fn is_stale(&self, now_unix_ms: u64) -> bool {
        let age_ms = now_unix_ms.saturating_sub(self.last_refreshed_unix_ms);
        age_ms > self.class.sla_budget_ms()
    }
}

/// Scope marker at the read boundary.
/// `None` is the supported internal scope; `Some` is refused as unsupported.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TenantScope(pub Option<String>); // data_class: INTERNAL_ONLY (SHA-256 of tenant id; never raw)

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TenantScopeRefusal {
    Unsupported(TenantScope),
}

/// Manifest query: read-side projection of the docs portal state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestQuery {
    pub tenant_scope: TenantScope, // data_class: INTERNAL_ONLY
    pub extractor_filter: Option<ExtractorClass>, // data_class: INTERNAL_ONLY
    pub include_stale: bool,       // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestSnapshot {
    pub records: Vec<ExtractorRecord>, // data_class: INTERNAL_ONLY
    pub freshness_unix_ms: u64,        // data_class: INTERNAL_ONLY
    pub stale_record_count: usize,     // data_class: INTERNAL_ONLY
}

pub trait ManifestPort {
    fn register_extractor(&mut self, record: ExtractorRecord) -> Result<(), ManifestError>;
    fn refresh_extractor(
        &mut self,
        id: &ExtractorId,
        record_count: u64,
        unix_ms: u64,
    ) -> Result<(), ManifestError>;
    fn query(
        &self,
        query: &ManifestQuery,
        now_unix_ms: u64,
    ) -> Result<ManifestSnapshot, ManifestError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ManifestError {
    DuplicateExtractorId(ExtractorId),
    UnknownExtractorId(ExtractorId),
    TenantScope(TenantScopeRefusal),
    StaleTimestamp {
        id: ExtractorId,
        prior: u64,
        attempted: u64,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryManifest {
    by_id: BTreeMap<ExtractorId, ExtractorRecord>, // data_class: INTERNAL_ONLY
}

impl InMemoryManifest {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ManifestPort for InMemoryManifest {
    fn register_extractor(&mut self, record: ExtractorRecord) -> Result<(), ManifestError> {
        if self.by_id.contains_key(&record.id) {
            return Err(ManifestError::DuplicateExtractorId(record.id));
        }
        self.by_id.insert(record.id.clone(), record);
        Ok(())
    }

    fn refresh_extractor(
        &mut self,
        id: &ExtractorId,
        record_count: u64,
        unix_ms: u64,
    ) -> Result<(), ManifestError> {
        let entry = self
            .by_id
            .get_mut(id)
            .ok_or_else(|| ManifestError::UnknownExtractorId(id.clone()))?;
        if unix_ms < entry.last_refreshed_unix_ms {
            return Err(ManifestError::StaleTimestamp {
                id: id.clone(),
                prior: entry.last_refreshed_unix_ms,
                attempted: unix_ms,
            });
        }
        entry.last_refreshed_unix_ms = unix_ms;
        entry.manifest_record_count = record_count;
        Ok(())
    }

    fn query(
        &self,
        query: &ManifestQuery,
        now_unix_ms: u64,
    ) -> Result<ManifestSnapshot, ManifestError> {
        if query.tenant_scope.0.is_some() {
            return Err(ManifestError::TenantScope(TenantScopeRefusal::Unsupported(
                query.tenant_scope.clone(),
            )));
        }
        let mut records: Vec<ExtractorRecord> = self
            .by_id
            .values()
            .filter(|record| {
                query
                    .extractor_filter
                    .map(|class| record.class == class)
                    .unwrap_or(true)
            })
            .filter(|record| query.include_stale || !record.is_stale(now_unix_ms))
            .cloned()
            .collect();
        records.sort_by(|a, b| a.id.cmp(&b.id));
        let stale_record_count = records.iter().filter(|r| r.is_stale(now_unix_ms)).count();
        Ok(ManifestSnapshot {
            records,
            freshness_unix_ms: now_unix_ms,
            stale_record_count,
        })
    }
}

/// Live-feed event class per ADR-0066 §3.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LiveFeedEventKind {
    ExtractorRefreshed,
    ManifestRowChanged,
    DeadCodeDetected,
    LinkBroken,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveFeedEvent {
    pub kind: LiveFeedEventKind,           // data_class: INTERNAL_ONLY
    pub extractor_id: Option<ExtractorId>, // data_class: INTERNAL_ONLY
    pub tenant_scope: TenantScope,         // data_class: INTERNAL_ONLY
    pub emitted_at_unix_ms: u64,           // data_class: INTERNAL_ONLY
    pub payload_hash: String,              // data_class: INTERNAL_ONLY (SHA-256; never raw)
}

pub trait LiveFeedPort {
    fn emit(&mut self, event: LiveFeedEvent) -> Result<(), LiveFeedError>;
    fn recent(
        &self,
        tenant_scope: &TenantScope,
        since_unix_ms: u64,
        limit: usize,
    ) -> Result<Vec<&LiveFeedEvent>, LiveFeedError>;
    fn count(&self) -> usize;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LiveFeedError {
    TenantScope(TenantScopeRefusal),
    NonMonotonicTimestamp { prior: u64, attempted: u64 },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InMemoryLiveFeed {
    events: Vec<LiveFeedEvent>,  // data_class: INTERNAL_ONLY (chronological)
    high_watermark_unix_ms: u64, // data_class: INTERNAL_ONLY
}

impl InMemoryLiveFeed {
    pub fn new() -> Self {
        Self::default()
    }
}

impl LiveFeedPort for InMemoryLiveFeed {
    fn emit(&mut self, event: LiveFeedEvent) -> Result<(), LiveFeedError> {
        if event.emitted_at_unix_ms < self.high_watermark_unix_ms {
            return Err(LiveFeedError::NonMonotonicTimestamp {
                prior: self.high_watermark_unix_ms,
                attempted: event.emitted_at_unix_ms,
            });
        }
        self.high_watermark_unix_ms = event.emitted_at_unix_ms;
        self.events.push(event);
        Ok(())
    }

    fn recent(
        &self,
        tenant_scope: &TenantScope,
        since_unix_ms: u64,
        limit: usize,
    ) -> Result<Vec<&LiveFeedEvent>, LiveFeedError> {
        if tenant_scope.0.is_some() {
            return Err(LiveFeedError::TenantScope(TenantScopeRefusal::Unsupported(
                tenant_scope.clone(),
            )));
        }
        Ok(self
            .events
            .iter()
            .filter(|e| e.emitted_at_unix_ms >= since_unix_ms)
            .take(limit)
            .collect())
    }

    fn count(&self) -> usize {
        self.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str, class: ExtractorClass, fresh: u64) -> ExtractorRecord {
        ExtractorRecord {
            id: ExtractorId(id.into()),
            class,
            source_path: format!("docs/extractors/{id}.json"),
            last_refreshed_unix_ms: fresh,
            manifest_record_count: 0,
        }
    }

    #[test]
    fn extractor_class_sla_ordering() {
        assert!(ExtractorClass::Hot.sla_budget_ms() < ExtractorClass::Warm.sla_budget_ms());
        assert!(ExtractorClass::Warm.sla_budget_ms() < ExtractorClass::Cold.sla_budget_ms());
    }

    #[test]
    fn extractor_stale_per_class() {
        let hot = record("a", ExtractorClass::Hot, 0);
        assert!(!hot.is_stale(400));
        assert!(hot.is_stale(600));
        let cold = record("b", ExtractorClass::Cold, 0);
        assert!(!cold.is_stale(599_000));
        assert!(cold.is_stale(700_000));
    }

    #[test]
    fn register_and_query() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("a", ExtractorClass::Hot, 100))
            .unwrap();
        m.register_extractor(record("b", ExtractorClass::Cold, 100))
            .unwrap();
        let snap = m
            .query(
                &ManifestQuery {
                    tenant_scope: TenantScope(None),
                    extractor_filter: None,
                    include_stale: true,
                },
                200,
            )
            .unwrap();
        assert_eq!(snap.records.len(), 2);
    }

    #[test]
    fn query_filter_by_class() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("a", ExtractorClass::Hot, 100))
            .unwrap();
        m.register_extractor(record("b", ExtractorClass::Cold, 100))
            .unwrap();
        let snap = m
            .query(
                &ManifestQuery {
                    tenant_scope: TenantScope(None),
                    extractor_filter: Some(ExtractorClass::Hot),
                    include_stale: true,
                },
                200,
            )
            .unwrap();
        assert_eq!(snap.records.len(), 1);
        assert_eq!(snap.records[0].class, ExtractorClass::Hot);
    }

    #[test]
    fn query_skips_stale_by_default() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("hot-stale", ExtractorClass::Hot, 0))
            .unwrap();
        m.register_extractor(record("cold-fresh", ExtractorClass::Cold, 200))
            .unwrap();
        let snap = m
            .query(
                &ManifestQuery {
                    tenant_scope: TenantScope(None),
                    extractor_filter: None,
                    include_stale: false,
                },
                10_000,
            )
            .unwrap();
        assert_eq!(snap.records.len(), 1);
        assert_eq!(snap.records[0].id, ExtractorId("cold-fresh".into()));
    }

    #[test]
    fn refresh_monotonic() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("a", ExtractorClass::Hot, 100))
            .unwrap();
        m.refresh_extractor(&ExtractorId("a".into()), 5, 200)
            .unwrap();
        let result = m.refresh_extractor(&ExtractorId("a".into()), 5, 150);
        assert!(matches!(result, Err(ManifestError::StaleTimestamp { .. })));
    }

    #[test]
    fn refresh_unknown_errors() {
        let mut m = InMemoryManifest::new();
        let result = m.refresh_extractor(&ExtractorId("x".into()), 0, 100);
        assert!(matches!(result, Err(ManifestError::UnknownExtractorId(_))));
    }

    #[test]
    fn duplicate_extractor_errors() {
        let mut m = InMemoryManifest::new();
        m.register_extractor(record("a", ExtractorClass::Hot, 0))
            .unwrap();
        let result = m.register_extractor(record("a", ExtractorClass::Cold, 0));
        assert!(matches!(
            result,
            Err(ManifestError::DuplicateExtractorId(_))
        ));
    }

    fn event(kind: LiveFeedEventKind, ms: u64) -> LiveFeedEvent {
        LiveFeedEvent {
            kind,
            extractor_id: Some(ExtractorId("a".into())),
            tenant_scope: TenantScope(None),
            emitted_at_unix_ms: ms,
            payload_hash: "deadbeef".into(),
        }
    }

    #[test]
    fn live_feed_monotonic() {
        let mut feed = InMemoryLiveFeed::new();
        feed.emit(event(LiveFeedEventKind::ExtractorRefreshed, 100))
            .unwrap();
        feed.emit(event(LiveFeedEventKind::ManifestRowChanged, 200))
            .unwrap();
        let result = feed.emit(event(LiveFeedEventKind::DeadCodeDetected, 150));
        assert!(matches!(
            result,
            Err(LiveFeedError::NonMonotonicTimestamp { .. })
        ));
    }

    #[test]
    fn manifest_rejects_unsupported_tenant_scope_before_read() {
        let mut manifest = InMemoryManifest::new();
        manifest
            .register_extractor(record("a", ExtractorClass::Hot, 100))
            .unwrap();
        let result = manifest.query(
            &ManifestQuery {
                tenant_scope: TenantScope(Some("tenant-hash".into())),
                extractor_filter: None,
                include_stale: true,
            },
            200,
        );
        assert_eq!(
            result,
            Err(ManifestError::TenantScope(TenantScopeRefusal::Unsupported(
                TenantScope(Some("tenant-hash".into()))
            )))
        );
    }

    #[test]
    fn live_feed_rejects_unsupported_tenant_scope_before_read() {
        let mut feed = InMemoryLiveFeed::new();
        feed.emit(event(LiveFeedEventKind::ExtractorRefreshed, 100))
            .unwrap();
        let result = feed.recent(&TenantScope(Some("tenant-hash".into())), 0, 10);
        assert_eq!(
            result,
            Err(LiveFeedError::TenantScope(TenantScopeRefusal::Unsupported(
                TenantScope(Some("tenant-hash".into()))
            )))
        );
    }

    #[test]
    fn live_feed_recent_window() {
        let mut feed = InMemoryLiveFeed::new();
        for ms in [100u64, 200, 300, 400, 500] {
            feed.emit(event(LiveFeedEventKind::ExtractorRefreshed, ms))
                .unwrap();
        }
        let since = feed.recent(&TenantScope(None), 250, 10).unwrap();
        assert_eq!(since.len(), 3);
    }
}
