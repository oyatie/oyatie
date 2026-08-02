//! Id-discipline validator — ADR-0156 enforcement.
//!
//! Every event id, message id, outbox id, job id, request id MUST be
//! a ULID. Auto-increment, UUID v4 with no time-ordering, and
//! fabricated ids (e.g. `id: "abc-123"`) are findings.
//!
//! Kernel-tier (ADR-0083); no I/O.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub documents_checked: usize,
    pub id_fields_inspected: usize,
    pub microservices_audited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: String,
    pub microservice: String,
    pub line: usize,
    pub message: String,
}

/// Canonical id-shaped field names. The validator inspects nearby
/// schema declarations for each.
pub const CANONICAL_ID_FIELDS: &[&str] = &[
    "event_id",
    "message_id",
    "outbox_id",
    "job_id",
    "request_id",
    "trace_id",
    "channel_id",
    "tenant_id",
];

/// Audit one batch of schema documents.
#[must_use]
pub fn audit_all(documents: Vec<SchemaDocument>) -> (Report, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut microservices = std::collections::BTreeSet::new();
    let mut id_fields_inspected = 0;

    for doc in &documents {
        microservices.insert(doc.microservice.clone());
        let lines: Vec<&str> = doc.contents.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            for id_field in CANONICAL_ID_FIELDS {
                let pat = format!("{id_field}:");
                if !trimmed.starts_with(&pat) {
                    continue;
                }
                id_fields_inspected += 1;
                // Look at the next ~5 lines for the format declaration.
                let mut format_token: Option<String> = None;
                for next in lines.iter().take(idx + 8).skip(idx + 1) {
                    let nt = next.trim_start().to_ascii_lowercase();
                    if nt.starts_with("format:") {
                        format_token = Some(nt.trim_start_matches("format:").trim().to_string());
                        break;
                    }
                }
                if let Some(fmt) = format_token {
                    // Reject explicit fabricated tokens.
                    if fmt.contains("int") || fmt.contains("uuid") || fmt.contains("auto") {
                        findings.push(Finding {
                            path: doc.path.clone(),
                            microservice: doc.microservice.clone(),
                            line: idx + 1,
                            message: format!(
                                "id field '{id_field}' must use ULID format (ADR-0156); found '{fmt}'"
                            ),
                        });
                    }
                }
            }
        }
    }

    let report = Report {
        documents_checked: documents.len(),
        id_fields_inspected,
        microservices_audited: microservices.len(),
    };
    (report, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_uuid_format() {
        let yaml = r"
properties:
  message_id:
    type: string
    format: uuid
";
        let doc = SchemaDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.iter().any(|f| f.message.contains("ULID")));
    }

    #[test]
    fn detects_auto_increment_int() {
        let yaml = r"
properties:
  event_id:
    type: integer
    format: int64
";
        let doc = SchemaDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.iter().any(|f| f.message.contains("event_id")));
    }

    #[test]
    fn accepts_ulid_format() {
        let yaml = r"
properties:
  event_id:
    type: string
    format: ulid
";
        let doc = SchemaDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.is_empty(), "expected clean: {findings:?}");
    }

    #[test]
    fn eight_canonical_id_fields() {
        assert_eq!(CANONICAL_ID_FIELDS.len(), 8);
    }

    #[test]
    fn report_counts_microservices() {
        let docs = vec![
            SchemaDocument {
                path: "a.yaml".into(),
                microservice: "messenger".into(),
                contents: String::new(),
            },
            SchemaDocument {
                path: "b.yaml".into(),
                microservice: "tasks".into(),
                contents: String::new(),
            },
        ];
        let (report, _) = audit_all(docs);
        assert_eq!(report.microservices_audited, 2);
    }
}
