---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-006-iac-validator-kernel-domain-usecase
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness, check-cedar-fragment-coverage]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006: cloud-iac-iac-validator-{kernel,domain,usecase,api,adapter}

## Intent

Scaffold the iac-validator BC's core stack: kernel (port traits + entities) + domain (plan-diff math + drift comparison) + usecase (plan-preview orchestrator + Cedar policy evaluator) + api (typed I/O) + adapter (live-cluster API client + Cedar evaluator).

## ChangeSet boundary

Five new crates per ADR-0105: `-kernel`, `-domain`, `-usecase`, `-api`, `-adapter`. Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-iac/src/crates/cloud-iac-iac-validator-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs,src/errors.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-validator-domain/{Cargo.toml,src/lib.rs,src/plan_diff.rs,src/drift_compare.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-validator-usecase/{Cargo.toml,src/lib.rs,src/plan_preview.rs,src/cedar_evaluator.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-validator-api/{Cargo.toml,src/lib.rs}` | create |
| `microservices/cloud-iac/src/crates/cloud-iac-iac-validator-adapter/{Cargo.toml,src/lib.rs,src/k8s_client.rs,src/cedar_engine.rs}` | create |
| `microservices/cloud-iac/catalog/cloud-iac-iac-validator-*.yaml` | create (5 rows) |

## Code Shape

```rust
// kernel/src/entities.rs
pub struct PlanPreview {
    #[data_class(INTERNAL_ONLY)] pub microservice: String,
    #[data_class(AUDIT)]         pub plan_id: String,
    #[data_class(INTERNAL_ONLY)] pub summary: PlanSummary,
    #[data_class(BEHAVIORAL_TENANT_PRODUCT)] pub changes: Vec<PlanChange>,
    #[data_class(AUDIT)]         pub validation_verdict: ValidationVerdict,
}

pub enum ValidationVerdict { Allow, Refuse(String), Advisory(String) }
```

```rust
// usecase/src/plan_preview.rs
pub struct PlanPreviewOrchestrator<P, D, C> {
    plan_computer: P,
    drift_differ: D,
    cedar_evaluator: C,
}

impl<P, D, C> PlanPreviewOrchestrator<P, D, C>
where P: PlanComputer, D: DriftDiffer, C: PolicyEvaluator {
    pub async fn preview(&self, microservice: &str, sha: &str, pack: &str, env: Environment) -> Result<PlanPreview, ValidationError> {
        let plan = self.plan_computer.plan(microservice, sha, pack, env).await?;
        let verdict = self.cedar_evaluator.evaluate(&plan).await?;
        Ok(PlanPreview { microservice, plan_id, summary: ..., changes: ..., validation_verdict: verdict })
    }
}
```

```cedar
// usecase/src/cedar_evaluator.rs invokes policies in microservices/cloud-iac/policy/*.cedar
// Default deny + per-µservice apply scope + reserved scope checks.
```

## Acceptance Gates

```bash
cargo check -p cloud-iac-iac-validator-kernel -p cloud-iac-iac-validator-domain -p cloud-iac-iac-validator-usecase -p cloud-iac-iac-validator-api -p cloud-iac-iac-validator-adapter --all-features
cargo nextest run -p cloud-iac-iac-validator-kernel -p cloud-iac-iac-validator-domain -p cloud-iac-iac-validator-usecase -p cloud-iac-iac-validator-api -p cloud-iac-iac-validator-adapter --all-features
cloud-ci/ci governance gate `layer-correctness` for --microservice cloud-iac is green in the branch-protected `presubmit` context
cloud-ci/ci governance gate `cedar-fragment-coverage` for --microservice cloud-iac is green in the branch-protected `presubmit` context
```

## Test Plan

| Test | Layer | Verifies |
|---|---|---|
| `test_plan_diff_pure` | domain | plan-diff math correctness |
| `test_drift_compare_property` | domain | drift comparison is deterministic |
| `test_plan_preview_cedar_refuse` | usecase | Cedar refuse → ValidationVerdict::Refuse |
| `test_plan_preview_cross_microservice_refused` | usecase | apply-scope violation → Refuse |
| `test_cedar_engine_fuzz` | adapter | fuzz Cedar inputs; no panics |
| `integration_k8s_dry_run` | adapter | against kind cluster |

## Halt Conditions

- Plan-diff non-deterministic — refactor.
- Cedar refuse rationale not surfaced in audit log — fix.

## Next IP

[`IP-007-iac-applier-kernel-domain-usecase.md`](IP-007-iac-applier-kernel-domain-usecase.md)

## References

- ADR-0105; ADR-0140 (retired per ADR-0145) (Cedar).
- PRD §"Bounded Contexts" iac-validator BC.
- Cedar v4 — `docs.cedarpolicy.com/`.
