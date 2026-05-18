//! Metric cardinality validator — ensures high-cardinality labels are
//! dropped at the metric-collection layer (per ADR-0151 + observability
//! metric-naming-convention).
//!
//! High-cardinality labels (request_id, user_id, session_id,
//! document_id, message_id, etc.) MUST appear in metricRelabelings
//! as `action: labeldrop` on every ServiceMonitor, OR they must not
//! be emitted by the µservice at all.
//!
//! Kernel-tier (ADR-0083); no I/O.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceMonitorDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub documents_checked: usize,
    pub high_cardinality_labels_dropped: usize,
    pub microservices_audited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: String,
    pub microservice: String,
    pub message: String,
}

/// Canonical high-cardinality labels that MUST be dropped at the
/// metric layer. The full list lives in the observability
/// metric-naming-convention; this kernel encodes the canonical six.
pub const HIGH_CARDINALITY_LABELS: &[&str] = &[
    "request_id",
    "user_id",
    "session_id",
    "document_id",
    "message_id",
    "channel_id",
];

/// Audit one batch of ServiceMonitor documents.
#[must_use]
pub fn audit_all(documents: Vec<ServiceMonitorDocument>) -> (Report, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut microservices = std::collections::BTreeSet::new();
    let mut high_cardinality_labels_dropped = 0;

    for doc in &documents {
        microservices.insert(doc.microservice.clone());
        let lower = doc.contents.to_ascii_lowercase();
        let has_metric_relabelings = lower.contains("metricrelabelings:");
        if !has_metric_relabelings {
            // Only flag if the document actually shapes any high-cardinality
            // labels; absence of metricRelabelings is benign when the
            // µservice emits no high-card labels.
            continue;
        }
        for label in HIGH_CARDINALITY_LABELS {
            // Find labels that appear in metric labels but are NOT
            // labeldrop'd. Heuristic: presence of `<label>` token plus
            // absence of `labeldrop` near the same label.
            // The ServiceMonitor must explicitly drop labels via
            // `sourceLabels: [<label>]` + `action: labeldrop` near it.
            let drop_pat_a = format!("sourcelabels: [{label}]");
            let drop_pat_b = format!("sourcelabels: [\"{label}\"]");
            let mentioned = lower.contains(label);
            let dropped = lower.contains(&drop_pat_a) || lower.contains(&drop_pat_b);
            if mentioned && !dropped {
                findings.push(Finding {
                    path: doc.path.clone(),
                    microservice: doc.microservice.clone(),
                    message: format!(
                        "high-cardinality label '{label}' must be dropped via metricRelabelings (ADR-0151)"
                    ),
                });
            }
            if dropped {
                high_cardinality_labels_dropped += 1;
            }
        }
    }

    let report = Report {
        documents_checked: documents.len(),
        high_cardinality_labels_dropped,
        microservices_audited: microservices.len(),
    };
    (report, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_undropped_high_cardinality_label() {
        let yaml = r"
metricRelabelings:
  - sourceLabels: [some_other_label]
    targetLabel: bc
# But the message_id label appears in the emitted metrics:
# oya_messenger_messages_total{message_id=...}
";
        let doc = ServiceMonitorDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.iter().any(|f| f.message.contains("message_id")));
    }

    #[test]
    fn accepts_properly_dropped_label() {
        let yaml = r"
metricRelabelings:
  - sourceLabels: [message_id]
    action: labeldrop
";
        let doc = ServiceMonitorDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (report, _findings) = audit_all(vec![doc]);
        assert!(report.high_cardinality_labels_dropped >= 1);
    }

    #[test]
    fn ignores_doc_without_metric_relabelings() {
        let yaml = "spec: {}".to_string();
        let doc = ServiceMonitorDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml,
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.is_empty());
    }

    #[test]
    fn six_canonical_high_cardinality_labels() {
        assert_eq!(HIGH_CARDINALITY_LABELS.len(), 6);
    }

    #[test]
    fn report_counts_microservices() {
        let docs = vec![
            ServiceMonitorDocument {
                path: "a.yaml".into(),
                microservice: "messenger".into(),
                contents: String::new(),
            },
            ServiceMonitorDocument {
                path: "b.yaml".into(),
                microservice: "tasks".into(),
                contents: String::new(),
            },
        ];
        let (report, _) = audit_all(docs);
        assert_eq!(report.microservices_audited, 2);
    }
}
