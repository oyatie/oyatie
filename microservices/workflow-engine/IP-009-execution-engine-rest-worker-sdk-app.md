---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate-ready
phase: P01-durable-execution-substrate
impl_plan_id: IP-009-execution-engine-rest-worker-sdk-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-workflow-engine-execution-engine-{rest,worker,sdk,app}

## Intent

Complete execution-engine BC. The engine worker binary; rest surface for operator actions; SDK for tenant programmatic control; app composition root.

## ChangeSet boundary

4 new crates.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `src/crates/oya-workflow-engine-execution-engine-rest/{Cargo.toml,src/{lib,routes,middleware}.rs}` | create | HTTP routes per `contracts/openapi/workflow-engine.yaml`; OIDC + Cedar |
| `src/crates/oya-workflow-engine-execution-engine-worker/{Cargo.toml,src/{lib,run_dispatcher,step_executor,resume_throttle}.rs}` | create | Long-lived step-dispatch worker; HA via Valkey lease; resume-rate-limit at cold-start |
| `src/crates/oya-workflow-engine-execution-engine-sdk/{Cargo.toml,src/{lib,client}.rs}` | create | Tenant SDK |
| `src/crates/oya-workflow-engine-execution-engine-app/{Cargo.toml,src/main.rs}` | create | Composition root binary |
| `microservices/workflow-engine/catalog/oya-workflow-engine-execution-engine-*.yaml` | create | 4 catalog rows |

## Acceptance Gates

```bash
cargo nextest run -p oya-workflow-engine-execution-engine-rest --all-features
cargo nextest run -p oya-workflow-engine-execution-engine-worker --all-features
cargo nextest run -p oya-workflow-engine-execution-engine-sdk --all-features
cargo nextest run -p oya-workflow-engine-execution-engine-app --all-features
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_start_workflow_run_happy_path` | run started, first step executed |
| `test_cancel_workflow_run_2person_rule_production` | production cancel refuses single-signer |
| `test_resume_rate_limit_at_coldstart` | resume-rate ≤ 100 runs/s/worker enforced |
| `test_pod_eviction_run_resumption` (e2e) | run resumes on different worker; identical step sequence |
| `test_sla_timer_escalation_on_breach` | timer fires; no false positives |

## Next IP

[`IP-010-replay-debugger-backend-kernel-domain.md`](IP-010-replay-debugger-backend-kernel-domain.md)

## References

- PRD AC-03, AC-04, AC-05, AC-10
- `runbooks/durable-execution-restart.md`
- `contracts/openapi/workflow-engine.yaml`
