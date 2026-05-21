---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-NEW-eu-ai-act-annex-iii-refusal-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: council-privacy + council-architecture
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, eu-ai-act-annex-iii-refusal]
related_adrs:
  - ADR-0064
  - ADR-0133
  - ADR-0140 (retired per ADR-0145)
  - ADR-0144
related_crates:
  - oya-check-eu-ai-act-annex-iii-refusal
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md -->

# IP-NEW: wire `oya-check-eu-ai-act-annex-iii-refusal` into oya-dev-cli gate validate

## Intent

Activate the `oya-check-eu-ai-act-annex-iii-refusal` kernel
(authored 2026-05-18 at
`crates/oya-check-eu-ai-act-annex-iii-refusal/`) as a fitness lane
`oya gate validate eu-ai-act-annex-iii-refusal`. The lane reads every
`microservices/<ms>/capabilities/T2-auto.yaml` and every
`microservices/<ms>/policy/*.cedar` and refuses the build when a
declared Annex III refusal claim has no matching `forbid` rule.

## ChangeSet boundary

- Add `oya-check-eu-ai-act-annex-iii-refusal` as a workspace dep of
  `oya-dev-cli`.
- Author `crates/oya-dev-cli/src/eu_ai_act_annex_iii_refusal_gate.rs`
  with the runner that reads files and forwards to the kernel.
- Wire the new subcommand into `crates/oya-dev-cli/src/commands/gate/mod.rs`.
- Register the lane in `AGGREGATED_VALIDATE_LANES` in
  `crates/oya-foundry-gate-catalog-domain/src/lib.rs`.
- Register the lane in `.github/branch-protection.yaml` as a required
  status check on `dev`.

## Concrete file targets

| Path | Action |
|---|---|
| `crates/oya-dev-cli/Cargo.toml` | edit — add dep on `oya-check-eu-ai-act-annex-iii-refusal` |
| `crates/oya-dev-cli/src/eu_ai_act_annex_iii_refusal_gate.rs` | create — file-reading runner |
| `crates/oya-dev-cli/src/lib.rs` | edit — declare module |
| `crates/oya-dev-cli/src/commands/gate/mod.rs` | edit — add `(Some("validate"), Some("eu-ai-act-annex-iii-refusal"))` arm |
| `crates/oya-foundry-gate-catalog-domain/src/lib.rs` | edit — append `"eu-ai-act-annex-iii-refusal"` to `AGGREGATED_VALIDATE_LANES` |
| `.github/branch-protection.yaml` | edit — add to dev's `required_status_checks` |
| `microservices/governance/catalog/oya-check-eu-ai-act-annex-iii-refusal.yaml` | create — catalog entry |

## Code shape

```rust
// crates/oya-dev-cli/src/eu_ai_act_annex_iii_refusal_gate.rs
use std::fs;
use std::path::PathBuf;

use oya_check_eu_ai_act_annex_iii_refusal::{
    CapabilityDocument, CedarPolicyDocument, validate_annex_iii_refusals,
};

pub(crate) fn validate_annex_iii_refusal_gate(
    microservices_dir: &str,
) -> Result<usize, String> {
    let mut capabilities = Vec::new();
    let mut cedar_fragments = Vec::new();
    for ms_entry in fs::read_dir(microservices_dir)
        .map_err(|e| format!("read {microservices_dir}: {e}"))?
        .flatten()
    {
        let ms_path = ms_entry.path();
        let ms_name = ms_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let t2_auto = ms_path.join("capabilities").join("T2-auto.yaml");
        if t2_auto.exists() {
            let contents = fs::read_to_string(&t2_auto)
                .map_err(|e| format!("read {t2_auto:?}: {e}"))?;
            capabilities.push(CapabilityDocument {
                path: t2_auto.display().to_string(),
                microservice: ms_name.clone(),
                contents,
            });
        }
        let policy_dir = ms_path.join("policy");
        if policy_dir.exists() {
            for entry in fs::read_dir(&policy_dir).map_err(|e| format!("read {policy_dir:?}: {e}"))?
                .flatten()
            {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("cedar") {
                    let contents = fs::read_to_string(&p)
                        .map_err(|e| format!("read {p:?}: {e}"))?;
                    cedar_fragments.push(CedarPolicyDocument {
                        path: p.display().to_string(),
                        microservice: ms_name.clone(),
                        contents,
                    });
                }
            }
        }
    }
    let report = validate_annex_iii_refusals(capabilities, cedar_fragments)
        .map_err(|v| format!("EU AI Act Annex III refusal gap: {v}"))?;
    Ok(report.claims_found)
}
```

## Acceptance gates

```bash
cargo check -p oya-dev-cli
cargo nextest run -p oya-check-eu-ai-act-annex-iii-refusal
cargo run -p oya-dev-cli -- gate validate eu-ai-act-annex-iii-refusal
cargo run -p oya-dev-cli -- gate run-all   # aggregated lane includes new gate
```

## Halt conditions

- Lane fires on existing T2-auto.yaml files that lack matching Cedar
  fragments → file the gap as a per-µservice IP **before** flipping
  the lane to BLOCKER. First-run amnesty per ADR-0133 §"Operational."

## References

- ADR-0064 — canonical-base-and-localization-packs.
- ADR-0133 — industry-best-practice + hyperscaler-conformance.
- ADR-0140 — Cedar policy enforcement substrate.
- ADR-0144 — EU AI Act graduated risk-tier model (generalisation target).
- EU AI Act Regulation (EU) 2024/1689 Annex III.
- `crates/oya-check-eu-ai-act-annex-iii-refusal`.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
