---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P15-supply-chain
impl_plan_id: IP-NEW-slsa-l3-evidence-grounded-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: ops-security + council-architecture
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, slsa-l3-evidence-grounded]
related_adrs:
  - ADR-0064
  - ADR-0133
related_crates:
  - oya-check-slsa-l3-evidence-grounded
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md -->

# IP-NEW: wire `oya-check-slsa-l3-evidence-grounded` into oya-dev-cli gate validate

## Intent

Activate the `oya-check-slsa-l3-evidence-grounded` kernel (authored
2026-05-18 at `crates/oya-check-slsa-l3-evidence-grounded/`) as a
fitness lane `oya gate validate slsa-l3-evidence-grounded`. The lane
reads every `microservices/<ms>/scorecards/overrides.json` and refuses
the build when a scorecard claims `slsa_l3: green` but its citation
chain does not resolve to existing `.github/workflows/<file>.yml`
files that declare the SLSA-relevant primitives.

The lane closes SEC-MAJ-01 (SLSA L3 evidence ungrounded). The
canonical citation set every green-claiming scorecard MUST resolve:
1. `.github/workflows/slsa.yml` — slsa-github-generator provenance.
2. `.github/workflows/cosign.yml` — Sigstore signing + Rekor anchor.
3. `.github/workflows/sbom.yml` — CycloneDX + SPDX SBOM.

All three exist in the repo today (verified at
`.github/workflows/{slsa,cosign,sbom}.yml`) and declare the canonical
primitives. The lane prevents silent regression of these workflows
without simultaneously downgrading the scorecard.

## ChangeSet boundary

- Add `oya-check-slsa-l3-evidence-grounded` as a workspace dep of
  `oya-dev-cli`.
- Author `crates/oya-dev-cli/src/slsa_l3_evidence_grounded_gate.rs`.
- Wire the new subcommand into `commands/gate/mod.rs`.
- Register in `AGGREGATED_VALIDATE_LANES`.
- Register in branch protection.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-dev-cli/Cargo.toml` | edit — add dep |
| `crates/oya-dev-cli/src/slsa_l3_evidence_grounded_gate.rs` | create — file-reading runner |
| `crates/oya-dev-cli/src/lib.rs` | edit — declare module |
| `crates/oya-dev-cli/src/commands/gate/mod.rs` | edit — add match arm |
| `crates/oya-governance-gate-catalog-domain/src/lib.rs` | edit — append `"slsa-l3-evidence-grounded"` |
| `.github/branch-protection.yaml` | edit — add to dev's required-status-checks |
| `microservices/governance/catalog/oya-check-slsa-l3-evidence-grounded.yaml` | create — catalog entry |

## Code shape

```rust
// crates/oya-dev-cli/src/slsa_l3_evidence_grounded_gate.rs
use std::fs;
use std::path::PathBuf;

use oya_check_slsa_l3_evidence_grounded::{
    ScorecardOverrideDocument, WorkflowDocument,
    validate_slsa_l3_evidence_grounded,
};

pub(crate) fn validate_slsa_l3_evidence_grounded_gate(
    microservices_dir: &str,
    workflows_dir: &str,
) -> Result<usize, String> {
    let mut scorecards = Vec::new();
    for ms_entry in fs::read_dir(microservices_dir)
        .map_err(|e| format!("read {microservices_dir}: {e}"))?
        .flatten()
    {
        let ms_path = ms_entry.path();
        let ms_name = ms_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let overrides = ms_path.join("scorecards").join("overrides.json");
        if overrides.exists() {
            let contents = fs::read_to_string(&overrides)
                .map_err(|e| format!("read {overrides:?}: {e}"))?;
            scorecards.push(ScorecardOverrideDocument {
                path: overrides.display().to_string(),
                microservice: ms_name,
                contents,
            });
        }
    }

    let mut workflows = Vec::new();
    for entry in fs::read_dir(workflows_dir)
        .map_err(|e| format!("read {workflows_dir}: {e}"))?
        .flatten()
    {
        let p = entry.path();
        if p.extension().and_then(|s| s.to_str()) == Some("yml") {
            let contents = fs::read_to_string(&p)
                .map_err(|e| format!("read {p:?}: {e}"))?;
            workflows.push(WorkflowDocument {
                path: p.display().to_string(),
                contents,
            });
        }
    }

    let report = validate_slsa_l3_evidence_grounded(scorecards, workflows)
        .map_err(|v| format!("SLSA L3 evidence ungrounded: {v}"))?;
    Ok(report.citations_checked)
}
```

## Acceptance gates

```bash
cargo check -p oya-dev-cli
cargo nextest run -p oya-check-slsa-l3-evidence-grounded
buck2 build //:quality-lane-registry-authority-check # lane=slsa-l3-evidence-grounded \
    --microservices-dir microservices --workflows-dir .github/workflows
buck2 build //:repo-hygiene-automation-check
```

## Halt conditions

- Lane fires on existing green scorecards → either fix the citation
  chain (add the missing workflow declaration) OR downgrade the
  scorecard to `yellow` BEFORE flipping the lane to BLOCKER.

## References

- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0133 — industry-best-practice + hyperscaler-conformance.
- SLSA v1.0 spec (slsa.dev/spec/v1.0).
- Sigstore documentation (docs.sigstore.dev).
- `.github/workflows/{slsa,cosign,sbom}.yml`.
- `crates/oya-check-slsa-l3-evidence-grounded`.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
