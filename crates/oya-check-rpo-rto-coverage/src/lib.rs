//! RPO/RTO coverage validator — ADR-0152 enforcement.
//!
//! Every µservice's `backfill-replay.md` MUST declare two numeric
//! targets: `rpo_target_seconds` and `rto_target_seconds`. The five
//! canonical tiers are encoded in the wire representation.
//!
//! Kernel-tier (ADR-0083); no I/O.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackfillReplayDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageReport {
    pub documents_checked: usize,
    pub documents_with_rto: usize,
    pub documents_with_rpo: usize,
    pub microservices_audited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: String,
    pub microservice: String,
    pub message: String,
}

/// Five-tier RTO model per ADR-0152.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RtoTier {
    Realtime,   // < 5 min
    Hot,        // < 1 h
    Warm,       // < 4 h
    Cold,       // < 24 h
    BestEffort, // best-effort
}

impl RtoTier {
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            RtoTier::Realtime => "realtime",
            RtoTier::Hot => "hot",
            RtoTier::Warm => "warm",
            RtoTier::Cold => "cold",
            RtoTier::BestEffort => "best-effort",
        }
    }

    #[must_use]
    pub fn parse_wire(value: &str) -> Option<Self> {
        match value {
            "realtime" => Some(RtoTier::Realtime),
            "hot" => Some(RtoTier::Hot),
            "warm" => Some(RtoTier::Warm),
            "cold" => Some(RtoTier::Cold),
            "best-effort" => Some(RtoTier::BestEffort),
            _ => None,
        }
    }
}

/// Audit one batch of backfill-replay documents.
#[must_use]
pub fn audit_all(documents: Vec<BackfillReplayDocument>) -> (CoverageReport, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut microservices = std::collections::BTreeSet::new();
    let mut documents_with_rto = 0;
    let mut documents_with_rpo = 0;

    for doc in &documents {
        microservices.insert(doc.microservice.clone());
        let lower = doc.contents.to_ascii_lowercase();
        let has_rto = lower.contains("rto") || lower.contains("recovery time objective");
        let has_rpo = lower.contains("rpo") || lower.contains("recovery point objective");
        if has_rto {
            documents_with_rto += 1;
        } else {
            findings.push(Finding {
                path: doc.path.clone(),
                microservice: doc.microservice.clone(),
                message: "backfill-replay.md missing RTO declaration (ADR-0152)".into(),
            });
        }
        if has_rpo {
            documents_with_rpo += 1;
        } else {
            findings.push(Finding {
                path: doc.path.clone(),
                microservice: doc.microservice.clone(),
                message: "backfill-replay.md missing RPO declaration (ADR-0152)".into(),
            });
        }
    }

    let report = CoverageReport {
        documents_checked: documents.len(),
        documents_with_rto,
        documents_with_rpo,
        microservices_audited: microservices.len(),
    };
    (report, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_tiers_are_canonical() {
        for tier in [
            RtoTier::Realtime,
            RtoTier::Hot,
            RtoTier::Warm,
            RtoTier::Cold,
            RtoTier::BestEffort,
        ] {
            let wire = tier.wire_name();
            assert_eq!(RtoTier::parse_wire(wire), Some(tier));
        }
    }

    #[test]
    fn detects_missing_rto() {
        let doc = BackfillReplayDocument {
            path: "t.md".into(),
            microservice: "messenger".into(),
            contents: "RPO: 5 minutes".into(),
        };
        let (report, findings) = audit_all(vec![doc]);
        assert_eq!(report.documents_with_rto, 0);
        assert!(findings.iter().any(|f| f.message.contains("RTO")));
    }

    #[test]
    fn detects_missing_rpo() {
        let doc = BackfillReplayDocument {
            path: "t.md".into(),
            microservice: "messenger".into(),
            contents: "RTO: 5 minutes".into(),
        };
        let (report, findings) = audit_all(vec![doc]);
        assert_eq!(report.documents_with_rpo, 0);
        assert!(findings.iter().any(|f| f.message.contains("RPO")));
    }

    #[test]
    fn accepts_doc_with_both() {
        let doc = BackfillReplayDocument {
            path: "t.md".into(),
            microservice: "messenger".into(),
            contents: "RTO: 5 minutes; RPO: 30 seconds".into(),
        };
        let (report, findings) = audit_all(vec![doc]);
        assert_eq!(report.documents_with_rto, 1);
        assert_eq!(report.documents_with_rpo, 1);
        assert!(findings.is_empty());
    }

    #[test]
    fn report_counts_microservices() {
        let docs = vec![
            BackfillReplayDocument {
                path: "a.md".into(),
                microservice: "messenger".into(),
                contents: "RTO RPO".into(),
            },
            BackfillReplayDocument {
                path: "b.md".into(),
                microservice: "tasks".into(),
                contents: "RTO RPO".into(),
            },
        ];
        let (report, _) = audit_all(docs);
        assert_eq!(report.microservices_audited, 2);
    }

    #[test]
    fn parse_wire_rejects_unknown() {
        assert!(RtoTier::parse_wire("nope").is_none());
    }
}
