---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P22-m02-exit-gate
impl_plan_id: IP-001-flip-lanes-to-blocker
status: pending
owner: council-foundry
blocked_by:
  - impl_plan: P21-architecture-planes-green/IP-001
    reason: "All 9 planes must be verified at L4+ before flipping lanes to BLOCKER; a BLOCKER violation with unresolved plane issues would permanently block merges."
  - impl_plan: P21-architecture-planes-green/IP-002
    reason: "Wave integration DAG must be confirmed acyclic before M02 is declared complete."
acceptance_lanes:
  - cargo-check
  - cargo-nextest
  - cargo-deny
---

# IP-001-flip-lanes-to-blocker: Flip All 14 CI Fitness Lanes from --report-only to BLOCKER

## Intent

Removes `--report-only` flags from all 14 fitness lane invocations in
`.github/workflows/ci-fitness-lanes.yml`. From this commit forward, any violation
detected by any of the 14 lanes causes CI to fail hard, blocking merge. Simultaneously
produces `docs/architecture/m02-exit-checklist.md` confirming each gate was verified
before the flip.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `.github/workflows/ci-fitness-lanes.yml` | update | Remove `--report-only` from all 14 lane invocations; set `exit-on-violation: true` where applicable |
| `docs/architecture/m02-exit-checklist.md` | create | Per-gate evidence checklist: gate name + evidence reference + status |

---

## Code Shape

### `.github/workflows/ci-fitness-lanes.yml` (diff shape)

Before (P20 state — all lanes in report-only mode):
```yaml
- name: statelessness-check
  run: cargo run -p oya-check-statelessness -- --workspace --report-only

- name: shardability-check
  run: cargo run -p oya-check-shardability -- --migrations-dir migrations/ --report-only

- name: doc-coverage-check                         # LEAN-A5; ADR-0063
  run: cargo run -p oya-check-documentation -- --workspace --report-only

- name: architecture-dependency-direction
  run: cargo run -p oya-check-architecture -- dependency-direction --workspace --report-only

- name: architecture-layer-correctness
  run: cargo run -p oya-check-architecture -- layer-correctness --workspace --report-only

- name: architecture-lib-name-parity
  run: cargo run -p oya-check-architecture -- lib-name-parity --workspace --report-only

- name: architecture-port-location
  run: cargo run -p oya-check-architecture -- port-location --workspace --report-only

- name: architecture-cross-product-refusal
  run: cargo run -p oya-check-architecture -- cross-product-refusal --workspace --report-only

- name: architecture-composition-root-only
  run: cargo run -p oya-check-architecture -- composition-root-only --workspace --report-only

- name: architecture-sdk-kernel-only
  run: cargo run -p oya-check-architecture -- sdk-kernel-only --workspace --report-only

- name: architecture-canonical-base-neutrality     # ADR-0064 §8
  run: cargo run -p oya-check-architecture -- canonical-base-neutrality --workspace --report-only

- name: architecture-cross-pack-refusal            # ADR-0064 §7
  run: cargo run -p oya-check-architecture -- cross-pack-refusal --workspace --report-only

- name: perf-budget-check
  run: cargo run -p oya-check-perf-budget -- --workspace --report-only

- name: benchmark-check
  run: cargo run -p oya-check-benchmark -- --workspace --report-only
```

After (P22 state — BLOCKER):
```yaml
- name: statelessness-check
  run: cargo run -p oya-check-statelessness -- --workspace

- name: shardability-check
  run: cargo run -p oya-check-shardability -- --migrations-dir migrations/

- name: doc-coverage-check                         # LEAN-A5; ADR-0063; --blocker required because the CLI exits 1 only with --blocker
  run: cargo run -p oya-check-documentation -- --workspace --blocker

- name: architecture-dependency-direction
  run: cargo run -p oya-check-architecture -- dependency-direction --workspace

- name: architecture-layer-correctness
  run: cargo run -p oya-check-architecture -- layer-correctness --workspace

- name: architecture-lib-name-parity
  run: cargo run -p oya-check-architecture -- lib-name-parity --workspace

- name: architecture-port-location
  run: cargo run -p oya-check-architecture -- port-location --workspace

- name: architecture-cross-product-refusal
  run: cargo run -p oya-check-architecture -- cross-product-refusal --workspace

- name: architecture-composition-root-only
  run: cargo run -p oya-check-architecture -- composition-root-only --workspace

- name: architecture-sdk-kernel-only
  run: cargo run -p oya-check-architecture -- sdk-kernel-only --workspace

- name: architecture-canonical-base-neutrality     # ADR-0064 §8 — BLOCKER
  run: cargo run -p oya-check-architecture -- canonical-base-neutrality --workspace

- name: architecture-cross-pack-refusal            # ADR-0064 §7 — BLOCKER
  run: cargo run -p oya-check-architecture -- cross-pack-refusal --workspace

- name: perf-budget-check
  run: cargo run -p oya-check-perf-budget -- --workspace

- name: benchmark-check
  run: cargo run -p oya-check-benchmark -- --workspace
```

The remaining 3 standard lanes (`cargo-check`, `cargo-nextest`, `cargo-deny`) are
already BLOCKER from P20 and require no change. doc-coverage uses explicit
`--blocker` flag because the CLI exits 1 only when `--blocker` is set
(`crates/oya-check-documentation/src/main.rs:38`); removing `--report-only`
alone leaves it permissive. canonical-base-neutrality + cross-pack-refusal
are added per ADR-0064 §7 §8 enforcement.

### `docs/architecture/m02-exit-checklist.md` (full content)

```markdown
# M02 Exit Gate Checklist

**Date:** 2026-05-13
**Milestone:** M02-substrate
**Phase:** P22-m02-exit-gate
**Assessor:** council-architecture

## Pre-conditions

| Gate | Evidence | Status |
|---|---|---|
| All 9 planes L4+ | `docs/architecture/plane-verification-M02.md` | ✓ |
| Wave DAG acyclic | `docs/architecture/wave-integration-report-M02.md` | ✓ |
| 14 lanes exit 0 (report-only) | P21 CI run log | ✓ |
| cargo check clean | workspace build | ✓ |
| cargo nextest 0 failures | workspace test run | ✓ |
| cargo deny clean | dependency audit | ✓ |

## Lane Flip (--report-only → BLOCKER)

| Lane | Binary | Pre-flip exit | Post-flip exit | Violations |
|---|---|---|---|---|
| statelessness | oya-check-statelessness | 0 | 0 | 0 |
| shardability | oya-check-shardability | 0 | 0 | 0 |
| dependency-direction | oya-check-architecture | 0 | 0 | 0 |
| layer-correctness | oya-check-architecture | 0 | 0 | 0 |
| lib-name-parity | oya-check-architecture | 0 | 0 | 0 |
| port-location | oya-check-architecture | 0 | 0 | 0 |
| cross-product-refusal | oya-check-architecture | 0 | 0 | 0 |
| composition-root-only | oya-check-architecture | 0 | 0 | 0 |
| sdk-kernel-only | oya-check-architecture | 0 | 0 | 0 |
| perf-budget | oya-check-perf-budget | 0 | 0 | 0 |
| benchmark | oya-check-benchmark | 0 | 0 | 0 |
| cargo-check | cargo | 0 | 0 | — |
| cargo-nextest | cargo nextest | 0 | 0 | — |
| cargo-deny | cargo deny | 0 | 0 | — |

## App Shell Deployment (Stage 0)

| Step | Command | Expected | Status |
|---|---|---|---|
| Image build | `docker build --platform linux/arm64 -t oya-application-shell:m02 .` | exit 0 | ✓ |
| OKE deploy | `kubectl apply -f deployments/stage0/oya-application-shell.yaml` | applied | ✓ |
| Rollout ready | `kubectl rollout status deployment/oya-application-shell -n oyatie-stage0` | ready | ✓ |
| Health check | `curl -f https://stage0.oyatie.internal/health` | HTTP 200 | ✓ |
| SSO round-trip | browser PKCE flow via SSO hub | token issued | ✓ |
| Product catalog | TenantProductRegistry resolves ≥1 product | resolved | ✓ |

## Sibling Team Smoke Test

| Step | Tool | Expected | Status |
|---|---|---|---|
| grit claim | `grit claim --agent sibling-test --intent "tracer feature" --ttl 1800 tracer/src/main.rs::tracer_feature` | lock acquired | ✓ |
| scaffold product | follow `docs/runbooks/sibling-team-onboarding.md` §3 | crate compiles | ✓ |
| ship feature | `cargo nextest run -p tracer --test integration` | exit 0 | ✓ |
| grit done | `grit done --agent sibling-test` | exit 0; no conflict | ✓ |

## M02 Declaration

All gates passed. M02-substrate milestone is **COMPLETE**.
Next milestone: **M03** (Citus deployment + mTLS Istio + Grafana dashboards + active-active prep).
```

---

## Acceptance Gates

```bash
# Verify BLOCKER mode — each must exit 0 (means 0 violations when running against full workspace)
cargo run -p oya-check-statelessness -- --workspace                          # exit 0
cargo run -p oya-check-shardability -- --migrations-dir migrations/          # exit 0
cargo run -p oya-check-architecture -- dependency-direction --workspace      # exit 0
cargo run -p oya-check-architecture -- layer-correctness --workspace         # exit 0
cargo run -p oya-check-architecture -- lib-name-parity --workspace           # exit 0
cargo run -p oya-check-architecture -- port-location --workspace             # exit 0
cargo run -p oya-check-architecture -- cross-product-refusal --workspace     # exit 0
cargo run -p oya-check-architecture -- composition-root-only --workspace     # exit 0
cargo run -p oya-check-architecture -- sdk-kernel-only --workspace           # exit 0
cargo run -p oya-check-perf-budget -- --workspace                            # exit 0
cargo run -p oya-check-benchmark -- --workspace                              # exit 0
cargo check --workspace --all-features                                        # exit 0
cargo nextest run --workspace --all-features                                  # exit 0; 0 failures
cargo deny check                                                              # exit 0
oya gate validate planes --all                                                # exit 0
oya gate validate wave-integration --milestone M02                            # exit 0; no cycles
```

---

## Test Plan

No new test code. This IP modifies GitHub Actions YAML and produces a documentation artifact.
The "test" is CI itself: the first PR merged after this IP must have all 14 lanes green in
BLOCKER mode with no violations.

| Verification step | Tool | Expected |
|---|---|---|
| statelessness BLOCKER | oya-check-statelessness (no --report-only) | exit 0 |
| shardability BLOCKER | oya-check-shardability (no --report-only) | exit 0 |
| architecture checks BLOCKER | oya-check-architecture (7 sub-cmds, no --report-only) | exit 0 |
| perf-budget BLOCKER | oya-check-perf-budget (no --report-only) | exit 0 |
| benchmark BLOCKER | oya-check-benchmark (no --report-only) | exit 0 |
| CI green | GitHub Actions ci-fitness-lanes workflow | all jobs green |

---

## Sibling-Team Onboarding Runbook Shape

`docs/runbooks/sibling-team-onboarding.md` — structure:

```markdown
# Sibling Team Onboarding: Shipping a Product Vertical on M02 Substrate

## Prerequisites
- grit CLI installed (`rtk-ai/grit`)
- Rust stable toolchain + cargo nextest
- Access to oyatie workspace

## Step 1: Claim your symbols
```bash
grit claim \
  --agent <your-team-id> \
  --intent "<product-vertical>: <feature-description>" \
  --ttl 3600 \
  crates/oya-<microservice>-<bc>-<layer>/src/...
```

## Step 2: Scaffold your product crate
BNF v4.1 name: `oya-<microservice>(-<bc-tokens>)?-<layer>`
Layer options: kernel | domain | application | adapter | rest | grpc | worker | cli | sdk

Kernel crate declares port traits (Send + Sync). No impls.
Domain crate: business logic. No infra deps.
Adapter crate: implements port traits. Postgres/HTTP/gRPC allowed here.
Application crate: orchestrates domain + ports via use-case structs.

Workspace entry in Cargo.toml:
```toml
[workspace.members]
members = [
  # ... existing ...
  "crates/oya-<your-microservice>-kernel",
  "crates/oya-<your-microservice>-domain",
  "crates/oya-<your-microservice>-adapter",
]
```

## Step 3: Cross-product actions — use WorkflowBridgePort
All actions that cross product boundaries go through Workflow.
Import `oya-workflow-engine-kernel` and call `WorkflowBridgePort::submit_action()`.
NEVER import another product's kernel directly for actions.

## Step 4: Cross-product data — use Ontology ObjectStore
All reads/writes of shared entities go through Ontology.
Import `oya-ontology-entity-kernel` and use `ObjectStore`, `LinkStore`, `ActionStore`.
NEVER query another product's schema directly.

## Step 5: Run all CI lanes locally before PR
```bash
cargo check --workspace --all-features
cargo nextest run --workspace --all-features
cargo run -p oya-check-architecture -- cross-product-refusal --workspace
cargo run -p oya-check-architecture -- dependency-direction --workspace
cargo run -p oya-check-statelessness -- --workspace
cargo deny check
```

## Step 6: Ship
```bash
grit done --agent <your-team-id>
```
CI runs all 14 lanes in BLOCKER mode. All must exit 0. PR merges.
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-foundry \
  --intent "IP-001-flip-lanes-to-blocker: flip 14 lanes BLOCKER; m02-exit-checklist produced" \
  --ttl 3600 \
  .github/workflows/ci-fitness-lanes.yml::fitness-lanes-blocker \
  docs/architecture/m02-exit-checklist.md::M02ExitChecklist \
  docs/runbooks/sibling-team-onboarding.md::SiblingOnboardingRunbook
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-flip-lanes-to-blocker merged; all 14 CI fitness lanes now BLOCKER mode; m02-exit-checklist.md produced; sibling-team-onboarding.md produced; M02-substrate COMPLETE; next: M03" \
  -i high \
  -k "M02,P22,IP-001,lanes-blocker,milestone-complete"
```

---

## Halt Conditions

1. Any lane reports violations in BLOCKER mode — do NOT merge; open a remediation issue
   against the owning phase team; the violation must be fixed before lane flip proceeds.
2. App shell deployment fails on OCI ARM64 — escalate to P19 team (cloud-tenancy or
   application phase); do not fake the health check.
3. Sibling team smoke test fails due to grit conflict — indicates a symbol-lock sequencing
   error; escalate to council-architecture; do not skip grit.

---

## Next IP Pointer

M02 is complete after this IP. Next milestone: `M03-citus-mTLS-observability`.

---

## Cross-References

- Phase spec: `phase-spec.md`
- P20-ci-lanes-operational/IP-001..IP-004 (lane binaries)
- P21-architecture-planes-green/IP-001 (plane verification evidence)
- ADR-0056 v4.1 (BNF + check-namespace), ADR-0062 (quality/perf bar), ADR-0117 (OCI stages)
- Bominal ADR-0232 (wave integration framework)
