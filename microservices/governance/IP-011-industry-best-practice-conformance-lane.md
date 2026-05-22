---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-011-industry-best-practice-conformance-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: council-architecture
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, industry-best-practice-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-011: oya-check-industry-best-practice-conformance lane (BLOCKER on dev)

## Intent

Author the new BLOCKER lane that implements ADR-0133's 6-axis program. Reads `/specs/industry-best-practice-conformance.json`. Refuses unaudited new artifacts.

## ChangeSet boundary

New crate `microservices/governance/src/crates/oya-check-industry-best-practice-conformance/` + activation in `.github/branch-protection.yaml`.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-check-industry-best-practice-conformance/Cargo.toml` | create |
| `…/src/main.rs` | create — CLI entry point invokable via `oya gate validate industry-best-practice-conformance` |
| `…/src/per_axis/{pipeline,directory,naming,standards,practices,policies}.rs` | create — one module per axis |
| `…/rules/2026-q2.toml` | create — first quarterly pin |
| `.github/branch-protection.yaml` | edit — add to `required_status_checks` on `dev` |
| `/specs/industry-best-practice-conformance.json` | create — initial pin map |
| `/specs/hyperscaler-gates.json` | edit — register `HG-GOV` gate |
| `microservices/governance/catalog/oya-check-industry-best-practice-conformance.yaml` | create |

## Code Shape

```rust
// src/main.rs
use clap::Parser;
use oya_governance_policy_engine_usecase::evaluate_six_axes;

#[derive(Parser)]
struct Args {
    #[arg(long)] microservice: Option<String>,
    #[arg(long)] all: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let repo = TomlRulepackReader::open("/specs/industry-best-practice-conformance.json")?;
    let input = workspace_to_rule_input(args.microservice)?;
    let postures = evaluate_six_axes(&repo, &input).await?;

    let blocker_count: usize = postures.iter().map(|p| p.blocker_count()).sum();
    if blocker_count > 0 {
        println!("BLOCKER: {} axis-conformance violations", blocker_count);
        std::process::exit(1);
    }
    println!("PASS: 6-axis conformance green");
    Ok(())
}
```

```rust
// src/per_axis/pipeline.rs
use oya_governance_policy_engine_kernel::*;

pub fn evaluate_pipeline_axis(input: &RuleInput) -> Vec<Finding> {
    let mut findings = Vec::new();

    // SLSA Build L3: every release has provenance
    if !input.has_slsa_build_provenance() {
        findings.push(Finding::new("pipeline.slsa-build-l3", BLOCKER, "SLSA v1.0 Build L3"));
    }

    // NIST SSDF PS.2: software release integrity verification
    if !input.has_release_integrity_check() {
        findings.push(Finding::new("pipeline.nist-ssdf-ps2", BLOCKER, "NIST SSDF SP 800-218 PS.2"));
    }

    findings
}
```

## Acceptance Gates

```bash
cargo check -p oya-check-industry-best-practice-conformance
cargo nextest run -p oya-check-industry-best-practice-conformance
cargo run -p oya-dev-cli -- gate validate industry-best-practice-conformance --microservice governance
# Self-application: lane refuses governance µservice itself if governance lacks
# any of the 6-axis baselines. Should PASS at M01.
cargo run -p oya-dev-cli -- gate validate industry-best-practice-conformance --microservice observability
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_pipeline_axis_evaluator` | SLSA + NIST findings |
| `test_directory_axis_evaluator` | per-microservice-layout findings |
| `test_self_application_passes` | governance µservice itself green |
| `test_first_run_amnesty` | `legacy-grandfathered` severity for pre-existing violations per ADR-0133 §"Operational" |

## Halt Conditions

- Self-application fails on governance → fix in same PR OR use synthetic-probe fallback per F-13.
- New BLOCKER finding on every µservice → first-run amnesty did not apply correctly; halt and fix.

## Next IP

[`IP-012-per-microservice-layout-lane.md`](IP-012-per-microservice-layout-lane.md)

## References

- ADR-0133 (industry-best-practice + hyperscaler-grade conformance program).
- `microservices/governance/competitor-parity-matrix.md`.
- `microservices/governance/policy/lane-execution.md` Invariant 10 (self-application).

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
