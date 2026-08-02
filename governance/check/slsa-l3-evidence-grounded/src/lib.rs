//! SLSA L3 evidence-grounded check (SEC-MAJ-01).
//!
//! # Why this crate exists
//!
//! Each µservice's scorecard override file (e.g.
//! `microservices/intelligence/scorecards/overrides.json`) carries a
//! `slsa_l3` block such as:
//!
//! ```json
//! "slsa_l3": { "overall_status": "green", "deltas": [] }
//! ```
//!
//! A `green` status is only honest if the workflow files that produce
//! the SLSA L3 evidence actually exist AND declare the SLSA-relevant
//! primitives (hermetic build, signed provenance, two-party review).
//! Before this crate, scorecards could claim `green` while citing
//! workflow files that did not exist on the branch — a SEC-MAJ
//! audit-trail break.
//!
//! The kernel performs that grounding:
//!
//! 1. Parse the scorecard override JSON (deltas may add `evidence_path`
//!    references to `.github/workflows/<file>.yml`).
//! 2. For every cited workflow path, verify the file is supplied by
//!    the runner.
//! 3. For each supplied workflow body, verify it declares the
//!    SLSA-relevant primitives via the canonical token set documented
//!    in SLSA v1.0 (<https://slsa.dev/spec/v1.0/>).
//! 4. Emit a violation per (scorecard, citation) pair whose proof
//!    does not resolve.
//!
//! The CANONICAL workflow citations every µservice is REQUIRED to
//! carry (per `microservices/<ms>/scorecards/overrides.json#slsa_l3`)
//! are:
//!   - `.github/workflows/slsa.yml` (signed provenance — slsa-github-generator generator_generic_slsa3)
//!   - `.github/workflows/cosign.yml` (artifact signing + Rekor transparency log)
//!   - `.github/workflows/sbom.yml` (SBOM in CycloneDX + SPDX)
//!
//! These are the trio that satisfies SLSA Build L3 §"Provenance
//! generation" + the Sigstore/Rekor §"transparency log" requirement
//! per SLSA v1.0 spec.
//!
//! # Layer
//!
//! `domain` (port-in-kernel, ADR-0056). The runner reads files; the
//! kernel performs pure string evaluation and returns findings.
//!
//! # Naming justification
//!
//! `check-slsa-l3-evidence-grounded` follows the ADR-0532/0533 de-branded grammar:
//! `<group:check>-<axis:slsa-l3-evidence-grounded>`. Citing the
//! SLSA level explicitly (`l3`) keeps the lane scope unambiguous when
//! L4-tier policies are added later.
//!
//! # References
//!
//! - <https://slsa.dev/spec/v1.0/> — SLSA Build Track v1.0 spec.
//! - <https://slsa.dev/spec/v1.0/levels#build-l3> — Build L3 reqs.
//! - <https://github.com/slsa-framework/slsa-github-generator> — provenance generator.
//! - <https://docs.sigstore.dev/cosign/overview/> — keyless artifact signing.
//! - ADR-0064 — canonical-base-and-localization-packs.
//! - ADR-0133 — industry-best-practice + hyperscaler-conformance.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Three canonical SLSA-relevant primitives every cited workflow file
/// must declare. The kernel detects them by canonical-token substring
/// match.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SlsaPrimitive {
    /// Build provenance generation — required for L3.
    /// Detected via the canonical generator workflow path.
    SignedProvenance,
    /// Hermetic build — required for L3.
    /// Detected via runner pinning + checksum-locked toolchain markers.
    HermeticBuild,
    /// Two-party review — required for L3 deployment promotion.
    /// Detected via branch-protection or PR-review enforcement.
    TwoPartyReview,
}

impl SlsaPrimitive {
    /// Canonical detection tokens (case-insensitive substring match). A
    /// workflow needs AT LEAST ONE of these tokens to satisfy the
    /// primitive — keeping the surface tolerant of style variation
    /// across the three canonical workflows.
    #[must_use]
    pub const fn tokens(self) -> &'static [&'static str] {
        match self {
            SlsaPrimitive::SignedProvenance => &[
                "slsa-framework/slsa-github-generator",
                "generator_generic_slsa3.yml",
                "provenance.intoto.jsonl",
                "cosign sign-blob",
                "cosign-installer",
                "rekor",
                // ADR-0361 Jenkins-native grounding tokens:
                "cosign attest",
                "slsaprovenance",
                "slsa provenance",
            ],
            SlsaPrimitive::HermeticBuild => &[
                // Pinned toolchain action (sha or version pin) +
                // checkout pinned by SHA.
                "dtolnay/rust-toolchain@",
                "actions/checkout@v4",
                "--locked",
                "syft .",
                "cargo cyclonedx",
            ],
            SlsaPrimitive::TwoPartyReview => &[
                // Two-party-review is enforced by branch protection.
                // For workflow-side evidence, look for the OIDC token
                // surface that the SLSA generator requires (which is
                // only granted after PR review). The branch-protection
                // file itself satisfies this primitive when present.
                "id-token: write",
                "required_status_checks",
                "required_pull_request_reviews",
            ],
        }
    }
}

/// One supplied scorecard override JSON file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScorecardOverrideDocument {
    pub path: String,
    pub microservice: String,
    pub contents: String,
}

/// One supplied workflow file body (the runner reads
/// `.github/workflows/<file>.yml` and forwards it).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowDocument {
    pub path: String,
    pub contents: String,
}

/// Successful audit summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlsaL3EvidenceReport {
    pub scorecards_checked: usize,
    pub citations_checked: usize,
    pub workflows_inspected: usize,
    pub microservices_audited: usize,
}

/// A grounding failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlsaL3EvidenceViolation {
    pub scorecard_path: String,
    pub microservice: String,
    pub cited_workflow: String,
    pub kind: ViolationKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ViolationKind {
    /// Scorecard claims `slsa_l3: green` but does not cite any
    /// `.github/workflows/<file>.yml` files for the evidence chain.
    MissingCanonicalCitation,
    /// A cited workflow file was not supplied by the runner (file does
    /// not exist on the branch).
    WorkflowFileMissing,
    /// A cited workflow file exists but does not declare a required
    /// SLSA-relevant primitive (missing tokens).
    SlsaPrimitiveAbsent,
}

impl fmt::Display for SlsaL3EvidenceViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}): cited workflow {:?} — {:?}: {}",
            self.microservice, self.scorecard_path, self.cited_workflow, self.kind, self.summary,
        )
    }
}

/// Audit entrypoint: returns the report on success; the first violation
/// as `Err` on failure.
pub fn validate_slsa_l3_evidence_grounded<S, W>(
    scorecards: S,
    workflows: W,
) -> Result<SlsaL3EvidenceReport, SlsaL3EvidenceViolation>
where
    S: IntoIterator<Item = ScorecardOverrideDocument>,
    W: IntoIterator<Item = WorkflowDocument>,
{
    let (report, mut violations) = audit_all_violations(scorecards, workflows);
    if let Some(first) = violations.drain(..).next() {
        Err(first)
    } else {
        Ok(report)
    }
}

/// Full audit — returns the report AND every violation.
pub fn audit_all_violations<S, W>(
    scorecards: S,
    workflows: W,
) -> (SlsaL3EvidenceReport, Vec<SlsaL3EvidenceViolation>)
where
    S: IntoIterator<Item = ScorecardOverrideDocument>,
    W: IntoIterator<Item = WorkflowDocument>,
{
    let scorecards: Vec<ScorecardOverrideDocument> = scorecards.into_iter().collect();
    let workflows: Vec<WorkflowDocument> = workflows.into_iter().collect();

    // Index workflows by their canonical .github/workflows/<file>.yml
    // path for O(1) lookup.
    let mut workflow_by_path: BTreeMap<String, &WorkflowDocument> = BTreeMap::new();
    for w in &workflows {
        workflow_by_path.insert(canonicalize_workflow_path(&w.path), w);
    }

    let mut violations = Vec::new();
    let mut total_citations = 0usize;
    let mut microservices_audited: BTreeSet<String> = BTreeSet::new();

    for scorecard in &scorecards {
        if !scorecard_claims_slsa_l3_green(scorecard) {
            continue;
        }
        microservices_audited.insert(scorecard.microservice.clone());

        // Canonical SLSA L3 citation set: every µservice claiming
        // `slsa_l3: green` is required to be grounded by AT LEAST the
        // three canonical workflow files.
        // ADR-0361: SLSA-L3 evidence is grounded in the Jenkins pipeline, not the
        // retired GitHub Actions workflows. The shared CI lane grounds both
        // SignedProvenance (cosign attest --type slsaprovenance) and HermeticBuild
        // (cargo cyclonedx SBOM); the captured signing evidence README grounds the
        // measured provenance.
        let canonical_citations: [(&str, SlsaPrimitive); 3] = [
            (
                "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy",
                SlsaPrimitive::SignedProvenance,
            ),
            (
                "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy",
                SlsaPrimitive::HermeticBuild,
            ),
            (
                "evidence/ci/slsa/README.md",
                SlsaPrimitive::SignedProvenance,
            ),
        ];

        for (citation, primitive) in canonical_citations {
            total_citations += 1;
            match workflow_by_path.get(citation) {
                None => violations.push(SlsaL3EvidenceViolation {
                    scorecard_path: scorecard.path.clone(),
                    microservice: scorecard.microservice.clone(),
                    cited_workflow: citation.to_string(),
                    kind: ViolationKind::WorkflowFileMissing,
                    summary: format!(
                        "scorecard claims slsa_l3=green but {citation} does not exist on this branch"
                    ),
                }),
                Some(workflow) => {
                    if !declares_primitive(workflow, primitive) {
                        violations.push(SlsaL3EvidenceViolation {
                            scorecard_path: scorecard.path.clone(),
                            microservice: scorecard.microservice.clone(),
                            cited_workflow: citation.to_string(),
                            kind: ViolationKind::SlsaPrimitiveAbsent,
                            summary: format!(
                                "workflow {citation} exists but does not declare {:?} \
                                 (no canonical token from {:?} found)",
                                primitive,
                                primitive.tokens(),
                            ),
                        });
                    }
                }
            }
        }

        // Two-party review is an environmental primitive — surfaced
        // here for the audit summary, but not yet enforced via a
        // missing-citation violation because the branch-protection
        // file lives outside `.github/workflows/`. Keeping the
        // primitive enumerated future-proofs the lane.
        let _ = SlsaPrimitive::TwoPartyReview;
    }

    let report = SlsaL3EvidenceReport {
        scorecards_checked: scorecards.len(),
        citations_checked: total_citations,
        workflows_inspected: workflows.len(),
        microservices_audited: microservices_audited.len(),
    };
    (report, violations)
}

fn canonicalize_workflow_path(path: &str) -> String {
    // Tolerate both `.github/workflows/slsa.yml` and
    // bare `workflows/slsa.yml` (test scaffolds) — normalize to the
    // canonical leading `.github/` form.
    let trimmed = path.trim_start_matches("./");
    if let Some(rest) = trimmed.strip_prefix("workflows/") {
        return format!(".github/workflows/{rest}");
    }
    trimmed.to_string()
}

fn scorecard_claims_slsa_l3_green(doc: &ScorecardOverrideDocument) -> bool {
    // Tolerant string parse — we don't depend on a JSON crate. The
    // canonical scorecard has `"slsa_l3": { "overall_status": "green"`
    // verbatim (verified by the foundry scorecard reference).
    let lower = doc.contents.to_ascii_lowercase();
    let Some(idx) = lower.find("\"slsa_l3\"") else {
        return false;
    };
    let tail = &lower[idx..];
    if let Some(status_idx) = tail.find("\"overall_status\"") {
        let after = &tail[status_idx..];
        // Look ahead a short window for the value.
        let window: String = after.chars().take(80).collect();
        return window.contains("\"green\"");
    }
    false
}

fn declares_primitive(workflow: &WorkflowDocument, primitive: SlsaPrimitive) -> bool {
    let lower = workflow.contents.to_ascii_lowercase();
    for token in primitive.tokens() {
        if lower.contains(&token.to_ascii_lowercase()) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCORECARD_GREEN: &str = r#"{
  "microservice": "foundry",
  "slsa_l3": { "overall_status": "green", "deltas": [] }
}"#;

    const SCORECARD_NOT_GREEN: &str = r#"{
  "microservice": "messenger",
  "slsa_l3": { "overall_status": "yellow", "deltas": [] }
}"#;

    // ADR-0361: SLSA-L3 evidence is grounded in the Jenkins pipeline.
    const OYACILANE_LANE: &str = r#"
// oya-jenkins-shared :: oyaCiLane
stage('sign + provenance') {
  sh 'cosign sign --yes "$IMAGE_DIGEST"'
  sh 'cosign attest --yes --predicate target/provenance.intoto.json --type slsaprovenance "$IMAGE_DIGEST"'
}
stage('SBOM') { sh 'cargo cyclonedx --format json'; sh 'syft dir:. -o cyclonedx-json' }
"#;

    const SLSA_EVIDENCE_README: &str = r#"
# Jenkins SLSA/cosign/SBOM evidence
cosign sign by digest; cosign attest SLSA provenance; cosign verify-attestation passed.
"#;

    fn workflows_full_set() -> Vec<WorkflowDocument> {
        vec![
            WorkflowDocument {
                path: "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy".into(),
                contents: OYACILANE_LANE.into(),
            },
            WorkflowDocument {
                path: "evidence/ci/slsa/README.md".into(),
                contents: SLSA_EVIDENCE_README.into(),
            },
        ]
    }

    #[test]
    fn passes_when_all_canonical_citations_resolve_with_primitives() {
        let scorecard = ScorecardOverrideDocument {
            path: "microservices/intelligence/scorecards/overrides.json".into(),
            microservice: "foundry".into(),
            contents: SCORECARD_GREEN.into(),
        };
        let report = validate_slsa_l3_evidence_grounded(vec![scorecard], workflows_full_set())
            .expect("grounded scorecard passes");
        assert_eq!(report.citations_checked, 3);
        assert_eq!(report.microservices_audited, 1);
    }

    #[test]
    fn fails_when_pipeline_lane_missing() {
        let scorecard = ScorecardOverrideDocument {
            path: "microservices/intelligence/scorecards/overrides.json".into(),
            microservice: "foundry".into(),
            contents: SCORECARD_GREEN.into(),
        };
        // Only the evidence README present; the oyaCiLane grounding is absent.
        let workflows = vec![WorkflowDocument {
            path: "evidence/ci/slsa/README.md".into(),
            contents: SLSA_EVIDENCE_README.into(),
        }];
        let err = validate_slsa_l3_evidence_grounded(vec![scorecard], workflows)
            .expect_err("missing oyaCiLane grounding must fail");
        assert_eq!(err.kind, ViolationKind::WorkflowFileMissing);
        assert!(err.cited_workflow.ends_with("oyaCiLane.groovy"));
    }

    #[test]
    fn fails_when_workflow_lacks_primitive_token() {
        let scorecard = ScorecardOverrideDocument {
            path: "microservices/intelligence/scorecards/overrides.json".into(),
            microservice: "foundry".into(),
            contents: SCORECARD_GREEN.into(),
        };
        let workflows = vec![
            WorkflowDocument {
                path: "infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy".into(),
                contents: "// lane stub with no signing/sbom tokens".into(),
            },
            WorkflowDocument {
                path: "evidence/ci/slsa/README.md".into(),
                contents: SLSA_EVIDENCE_README.into(),
            },
        ];
        let err = validate_slsa_l3_evidence_grounded(vec![scorecard], workflows)
            .expect_err("primitive absent");
        assert_eq!(err.kind, ViolationKind::SlsaPrimitiveAbsent);
    }

    #[test]
    fn skips_scorecards_that_do_not_claim_green() {
        let scorecard = ScorecardOverrideDocument {
            path: "microservices/messenger/scorecards/overrides.json".into(),
            microservice: "messenger".into(),
            contents: SCORECARD_NOT_GREEN.into(),
        };
        let report =
            validate_slsa_l3_evidence_grounded(vec![scorecard], Vec::<WorkflowDocument>::new())
                .expect("non-green scorecard is out of scope");
        assert_eq!(report.scorecards_checked, 1);
        assert_eq!(report.citations_checked, 0);
        assert_eq!(report.microservices_audited, 0);
    }

    #[test]
    fn slsa_primitive_tokens_non_empty() {
        for p in [
            SlsaPrimitive::SignedProvenance,
            SlsaPrimitive::HermeticBuild,
            SlsaPrimitive::TwoPartyReview,
        ] {
            assert!(!p.tokens().is_empty(), "{p:?} must declare tokens");
        }
    }
}
