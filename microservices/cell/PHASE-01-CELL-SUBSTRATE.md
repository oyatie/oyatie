---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-cell-substrate
status: Active
entry_gate: |
  Bominal ADR-0009 + ADR-0019 inherited; ADR-0117 + ADR-0131 accepted; tenancy µservice's tenant_id taxonomy
  published; cargo workspace ready to accept the 45 new crates under microservices/cell/src/crates/.
exit_gate: |
  All 15 IPs merged; oya-cell-boundary CI lane present in .github/branch-protection.yaml required_status_checks
  on dev and staging; cargo nextest run --workspace exits 0; oya gate validate per-microservice-layout --microservice cell
  exits 0; HG-CELL gate in /specs/hyperscaler-gates.json registers green; tenant-migration drill passes ≤ 10min p99.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion (observability gate must be live so cell SLOs are authored against it)
    reason: cell µservice publishes its own OpenSLO manifests at microservices/cell/slos/
owner_team: axis-cell-substrate
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_specs: [/specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-cell-substrate: Land the cell substrate end-to-end

## Purpose

This phase ships the full cell substrate per Bominal ADR-0009 + ADR-0019: the cell-registry, tenant-assignment, scheduler, lifecycle-manager, and host-pool BCs; the cell-boundary CI lane; the Kubernetes Cluster API + Postgres + scheduler IaC. It is the precondition for every other oyatie workload µservice to enforce hard tenant isolation.

This phase advances master-plan principles:

- Hyperscaler-grade tenant isolation (cell == GKE-Autopilot-tier isolation; no shared-process tenancy).
- No silent regression (cell-boundary lane catches cross-cell coupling at PR time).
- Per-microservice flat layout (this phase is native under ADR-0131).
- Long-term-right > short-term cost (binpack-correct scheduler over naive "first cell" placement).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `cell` | `cell-registry`, `tenant-assignment`, `scheduler`, `lifecycle-manager`, `host-pool` | All under `microservices/cell/` per ADR-0131 | `oya-cell-{cell-registry,tenant-assignment,scheduler,lifecycle-manager,host-pool}-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-k8s,rest,worker,sdk,app}` |

Plus repo-wide artifacts:

- `.github/branch-protection.yaml` — add `oya-cell-boundary` to required_status_checks.
- `registry/cell-assignment.jsonl` (NEW) — append-only ledger.
- `Cargo.toml` (workspace) — register the 45 new crates under `microservices/cell/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-CELL gate.

### Out-of-scope

- Migration of legacy `oya-cell-domain` and `oya-cloud-cell-app` crates — folded into the new BCs by separate IP-MIGR-001 (parallel branch).
- Federated cross-pack cell topology — scheduled-for-distinct-tracked-work to subsequent-to-M01-completion ADR per PRD §"Horizontal Scalability".
- Cell-native autoscaling for individual workload µservices' compute — owned by `observability` + workload µservices, not by `cell`.

## Implementation Plans

Ordered list. Each IP is one ChangeSet under this phase folder.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-host-pool-iac.md`](IP-001-host-pool-iac.md) | Helm: warm K8s node pool per pack via Cluster API | pending | cloud-k8s | — |
| [`IP-002-cell-registry-postgres-schema.md`](IP-002-cell-registry-postgres-schema.md) | Postgres schema + per-pack shard topology + Helm chart | pending | axis-cell-substrate | — |
| [`IP-003-cell-registry-kernel.md`](IP-003-cell-registry-kernel.md) | `oya-cell-registry-kernel`: ports, entities, errors | pending | axis-cell-substrate | IP-002 |
| [`IP-004-cell-registry-domain-usecase.md`](IP-004-cell-registry-domain-usecase.md) | cell-state-machine domain + read/write use cases | pending | axis-cell-substrate | IP-003 |
| [`IP-005-cell-registry-adapter-postgres-rest-sdk-app.md`](IP-005-cell-registry-adapter-postgres-rest-sdk-app.md) | registry full stack: pg adapter + REST + SDK + app | pending | axis-cell-substrate | IP-004 |
| [`IP-006-cell-boundary-gate-lane.md`](IP-006-cell-boundary-gate-lane.md) | new BLOCKER CI lane refuses cross-cell coupling | pending | axis-foundry | IP-005 |
| [`IP-007-scheduler-binpack.md`](IP-007-scheduler-binpack.md) | placement decision engine (binpack over cluster state) | pending | axis-cell-substrate | IP-005 |
| [`IP-008-lifecycle-manager-k8s.md`](IP-008-lifecycle-manager-k8s.md) | cell CRUD on K8s Cluster API + Postgres + S3 | pending | axis-cell-substrate + cloud-k8s | IP-001, IP-005 |
| [`IP-009-tenant-assignment-stack.md`](IP-009-tenant-assignment-stack.md) | full BC: kernel→app for tenant binding lookup + write | pending | axis-cell-substrate | IP-005 |
| [`IP-010-tenant-migration-orchestrator.md`](IP-010-tenant-migration-orchestrator.md) | end-to-end migration use case (drain→copy→cutover) | pending | axis-cell-substrate | IP-009 |
| [`IP-011-host-pool-drain-primitive.md`](IP-011-host-pool-drain-primitive.md) | cordon + evict + verify; FM-08 recovery | pending | cloud-k8s | IP-001 |
| [`IP-012-cell-registry-events-emitter.md`](IP-012-cell-registry-events-emitter.md) | AsyncAPI event surface emission to Workflow bus | pending | axis-cell-substrate | IP-005 |
| [`IP-013-observability-slo-manifests.md`](IP-013-observability-slo-manifests.md) | cell µservice OpenSLO manifests at `microservices/cell/slos/` | pending | axis-cell-substrate + axis-observability | IP-005 |
| [`IP-014-branch-protection-gate-registration.md`](IP-014-branch-protection-gate-registration.md) | wire cell-boundary into branch-protection.yaml | pending | axis-foundry | IP-006 |
| [`IP-015-hyperscaler-claim-gate.md`](IP-015-hyperscaler-claim-gate.md) | HG-CELL claim registration per ADR-0123 | pending | council-architecture | IP-010 |

Coverage check: all 5 BCs across all layers; IaC; gate lane; events; SLO publication; branch-protection wiring; claim-gate registration. The migration of legacy `oya-cell-domain` is tracked by parallel IP-MIGR-001 (not in this phase).

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --microservice cell
oya gate validate lean-a2 --microservice cell
oya gate validate port-location --microservice cell
oya gate validate layer-correctness --microservice cell
oya gate validate per-microservice-layout --microservice cell
oya gate validate statelessness --microservice cell
oya gate validate shardability --microservice cell
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
oya gate validate cell-boundary --microservice cell
```

### Substrate gates introduced by this phase

```bash
oya cell get-assignment --tenant <hashed-id>           # p99 ≤ 50ms drill
oya cell migrate-tenant --tenant <id> --to <cell>      # ≤ 10min p99 drill
oya cell decommission --cell <id> --dry-run            # safe-default check
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| New tenant onboarding | `cargo nextest run -p oya-cell-tenant-assignment-usecase --test onboarding_happy_path` | placement decision in ≤ 500 ms; `CellAssigned` event in ≤ 2 s |
| Tenant migration | scripted e2e: drain → copy → cutover | end-to-end ≤ 10 min p99; per-checkpoint resumable |
| Cell decommission | scripted e2e: drain all tenants → delete cell | terminal state reached; ≥ 30d soft-delete window before Postgres schema drop |
| Cell-boundary lane | author PR introducing cross-cell DB ref | lane exits non-zero |
| Cross-pack write attempt | author PR placing tenant in wrong pack cell | Cedar policy refuses + audit-emitted |
| Host drain | cordon node → evict | tenant impact = 0; verified by burn-rate quiet during drain |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice cell
oya gate validate ontology-type-registry --microservice cell
```

## Clean Architecture Compliance

Layer assignments and dependency direction:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-cell-cell-registry-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-cell-cell-registry-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `oya-cell-cell-registry-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-cell-cell-registry-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-cell-cell-registry-adapter-postgres` | `adapter` (backend-qualified) | as above | as above |
| `oya-cell-cell-registry-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter` directly (uses ports) |
| `oya-cell-cell-registry-sdk` | `sdk` | `api`, `kernel` types | `adapter` |
| `oya-cell-cell-registry-app` | `app` | composition root | none — only wiring |
| (analogous for tenant-assignment / scheduler / lifecycle-manager / host-pool) | | | |

Port traits live exclusively in `*-kernel`; implementations in `*-adapter*`. Domain calls through ports; never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `cell` and any other product µservice. All cross-product flows go via Workflow events + Ontology.

## ChangeSet Contract per IP

Every IP emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload, written at `microservices/cell/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "cell",
  "milestone": "M01-foundation",
  "phase": "P01-cell-substrate",
  "claim_paths": ["microservices/cell/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/cell/PRD.md§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-nextest", "lean-a1", "lean-a2", "port-location", "per-microservice-layout", "cell-boundary"],
  "test_count": {"unit": 0, "integration": 0, "e2e": 0},
  "coverage_pct": 0.0,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

Schema validated by `oya-governance-multispectrum-evidence` lane against `/specs/multispectrum-review.json` v2.4.0.

## Per-IP Test Coverage Threshold

Same per-layer thresholds as `microservices/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` §"Per-IP Test Coverage Threshold". Enforced by `cargo nextest` + `cargo llvm-cov --fail-under-lines`.

## Naming Justifications

Per BC; see `PRD.md` §"Bounded Contexts" + each catalog row under `catalog/`.

## References

- Bominal ADR-0009; Bominal ADR-0019.
- ADR-0056; ADR-0105; ADR-0106; ADR-0117; ADR-0139; ADR-0131.
- `microservices/cell/PRD.md`.
- `microservices/cell/threat-model.md`.
- `microservices/cell/policy/cell-boundary.md`.
