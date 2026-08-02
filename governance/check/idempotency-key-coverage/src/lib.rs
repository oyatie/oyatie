//! Idempotency-Key coverage validator — ADR-0149 enforcement gate.
//!
//! # Contract
//!
//! Every state-changing operation (POST/PUT/PATCH/DELETE) in every
//! µservice OpenAPI 3.2.0 document MUST declare the canonical
//! `Idempotency-Key` header parameter — either inline or via a
//! `$ref` to `#/components/parameters/IdempotencyKey`.
//!
//! Implementation: line-oriented YAML scanner (no full YAML parser
//! to avoid an extra dependency); kernel-tier per ADR-0083.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenApiDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageReport {
    pub documents_checked: usize,
    pub state_changing_ops_checked: usize,
    pub state_changing_ops_covered: usize,
    pub microservices_audited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: String,
    pub microservice: String,
    pub line: usize,
    pub message: String,
}

const STATE_CHANGING_VERBS: &[&str] = &["post:", "put:", "patch:", "delete:"];

/// Audit one batch of OpenAPI documents and return (report, findings).
///
/// # Errors
/// None — invalid documents yield findings, not errors.
#[must_use]
pub fn audit_all(documents: Vec<OpenApiDocument>) -> (CoverageReport, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut microservices = std::collections::BTreeSet::new();
    let mut state_changing_ops_checked = 0;
    let mut state_changing_ops_covered = 0;

    for doc in &documents {
        microservices.insert(doc.microservice.clone());
        for op in extract_state_changing_operations(&doc.contents) {
            state_changing_ops_checked += 1;
            if operation_has_idempotency_key(&doc.contents, op.start_line, op.end_line) {
                state_changing_ops_covered += 1;
            } else {
                findings.push(Finding {
                    path: doc.path.clone(),
                    microservice: doc.microservice.clone(),
                    line: op.start_line,
                    message: format!(
                        "state-changing operation '{}' missing Idempotency-Key parameter (ADR-0149)",
                        op.verb
                    ),
                });
            }
        }
    }

    let report = CoverageReport {
        documents_checked: documents.len(),
        state_changing_ops_checked,
        state_changing_ops_covered,
        microservices_audited: microservices.len(),
    };
    (report, findings)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct OpBounds {
    verb: String,
    start_line: usize, // 1-indexed
    end_line: usize,
}

fn extract_state_changing_operations(yaml: &str) -> Vec<OpBounds> {
    let mut out = Vec::new();
    let lines: Vec<&str> = yaml.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        for verb in STATE_CHANGING_VERBS {
            if trimmed.starts_with(verb) {
                // Operation block spans until the next line at the same
                // or shallower indentation that starts with a sibling key
                // OR until EOF.
                let indent = line.len() - trimmed.len();
                let mut end = lines.len();
                for (jdx, next_line) in lines.iter().enumerate().skip(idx + 1) {
                    let next_trimmed = next_line.trim_start();
                    if next_trimmed.is_empty() {
                        continue;
                    }
                    let next_indent = next_line.len() - next_trimmed.len();
                    if next_indent <= indent {
                        end = jdx;
                        break;
                    }
                }
                out.push(OpBounds {
                    verb: verb.trim_end_matches(':').to_string(),
                    start_line: idx + 1,
                    end_line: end,
                });
            }
        }
    }
    out
}

fn operation_has_idempotency_key(yaml: &str, start: usize, end: usize) -> bool {
    // 1-indexed -> 0-indexed slice.
    let lines: Vec<&str> = yaml.lines().collect();
    let start_idx = start.saturating_sub(1);
    let end_idx = end.min(lines.len());
    for line in &lines[start_idx..end_idx] {
        let l = line.trim_start().to_ascii_lowercase();
        if l.contains("idempotency-key")
            || l.contains("idempotencykey")
            || l.contains("$ref: '#/components/parameters/idempotencykey'")
            || l.contains("$ref: \"#/components/parameters/idempotencykey\"")
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_state_changing_op_missing_idempotency() {
        let yaml = r"
paths:
  /channels:
    post:
      summary: Create
      responses: {'201': {}}
";
        let doc = OpenApiDocument {
            path: "test.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("post"));
    }

    #[test]
    fn accepts_op_with_inline_idempotency_key() {
        let yaml = r#"
paths:
  /channels:
    post:
      parameters:
        - in: header
          name: Idempotency-Key
          required: true
          schema: {type: string}
      responses: {'201': {}}
"#;
        let doc = OpenApiDocument {
            path: "test.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(
            findings.is_empty(),
            "no findings expected, got: {findings:?}"
        );
    }

    #[test]
    fn accepts_op_with_ref_to_canonical_component() {
        let yaml = r#"
paths:
  /channels:
    post:
      parameters:
        - $ref: '#/components/parameters/IdempotencyKey'
      responses: {'201': {}}
"#;
        let doc = OpenApiDocument {
            path: "test.yaml".into(),
            microservice: "tasks".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.is_empty());
    }

    #[test]
    fn ignores_read_operations() {
        let yaml = r"
paths:
  /channels:
    get:
      summary: List
      responses: {'200': {}}
";
        let doc = OpenApiDocument {
            path: "test.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (report, findings) = audit_all(vec![doc]);
        assert_eq!(report.state_changing_ops_checked, 0);
        assert!(findings.is_empty());
    }

    #[test]
    fn report_counts_microservices() {
        let docs = vec![
            OpenApiDocument {
                path: "a.yaml".into(),
                microservice: "messenger".into(),
                contents: "paths: {}".into(),
            },
            OpenApiDocument {
                path: "b.yaml".into(),
                microservice: "tasks".into(),
                contents: "paths: {}".into(),
            },
        ];
        let (report, _) = audit_all(docs);
        assert_eq!(report.documents_checked, 2);
        assert_eq!(report.microservices_audited, 2);
    }
}
