//! Cursor-pagination coverage validator — ADR-0150 enforcement gate.
//!
//! # Contract
//!
//! Every `get:` operation that returns a collection-shaped envelope
//! MUST declare `cursor` + `page_size` query parameters. NO `get:`
//! operation may declare `offset` or `page` query parameters.
//!
//! Kernel-tier (ADR-0083); no I/O.

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
    pub get_ops_checked: usize,
    pub list_ops_checked: usize,
    pub microservices_audited: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: String,
    pub microservice: String,
    pub line: usize,
    pub message: String,
}

/// Audit one batch of OpenAPI documents and return (report, findings).
#[must_use]
pub fn audit_all(documents: Vec<OpenApiDocument>) -> (CoverageReport, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut microservices = std::collections::BTreeSet::new();
    let mut get_ops_checked = 0;
    let mut list_ops_checked = 0;

    for doc in &documents {
        microservices.insert(doc.microservice.clone());
        let lines: Vec<&str> = doc.contents.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("get:") {
                continue;
            }
            get_ops_checked += 1;
            let indent = line.len() - trimmed.len();
            let mut end = lines.len();
            for (jdx, next) in lines.iter().enumerate().skip(idx + 1) {
                let next_t = next.trim_start();
                if next_t.is_empty() {
                    continue;
                }
                let next_indent = next.len() - next_t.len();
                if next_indent <= indent {
                    end = jdx;
                    break;
                }
            }
            // Look for `offset:` / `page:` (FORBIDDEN) inside the op
            // body. Use exact match — `name: page_size` is canonical
            // and MUST NOT trigger.
            for (kdx, body_line) in lines.iter().enumerate().take(end).skip(idx + 1) {
                let bt = body_line.trim_start().to_ascii_lowercase();
                let bt = bt.trim_end_matches([' ', '\t']).to_string();
                if bt == "name: offset" || bt == "name: page" {
                    findings.push(Finding {
                        path: doc.path.clone(),
                        microservice: doc.microservice.clone(),
                        line: kdx + 1,
                        message: "offset/page pagination is FORBIDDEN (ADR-0150)".into(),
                    });
                }
            }
            // Heuristic: is this a list operation? Look for list indicators
            // in the response schema (`items:`, `next_cursor`, etc.) or
            // path-end with a plural like `/channels`.
            let is_list = is_list_operation(&lines, idx, end);
            if is_list {
                list_ops_checked += 1;
                let has_cursor = op_declares_cursor_params(&lines, idx, end);
                if !has_cursor {
                    findings.push(Finding {
                        path: doc.path.clone(),
                        microservice: doc.microservice.clone(),
                        line: idx + 1,
                        message: "list operation missing cursor + page_size parameters (ADR-0150)"
                            .into(),
                    });
                }
            }
        }
    }

    let report = CoverageReport {
        documents_checked: documents.len(),
        get_ops_checked,
        list_ops_checked,
        microservices_audited: microservices.len(),
    };
    (report, findings)
}

fn is_list_operation(lines: &[&str], start: usize, end: usize) -> bool {
    // Cheap heuristic: scan the operation body for any of the
    // signals below.
    for line in &lines[start..end] {
        let l = line.trim_start().to_ascii_lowercase();
        if l.starts_with("items:")
            || l.contains("next_cursor")
            || l.contains("has_more")
            || l.starts_with("- $ref: '#/components/parameters/cursor'")
            || l.starts_with("- $ref: '#/components/parameters/pagesize'")
        {
            return true;
        }
    }
    false
}

fn op_declares_cursor_params(lines: &[&str], start: usize, end: usize) -> bool {
    let mut has_cursor = false;
    let mut has_page_size = false;
    for line in &lines[start..end] {
        let l = line.trim_start().to_ascii_lowercase();
        if l.contains("name: cursor") || l.contains("parameters/cursor") {
            has_cursor = true;
        }
        if l.contains("name: page_size") || l.contains("parameters/pagesize") {
            has_page_size = true;
        }
    }
    has_cursor && has_page_size
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_list_op_missing_cursor() {
        let yaml = r"
paths:
  /channels:
    get:
      responses:
        '200':
          schema:
            properties:
              items: {}
              next_cursor: {}
";
        let doc = OpenApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("missing cursor"))
        );
    }

    #[test]
    fn detects_forbidden_offset_param() {
        let yaml = r"
paths:
  /channels:
    get:
      parameters:
        - in: query
          name: offset
          schema: {type: integer}
      responses: {}
";
        let doc = OpenApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.iter().any(|f| f.message.contains("FORBIDDEN")));
    }

    #[test]
    fn accepts_list_op_with_cursor_and_page_size() {
        let yaml = r"
paths:
  /channels:
    get:
      parameters:
        - in: query
          name: cursor
          schema: {type: string}
        - in: query
          name: page_size
          schema: {type: integer}
      responses:
        '200':
          schema:
            properties:
              items: {}
              next_cursor: {}
";
        let doc = OpenApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (_report, findings) = audit_all(vec![doc]);
        assert!(findings.is_empty(), "expected clean, got: {findings:?}");
    }

    #[test]
    fn ignores_non_list_get_ops() {
        let yaml = r"
paths:
  /channels/{id}:
    get:
      responses:
        '200':
          schema:
            properties:
              id: {}
              name: {}
";
        let doc = OpenApiDocument {
            path: "t.yaml".into(),
            microservice: "messenger".into(),
            contents: yaml.into(),
        };
        let (report, findings) = audit_all(vec![doc]);
        assert_eq!(report.list_ops_checked, 0);
        assert!(findings.is_empty());
    }

    #[test]
    fn report_counts_microservices() {
        let docs = vec![
            OpenApiDocument {
                path: "a.yaml".into(),
                microservice: "messenger".into(),
                contents: String::new(),
            },
            OpenApiDocument {
                path: "b.yaml".into(),
                microservice: "tasks".into(),
                contents: String::new(),
            },
        ];
        let (report, _) = audit_all(docs);
        assert_eq!(report.microservices_audited, 2);
    }
}
