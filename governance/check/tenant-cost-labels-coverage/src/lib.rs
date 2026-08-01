//! Tenant cost-labels coverage check (ADR-0199 D-2).
//!
//! # Why this crate exists
//!
//! ADR-0199 D-2 mandates the canonical tenant cost label block on every
//! Kubernetes workload + cloud resource:
//!
//!   - `oya.io/tenant-id`
//!   - `oya.io/cost-center`
//!   - `oya.io/workload-class`
//!   - `oya.io/regulatory-pack`
//!
//! This crate scans rendered Helm output and reports per-µservice
//! coverage. Advisory mode this batch; strict promotion follows when
//! the per-µservice coverage backlog reaches zero.
//!
//! # Naming justification
//!
//! `check-tenant-cost-labels-coverage` follows the ADR-0532/0533 de-branded grammar:
//! `<group:check>-<axis:tenant-cost-labels-coverage>`.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;

/// One rendered Helm manifest, identified by file path + microservice +
/// raw YAML text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedManifest {
    pub source_path: String,
    pub microservice: String,
    pub yaml: String,
}

/// One workload finding (per pod-spec object) inside a manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageFinding {
    pub source_path: String,
    pub microservice: String,
    pub kind: WorkloadKind,
    pub workload_name: String,
    pub missing_labels: Vec<String>,
}

impl fmt::Display for CoverageFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}/{} {}): missing labels [{}]",
            self.microservice,
            self.source_path,
            self.kind.wire_name(),
            self.workload_name,
            self.missing_labels.join(", ")
        )
    }
}

/// Kinds of workload-owning Kubernetes objects we cover.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum WorkloadKind {
    Deployment,
    StatefulSet,
    DaemonSet,
    CronJob,
    Job,
}

impl WorkloadKind {
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Deployment => "Deployment",
            Self::StatefulSet => "StatefulSet",
            Self::DaemonSet => "DaemonSet",
            Self::CronJob => "CronJob",
            Self::Job => "Job",
        }
    }
}

/// Required label keys per ADR-0199 D-1.
pub const REQUIRED_LABELS: &[&str] = &[
    "oya.io/tenant-id",
    "oya.io/cost-center",
    "oya.io/workload-class",
    "oya.io/regulatory-pack",
];

/// Coverage report emitted in advisory mode.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct CoverageReport {
    pub manifests_scanned: usize,
    pub workloads_scanned: usize,
    pub workloads_with_full_coverage: usize,
    pub findings: Vec<CoverageFinding>,
    pub microservices_covered: BTreeSet<String>,
}

impl CoverageReport {
    /// Coverage ratio in the closed unit interval `[0.0, 1.0]`.
    #[must_use]
    pub fn coverage_ratio(&self) -> f64 {
        if self.workloads_scanned == 0 {
            return 1.0;
        }
        let total = u32::try_from(self.workloads_scanned).unwrap_or(u32::MAX);
        let covered = u32::try_from(self.workloads_with_full_coverage).unwrap_or(u32::MAX);
        f64::from(covered) / f64::from(total)
    }
}

/// Validate in advisory mode — emits the coverage report, never errors.
#[must_use]
pub fn validate_advisory<I>(manifests: I) -> CoverageReport
where
    I: IntoIterator<Item = RenderedManifest>,
{
    let mut report = CoverageReport::default();
    for m in manifests {
        report.manifests_scanned += 1;
        report.microservices_covered.insert(m.microservice.clone());
        let workloads = parse_workloads(&m.yaml);
        for (kind, name, present_labels) in workloads {
            report.workloads_scanned += 1;
            let mut missing = Vec::new();
            for required in REQUIRED_LABELS {
                if !present_labels.contains(*required) {
                    missing.push((*required).to_string());
                }
            }
            if missing.is_empty() {
                report.workloads_with_full_coverage += 1;
            } else {
                report.findings.push(CoverageFinding {
                    source_path: m.source_path.clone(),
                    microservice: m.microservice.clone(),
                    kind,
                    workload_name: name,
                    missing_labels: missing,
                });
            }
        }
    }
    report
}

/// Validate in strict mode — panics until promoted out of advisory.
/// Strict promotion lands when the per-µservice backlog reaches zero.
pub fn validate_strict<I>(_manifests: I) -> !
where
    I: IntoIterator<Item = RenderedManifest>,
{
    unimplemented!(
        "strict mode pending fleet migration; tracked in registry/placeholder-debt/adr-follow-ups.yaml#adr-0199-tenant-cost-labels-strict"
    )
}

// =====================================================================
// YAML parsing — minimal, deterministic, no external deps
// =====================================================================
//
// We scan the manifest for workload-owning kinds; for each, we extract
// the topmost `metadata.labels` block (skipping spec.template.metadata
// since that's the pod label set, also relevant — we include both).
//
// This parser is purpose-built: real Helm output is line-oriented YAML
// where indentation matters. We accept the simpler shape and the check
// is advisory; if the YAML is too exotic to parse, the workload is
// reported as "unscanned" via missing labels rather than crashing.

#[derive(Debug)]
struct ParsedWorkload {
    kind: WorkloadKind,
    name: String,
    labels: BTreeSet<String>,
}

fn parse_workloads(yaml: &str) -> Vec<(WorkloadKind, String, BTreeSet<String>)> {
    let docs: Vec<&str> = yaml.split("\n---").collect();
    let mut out = Vec::new();
    for doc in docs {
        if let Some(workload) = parse_single_doc(doc) {
            out.push((workload.kind, workload.name, workload.labels));
        }
    }
    out
}

fn parse_single_doc(doc: &str) -> Option<ParsedWorkload> {
    let lines: Vec<&str> = doc.lines().collect();
    let kind = find_kind(&lines)?;
    let name = find_name(&lines).unwrap_or_else(|| "<anonymous>".to_string());
    // Collect every label key that appears anywhere inside `labels:` blocks
    // within the document. This catches both top-level metadata.labels and
    // spec.template.metadata.labels (pod-spec labels — what OpenCost reads).
    let mut labels = BTreeSet::new();
    let mut in_labels_block = false;
    let mut labels_block_indent: Option<usize> = None;
    for line in &lines {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        if trimmed.starts_with("labels:") {
            in_labels_block = true;
            labels_block_indent = Some(indent);
            continue;
        }
        if in_labels_block {
            // exit the labels block once we see a sibling-or-shallower key
            if let Some(base) = labels_block_indent
                && !line.trim().is_empty()
                && indent <= base
            {
                in_labels_block = false;
                labels_block_indent = None;
                // fallthrough to process this line as a non-label
            }
            if in_labels_block {
                // expect `<key>: <value>` lines
                if let Some(colon) = trimmed.find(':') {
                    let key = trimmed[..colon].trim().to_string();
                    if !key.is_empty() && !key.starts_with('#') {
                        labels.insert(key);
                    }
                }
                continue;
            }
        }
    }
    Some(ParsedWorkload { kind, name, labels })
}

fn find_kind(lines: &[&str]) -> Option<WorkloadKind> {
    for line in lines {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("kind:") {
            let value = value.trim();
            return match value {
                "Deployment" => Some(WorkloadKind::Deployment),
                "StatefulSet" => Some(WorkloadKind::StatefulSet),
                "DaemonSet" => Some(WorkloadKind::DaemonSet),
                "CronJob" => Some(WorkloadKind::CronJob),
                "Job" => Some(WorkloadKind::Job),
                _ => None,
            };
        }
    }
    None
}

fn find_name(lines: &[&str]) -> Option<String> {
    // First `name:` that appears at the top-level metadata block.
    let mut in_metadata = false;
    for line in lines {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        if trimmed.starts_with("metadata:") && indent == 0 {
            in_metadata = true;
            continue;
        }
        if in_metadata {
            if !line.trim().is_empty() && indent == 0 {
                in_metadata = false;
            } else if let Some(name) = trimmed.strip_prefix("name:") {
                return Some(name.trim().to_string());
            }
        }
    }
    None
}

// =====================================================================
// Tests
// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render(path: &str, microservice: &str, yaml: &str) -> RenderedManifest {
        RenderedManifest {
            source_path: path.to_string(),
            microservice: microservice.to_string(),
            yaml: yaml.to_string(),
        }
    }

    #[test]
    fn fully_labelled_workload_passes() {
        let manifest = render(
            "microservices/intelligence/iac/helm/foundry/deployment.yaml",
            "foundry",
            r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: foundry-app
  labels:
    app.kubernetes.io/name: foundry
    oya.io/tenant-id: shared
    oya.io/cost-center: axis-foundry
    oya.io/workload-class: app
    oya.io/regulatory-pack: generic
spec:
  replicas: 3
"#,
        );
        let rep = validate_advisory(std::iter::once(manifest));
        assert_eq!(rep.manifests_scanned, 1);
        assert_eq!(rep.workloads_scanned, 1);
        assert_eq!(rep.workloads_with_full_coverage, 1);
        assert!(rep.findings.is_empty());
        assert!((rep.coverage_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn workload_missing_cost_center_is_flagged() {
        let manifest = render(
            "x.yaml",
            "drive",
            r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: drive-app
  labels:
    app.kubernetes.io/name: drive
    oya.io/tenant-id: shared
    oya.io/workload-class: app
    oya.io/regulatory-pack: generic
spec: {}
"#,
        );
        let rep = validate_advisory(std::iter::once(manifest));
        assert_eq!(rep.findings.len(), 1);
        assert_eq!(rep.findings[0].missing_labels, vec!["oya.io/cost-center"]);
    }

    #[test]
    fn workload_missing_all_labels_is_flagged() {
        let manifest = render(
            "x.yaml",
            "calendar",
            r#"---
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: calendar-app
  labels:
    app.kubernetes.io/name: calendar
spec: {}
"#,
        );
        let rep = validate_advisory(std::iter::once(manifest));
        assert_eq!(rep.findings.len(), 1);
        assert_eq!(rep.findings[0].missing_labels.len(), 4);
    }

    #[test]
    fn non_workload_kinds_are_skipped() {
        let manifest = render(
            "x.yaml",
            "calendar",
            r#"---
apiVersion: v1
kind: ConfigMap
metadata:
  name: calendar-config
data:
  key: value
"#,
        );
        let rep = validate_advisory(std::iter::once(manifest));
        assert_eq!(rep.manifests_scanned, 1);
        assert_eq!(rep.workloads_scanned, 0);
    }

    #[test]
    fn multi_doc_manifest_scans_all_workloads() {
        let manifest = render(
            "x.yaml",
            "audit-chain",
            r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: audit-chain-app
  labels:
    oya.io/tenant-id: shared
    oya.io/cost-center: axis-audit-chain
    oya.io/workload-class: app
    oya.io/regulatory-pack: generic
spec: {}
---
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: audit-chain-agent
  labels:
    oya.io/tenant-id: shared
spec: {}
"#,
        );
        let rep = validate_advisory(std::iter::once(manifest));
        assert_eq!(rep.workloads_scanned, 2);
        assert_eq!(rep.workloads_with_full_coverage, 1);
        assert_eq!(rep.findings.len(), 1);
        assert_eq!(rep.findings[0].kind, WorkloadKind::DaemonSet);
    }

    #[test]
    fn coverage_ratio_partial() {
        let manifest = render(
            "x.yaml",
            "ms",
            r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: a
  labels:
    oya.io/tenant-id: shared
    oya.io/cost-center: ms
    oya.io/workload-class: app
    oya.io/regulatory-pack: generic
spec: {}
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: b
  labels:
    oya.io/tenant-id: shared
spec: {}
"#,
        );
        let rep = validate_advisory(std::iter::once(manifest));
        assert!((rep.coverage_ratio() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_set_is_full_coverage() {
        let rep = validate_advisory(std::iter::empty::<RenderedManifest>());
        assert_eq!(rep.workloads_scanned, 0);
        assert!((rep.coverage_ratio() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn microservices_covered_is_unique_set() {
        let m1 = render(
            "a.yaml",
            "drive",
            "kind: Deployment\nmetadata:\n  name: x\n  labels: {}\n",
        );
        let m2 = render(
            "b.yaml",
            "drive",
            "kind: Deployment\nmetadata:\n  name: y\n  labels: {}\n",
        );
        let m3 = render(
            "c.yaml",
            "calendar",
            "kind: Deployment\nmetadata:\n  name: z\n  labels: {}\n",
        );
        let rep = validate_advisory([m1, m2, m3]);
        assert_eq!(rep.microservices_covered.len(), 2);
    }

    #[test]
    fn finding_display_renders_microservice_and_missing_labels() {
        let f = CoverageFinding {
            source_path: "p".into(),
            microservice: "ms".into(),
            kind: WorkloadKind::Deployment,
            workload_name: "x".into(),
            missing_labels: vec!["oya.io/cost-center".into()],
        };
        let s = format!("{f}");
        assert!(s.contains("ms"));
        assert!(s.contains("oya.io/cost-center"));
        assert!(s.contains("Deployment"));
    }
}
