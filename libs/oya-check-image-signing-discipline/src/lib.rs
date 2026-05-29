//! Image-signing discipline validator — enforces cosign + Trivy +
//! SLSA provenance per ADR-0146 + ADR-0039.
//!
//! Kernel-tier (ADR-0083); no I/O.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowDocument {
    pub path: String,
    pub contents: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    pub workflows_checked: usize,
    pub cosign_present: bool,
    pub trivy_present: bool,
    pub slsa_present: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: String,
    pub message: String,
}

/// Audit a batch of GitHub Actions workflow documents and return
/// (report, findings). The runner hands in `.github/workflows/*.yml`.
#[must_use]
pub fn audit_all(documents: Vec<WorkflowDocument>) -> (Report, Vec<Finding>) {
    let mut findings = Vec::new();
    let mut cosign_present = false;
    let mut trivy_present = false;
    let mut slsa_present = false;

    for doc in &documents {
        let lower = doc.contents.to_ascii_lowercase();
        if doc.path.ends_with("cosign.yml") || doc.path.ends_with("cosign.yaml") {
            if !lower.contains("cosign sign") && !lower.contains("cosign-sign") {
                findings.push(Finding {
                    path: doc.path.clone(),
                    message: "cosign workflow present but `cosign sign` step missing".into(),
                });
            } else {
                cosign_present = true;
            }
            if !lower.contains("fulcio") && !lower.contains("oidc") && !lower.contains("id-token") {
                findings.push(Finding {
                    path: doc.path.clone(),
                    message: "cosign workflow must use sigstore Fulcio / OIDC keyless flow".into(),
                });
            }
        }
        if lower.contains("trivy") {
            trivy_present = true;
        }
        if doc.path.ends_with("slsa.yml")
            || doc.path.ends_with("slsa.yaml")
            || lower.contains("slsa-github-generator")
            || lower.contains("slsa-framework")
        {
            slsa_present = true;
        }
    }

    if !cosign_present {
        findings.push(Finding {
            path: ".github/workflows/cosign.yml".into(),
            message: "no runnable cosign workflow found (ADR-0146)".into(),
        });
    }
    if !trivy_present {
        findings.push(Finding {
            path: ".github/workflows".into(),
            message: "no Trivy container-scan step found (image-signing-canonical.md)".into(),
        });
    }
    if !slsa_present {
        findings.push(Finding {
            path: ".github/workflows/slsa.yml".into(),
            message: "no SLSA provenance workflow found (ADR-0039)".into(),
        });
    }

    let report = Report {
        workflows_checked: documents.len(),
        cosign_present,
        trivy_present,
        slsa_present,
    };
    (report, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_missing_cosign() {
        let docs: Vec<WorkflowDocument> = vec![];
        let (report, findings) = audit_all(docs);
        assert!(!report.cosign_present);
        assert!(
            findings
                .iter()
                .any(|f| f.message.contains("cosign workflow"))
        );
    }

    #[test]
    fn accepts_full_cosign_with_oidc() {
        let cosign = WorkflowDocument {
            path: ".github/workflows/cosign.yml".into(),
            contents: "permissions:\n  id-token: write\nsteps:\n  - name: cosign sign\n    run: cosign sign --yes ...".into(),
        };
        let trivy = WorkflowDocument {
            path: ".github/workflows/security.yml".into(),
            contents: "steps:\n  - name: trivy\n    run: trivy image ...".into(),
        };
        let slsa = WorkflowDocument {
            path: ".github/workflows/slsa.yml".into(),
            contents: "uses: slsa-framework/slsa-github-generator/...".into(),
        };
        let (report, findings) = audit_all(vec![cosign, trivy, slsa]);
        assert!(report.cosign_present);
        assert!(report.trivy_present);
        assert!(report.slsa_present);
        assert!(findings.is_empty(), "expected clean: {findings:?}");
    }

    #[test]
    fn detects_cosign_without_oidc() {
        let cosign = WorkflowDocument {
            path: ".github/workflows/cosign.yml".into(),
            contents: "steps:\n  - name: cosign sign\n    run: cosign sign --key /priv.key".into(),
        };
        let trivy = WorkflowDocument {
            path: ".github/workflows/sec.yml".into(),
            contents: "trivy".into(),
        };
        let slsa = WorkflowDocument {
            path: ".github/workflows/slsa.yml".into(),
            contents: "slsa-framework".into(),
        };
        let (_report, findings) = audit_all(vec![cosign, trivy, slsa]);
        assert!(findings.iter().any(|f| f.message.contains("Fulcio")));
    }

    #[test]
    fn detects_missing_trivy() {
        let cosign = WorkflowDocument {
            path: ".github/workflows/cosign.yml".into(),
            contents: "id-token: write\ncosign sign".into(),
        };
        let slsa = WorkflowDocument {
            path: ".github/workflows/slsa.yml".into(),
            contents: "slsa-framework".into(),
        };
        let (_report, findings) = audit_all(vec![cosign, slsa]);
        assert!(findings.iter().any(|f| f.message.contains("Trivy")));
    }

    #[test]
    fn detects_missing_slsa() {
        let cosign = WorkflowDocument {
            path: ".github/workflows/cosign.yml".into(),
            contents: "id-token: write\ncosign sign".into(),
        };
        let trivy = WorkflowDocument {
            path: ".github/workflows/sec.yml".into(),
            contents: "trivy".into(),
        };
        let (_report, findings) = audit_all(vec![cosign, trivy]);
        assert!(findings.iter().any(|f| f.message.contains("SLSA")));
    }
}
