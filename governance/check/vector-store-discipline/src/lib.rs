//! `check-vector-store-discipline` — advisory gate per ADR-0192
//! §"pgvector — degraded fallback only for ≤10M-vector tenants".
//!
//! Ensures no µservice uses the embedded-tier pgvector path beyond the
//! 10M-vector ceiling; flags collections that exceed the ceiling and
//! directs the caller to delegate to the Milvus adapter (Phase 0) or
//! the Phase-2 in-house `oya-vector-store-server` adapter.
//!
//! Per ADR-0083 this kernel is pure; the caller pre-harvests
//! [`CollectionUsage`] records (one per per-tenant per-collection slot)
//! from a runner and feeds them in.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fmt;

/// ADR-0192 hard ceiling for the embedded-tier pgvector path.
pub const PGVECTOR_HARD_CEILING_VECTORS: u64 = 10_000_000;

/// Engine backend currently serving a collection.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VectorBackend {
    /// pgvector embedded in Tier 1 Postgres OLTP.
    Pgvector,
    /// Milvus 2.6.x via the canonical adapter (Phase 0).
    Milvus,
    /// Phase-2 in-house `oya-vector-store-server`.
    InHouse,
}

impl VectorBackend {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Pgvector => "pgvector",
            Self::Milvus => "milvus",
            Self::InHouse => "in_house",
        }
    }

    pub fn parse_label(value: &str) -> Option<Self> {
        match value {
            "pgvector" => Some(Self::Pgvector),
            "milvus" => Some(Self::Milvus),
            "in_house" => Some(Self::InHouse),
            _ => None,
        }
    }
}

/// Pre-harvested usage record. The runner emits one of these per per-tenant
/// per-collection slot (e.g., by scanning Postgres `pg_stat_user_tables` for
/// pgvector tables and Milvus's `getCollectionStatistics` for Milvus).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectionUsage {
    pub microservice: String,
    pub tenant_id: String,
    pub collection: String,
    pub backend: VectorBackend,
    pub vector_count: u64,
}

/// Violation kinds produced by this lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ViolationKind {
    /// pgvector beyond the ADR-0192 ceiling.
    PgvectorOverCeiling,
    /// Empty microservice / tenant / collection identifier (caller bug).
    MalformedRecord,
}

impl fmt::Display for ViolationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PgvectorOverCeiling => write!(f, "pgvector_over_ceiling"),
            Self::MalformedRecord => write!(f, "malformed_record"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Violation {
    pub microservice: String,
    pub tenant_id: String,
    pub collection: String,
    pub kind: ViolationKind,
    pub vector_count: u64,
    pub ceiling: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub records_checked: usize,
    pub violations: Vec<Violation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    DuplicateRecord {
        microservice: String,
        tenant_id: String,
        collection: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRecord {
                microservice,
                tenant_id,
                collection,
            } => write!(
                f,
                "duplicate usage record for {microservice}/{tenant_id}/{collection}"
            ),
        }
    }
}

impl std::error::Error for Error {}

pub fn check(records: &[CollectionUsage]) -> Result<Report, Error> {
    use std::collections::BTreeSet;
    let mut seen = BTreeSet::new();
    let mut violations = Vec::new();

    for rec in records {
        if rec.microservice.trim().is_empty()
            || rec.tenant_id.trim().is_empty()
            || rec.collection.trim().is_empty()
        {
            violations.push(Violation {
                microservice: rec.microservice.clone(),
                tenant_id: rec.tenant_id.clone(),
                collection: rec.collection.clone(),
                kind: ViolationKind::MalformedRecord,
                vector_count: rec.vector_count,
                ceiling: PGVECTOR_HARD_CEILING_VECTORS,
            });
            continue;
        }

        let key = (
            rec.microservice.clone(),
            rec.tenant_id.clone(),
            rec.collection.clone(),
        );
        if !seen.insert(key) {
            return Err(Error::DuplicateRecord {
                microservice: rec.microservice.clone(),
                tenant_id: rec.tenant_id.clone(),
                collection: rec.collection.clone(),
            });
        }

        if rec.backend == VectorBackend::Pgvector
            && rec.vector_count > PGVECTOR_HARD_CEILING_VECTORS
        {
            violations.push(Violation {
                microservice: rec.microservice.clone(),
                tenant_id: rec.tenant_id.clone(),
                collection: rec.collection.clone(),
                kind: ViolationKind::PgvectorOverCeiling,
                vector_count: rec.vector_count,
                ceiling: PGVECTOR_HARD_CEILING_VECTORS,
            });
        }
    }

    Ok(Report {
        records_checked: records.len(),
        violations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(ms: &str, tid: &str, coll: &str, backend: VectorBackend, n: u64) -> CollectionUsage {
        CollectionUsage {
            microservice: ms.into(),
            tenant_id: tid.into(),
            collection: coll.into(),
            backend,
            vector_count: n,
        }
    }

    #[test]
    fn empty_input_passes() {
        let r = check(&[]).unwrap();
        assert_eq!(r.records_checked, 0);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn pgvector_under_ceiling_passes() {
        let r = check(&[rec(
            "foundry",
            "ten_acme",
            "rag",
            VectorBackend::Pgvector,
            9_999_999,
        )])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn pgvector_at_exact_ceiling_passes() {
        let r = check(&[rec(
            "foundry",
            "ten_acme",
            "rag",
            VectorBackend::Pgvector,
            10_000_000,
        )])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn pgvector_over_ceiling_flags() {
        let r = check(&[rec(
            "foundry",
            "ten_acme",
            "rag",
            VectorBackend::Pgvector,
            20_000_000,
        )])
        .unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::PgvectorOverCeiling);
        assert_eq!(r.violations[0].vector_count, 20_000_000);
        assert_eq!(r.violations[0].ceiling, 10_000_000);
    }

    #[test]
    fn milvus_at_any_count_passes() {
        let r = check(&[rec(
            "foundry",
            "ten_acme",
            "rag",
            VectorBackend::Milvus,
            1_000_000_000,
        )])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn in_house_at_any_count_passes() {
        let r = check(&[rec(
            "foundry",
            "ten_acme",
            "rag",
            VectorBackend::InHouse,
            10_000_000_000,
        )])
        .unwrap();
        assert!(r.violations.is_empty());
    }

    #[test]
    fn malformed_record_flagged() {
        let r = check(&[rec("", "ten_acme", "rag", VectorBackend::Pgvector, 1)]).unwrap();
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].kind, ViolationKind::MalformedRecord);
    }

    #[test]
    fn duplicate_record_errors() {
        let err = check(&[
            rec("foundry", "ten_acme", "rag", VectorBackend::Pgvector, 1),
            rec("foundry", "ten_acme", "rag", VectorBackend::Milvus, 2),
        ])
        .unwrap_err();
        assert!(matches!(err, Error::DuplicateRecord { .. }));
    }

    #[test]
    fn vector_backend_label_round_trips() {
        for b in [
            VectorBackend::Pgvector,
            VectorBackend::Milvus,
            VectorBackend::InHouse,
        ] {
            assert_eq!(VectorBackend::parse_label(b.label()), Some(b));
        }
        assert_eq!(VectorBackend::parse_label("unknown"), None);
    }

    #[test]
    fn multi_record_only_offenders_flagged() {
        let records = vec![
            rec(
                "foundry",
                "ten_a",
                "rag",
                VectorBackend::Pgvector,
                5_000_000,
            ),
            rec(
                "foundry",
                "ten_b",
                "rag",
                VectorBackend::Pgvector,
                50_000_000,
            ), // OVER
            rec(
                "foundry",
                "ten_c",
                "rag",
                VectorBackend::Milvus,
                500_000_000,
            ),
            rec("search", "ten_a", "rag", VectorBackend::Pgvector, 1),
        ];
        let r = check(&records).unwrap();
        assert_eq!(r.records_checked, 4);
        assert_eq!(r.violations.len(), 1);
        assert_eq!(r.violations[0].tenant_id, "ten_b");
    }
}
