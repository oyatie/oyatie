//! Event schema versioning validator — ADR-0154 enforcement gate.
//!
//! Every AsyncAPI 3.1.0 message MUST declare a `version` header
//! matching the SemVer pattern. Missing `version` is a finding.
//!
//! Kernel-tier (ADR-0083); no I/O.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AsyncApiDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub documents_checked: usize,
    pub documents_with_version_field: usize,
    pub microservices_audited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: String,
    pub microservice: String,
    pub message: String,
}

/// Audit one batch of AsyncAPI documents.
#[must_use]
pub fn audit_all(documents: Vec<AsyncApiDocument>) -> (Report, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut microservices = std::collections::BTreeSet::new();
    let mut documents_with_version_field = 0;

    for doc in &documents {
        microservices.insert(doc.microservice.clone());
        let lower = doc.contents.to_ascii_lowercase();
        let asyncapi_present = lower.contains("asyncapi:");
        if !asyncapi_present {
            continue;
        }
        // Look for `version:` declared as a message-header field. We
        // accept either the canonical token `version: { type: string, pattern: "^[0-9]+\\.[0-9]+\\.[0-9]+$" }`
        // or any `version` field inside a `headers:` block.
        let has_version = lower.contains("version:")
            && (lower.contains("pattern: \"^[0-9]+")
                || lower.contains("event_version")
                || lower.contains("schema_version")
                || version_appears_in_headers_block(&doc.contents));
        if has_version {
            documents_with_version_field += 1;
        } else {
            findings.push(Finding {
                path: doc.path.clone(),
                microservice: doc.microservice.clone(),
                message: "AsyncAPI document missing canonical event `version` header (ADR-0154)"
                    .into(),
            });
        }
    }

    let report = Report {
        documents_checked: documents.len(),
        documents_with_version_field,
        microservices_audited: microservices.len(),
    };
    (report, findings)
}

fn version_appears_in_headers_block(yaml: &str) -> bool {
    // Scan for a `headers:` line then check the next ~20 lines for
    // `version:` (heuristic; tolerates indentation noise).
    let lines: Vec<&str> = yaml.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("headers:") {
            for next in lines.iter().take(i + 20).skip(i + 1) {
                if next.trim_start().starts_with("version:") {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing_version_field() {
        let yaml = r"
asyncapi: 3.1.0
channels:
  message-posted:
    messages:
      MessagePosted:
        payload: {}
";
        let doc = AsyncApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.iter().any(|f| f.message.contains("version")));
    }

    #[test]
    fn accepts_canonical_version_pattern() {
        let yaml = r#"
asyncapi: 3.1.0
channels:
  message-posted:
    messages:
      MessagePosted:
        headers:
          type: object
          required: [event_id, event_kind, version, tenant_id]
          properties:
            version: {type: string, pattern: "^[0-9]+\\.[0-9]+\\.[0-9]+$"}
"#;
        let doc = AsyncApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (report, findings) = audit_all(vec![doc]);
        assert_eq!(report.documents_with_version_field, 1);
        assert!(findings.is_empty());
    }

    #[test]
    fn ignores_non_asyncapi_documents() {
        let yaml = "kind: ConfigMap";
        let doc = AsyncApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (report, findings) = audit_all(vec![doc]);
        assert_eq!(report.documents_with_version_field, 0);
        assert!(findings.is_empty());
    }

    #[test]
    fn accepts_event_version_field_name_variant() {
        let yaml = r"
asyncapi: 3.1.0
channels:
  msg:
    messages:
      M:
        headers:
          properties:
            event_version: {type: string}
";
        let doc = AsyncApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.is_empty());
    }

    #[test]
    fn report_counts_microservices() {
        let docs = vec![
            AsyncApiDocument {
                path: "a.yaml".into(),
                microservice: "messenger".into(),
                contents: "asyncapi: 3.1.0".into(),
            },
            AsyncApiDocument {
                path: "b.yaml".into(),
                microservice: "tasks".into(),
                contents: "asyncapi: 3.1.0".into(),
            },
        ];
        let (report, _) = audit_all(docs);
        assert_eq!(report.microservices_audited, 2);
    }
}
