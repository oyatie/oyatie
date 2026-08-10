---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M01-foundation
phase: P01-tenancy-substrate-stable
status: Active
entry_gate: |
  Bominal ADR-0018 + ADR-0011 + ADR-0019 + ADR-0009 + ADR-0117 + ADR-0028 inherited;
  ADR-0131 (per-microservice flat layout) + ADR-0139 (agentic SLO gate) accepted;
  /specs/per-microservice-flat-layout.json published;
  microservices/observability/* substrate authored and at Slice D readiness (tenancy consumes it);
  existing crates/oya-tenancy-{kernel,domain,api} are RLS-correct and stable from M01-P01.
exit_gate: |
  All 15 IPs merged;
  `oya-tenancy-{tenant-lifecycle,isolation-policy,cell-assignment,dsr-cascade}-*` crates exist + build clean;
  branch-protection.yaml carries rls-no-superuser-bypass + rls-force-on-tenant-tables + jwt-key-fingerprint-advertised as required_status_checks on dev;
  Patroni HA + Citus shard cluster live in pack-kr (M01 launch);
  cargo nextest run --workspace exits 0;
  `oya gate validate per-microservice-layout --microservice tenancy` exit 0;
  `oya gate validate authority-cohesion` exit 0 with HG-TEN registered;
  proof-of-erasure certificate produced end-to-end for a synthetic DSR cascade across all M01 µservices;
  observability self-SLOs green at the µservice level for ≥ 7 days.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion (observability)
    reason: tenancy emits OpenSLOs and depends on the gate; observability must precede.
owner_team: axis-tenancy
related_adrs: [ADR-0018, ADR-0011, ADR-0019, ADR-0009, ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
doc_status: published
---

# P01-tenancy-substrate-stable: Land the multi-tenant isolation substrate end-to-end

## Purpose

This phase ships the full tenancy substrate per Bominal ADR-0018 inherited 1:1 + oyatie-specific additions:

- 4 BCs (tenant-lifecycle, isolation-policy, cell-assignment, dsr-cascade) across 35 crates.
- Layer-A persistence stack (Postgres + Citus + Patroni HA) for tenant metadata + multi-tenant sharding.
- Layer-B oyatie-owned code: lifecycle FSM, RLS policy generator, JWT issuer/verifier, cell-assignment controller, DSR cascade orchestrator.
- Cross-µservice DSR cascade contract with cryptographic proof-of-erasure.
- 3 NEW LEAN lanes specific to tenancy: rls-no-superuser-bypass, rls-force-on-tenant-tables, jwt-key-fingerprint-advertised.
- HG-TEN hyperscaler-maturity-claim gate registered in `/specs/hyperscaler-gates.json` per ADR-0123.

This phase advances master-plan principles:

- **Hyperscaler-grade in every practice** (Citus + Patroni HA; Cedar + RLS + JWT defence-in-depth; OpenBao-backed signing keys).
- **Nothing scheduled-for-distinct-tracked-work** (the existing `oya-tenancy-{kernel,domain,api}` crates are migrated into `microservices/tenancy/` and re-shaped per ADR-0105 + ADR-0131; no parallel-stub story).
- **No silent regression** (production-tier breach auto-reverts via the SLO gate; tenant validation kept on the 99.99% availability tier; rollback policy in `runbooks/rls-drift-recovery.md`).
- **Per-microservice flat layout** (this phase is a native author under ADR-0131).
- **Compliance is the highest-stakes axis** (tenancy authoring at SOC 2 / ISO 27001 / GDPR DPA / KR PIPC scrutiny bar; per-pack overlays for all 11 packs).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `tenancy` | `tenant-lifecycle`, `isolation-policy`, `cell-assignment`, `dsr-cascade` | All under `microservices/tenancy/` per ADR-0131 | `oya-tenancy-tenant-lifecycle-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,sdk,app}` + `oya-tenancy-isolation-policy-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,worker,app}` + `oya-tenancy-cell-assignment-{kernel,domain,usecase,api,adapter,adapter-citus,worker,app}` + `oya-tenancy-dsr-cascade-{kernel,domain,usecase,api,adapter,rest,worker,app}` |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):

- `.github/branch-protection.yaml` — add `rls-no-superuser-bypass` + `rls-force-on-tenant-tables` + `jwt-key-fingerprint-advertised` to required_status_checks on `dev`; add `release/tenancy/{staging,production}` pattern protection.
- `Cargo.toml` (workspace) — register the 35 new tenancy crates under `microservices/tenancy/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-TEN gate per ADR-0123.
- `docs/standards/multi-tenant-isolation.md` (NEW) — cross-cutting tenant-isolation invariants (per-µservice contract).
- `registry/catalog/oya-tenancy-*.yaml` — 35 catalog records (one per crate).

Naming justifications for the new crate families are in `tenancy/PRD.md` §"Bounded Contexts".

### Out-of-scope

- **Schema-per-tenant alternative** for very-large tenants (Open Question 1 in PRD) — scheduled-for-distinct-tracked-work to a successor-IP ADR; default shared-schema-RLS shape lands in this phase.
- **Cross-pack migration tooling** — tenants pin to one pack at creation; migrating a tenant across packs is a separate IP outside this phase (requires SCC review + DPO + ops-security 2-person rule).
- **ML-driven cell assignment** — Open Question 2; defer; consistent-hash baseline ships this phase.
- **OAuth2/OIDC issuer functionality** — tenancy issues internal-only JWTs; OAuth2/OIDC integration for end-user-facing auth belongs to a separate `identity` µservice.
- **Migration of physical crate locations** — IP-015 captures the migration scope; until that lands the legacy `crates/oya-tenancy-*` paths remain in place (read-only).

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-layer-a-postgres-citus-patroni-iac.md`](IP-001-layer-a-postgres-citus-patroni-iac.md) | Helm/Kustomize charts for Postgres + Citus + Patroni HA at `microservices/tenancy/iac/helm/`; pack-kr overlay | pending | ops-sre-reliability + axis-tenancy | — |
| [`IP-002-tenant-lifecycle-kernel.md`](IP-002-tenant-lifecycle-kernel.md) | `oya-tenancy-tenant-lifecycle-kernel` crate: ports + entities (Tenant, TenantId, TenantStatus, JurisdictionCode, PlanTier, TenantContext) | pending | axis-tenancy | IP-001 |
| [`IP-003-tenant-lifecycle-domain.md`](IP-003-tenant-lifecycle-domain.md) | `-domain` crate: lifecycle FSM, plan-tier rules, jurisdiction validators | pending | axis-tenancy | IP-002 |
| [`IP-004-tenant-lifecycle-usecase.md`](IP-004-tenant-lifecycle-usecase.md) | `-usecase` crate: create/activate/suspend/resume/delete orchestrators | pending | axis-tenancy | IP-003 |
| [`IP-005-tenant-lifecycle-adapter-postgres.md`](IP-005-tenant-lifecycle-adapter-postgres.md) | `-adapter-postgres` crate: TenantRepository over sqlx; schema migration runner | pending | axis-tenancy | IP-002, IP-001 |
| [`IP-006-isolation-policy-rls-generator.md`](IP-006-isolation-policy-rls-generator.md) | `oya-tenancy-isolation-policy-{kernel,domain,usecase,adapter-postgres}` — RLS policy YAML → Postgres DDL emitter + FORCE ROW LEVEL SECURITY enforcement | pending | axis-tenancy + ops-security | IP-001 |
| [`IP-007-isolation-policy-jwt-issuer.md`](IP-007-isolation-policy-jwt-issuer.md) | JWT issuer + verifier crates; OpenBao-backed Ed25519; fingerprint advertise via Workflow | pending | axis-tenancy + ops-security | IP-006 |
| [`IP-008-cell-assignment-controller.md`](IP-008-cell-assignment-controller.md) | `oya-tenancy-cell-assignment-*` — consistent-hash shard derivation; cell-health probe loop; Citus pg_dist_* writes via `-adapter-citus` | pending | axis-tenancy + ops-sre-reliability | IP-001 |
| [`IP-009-dsr-cascade-runner.md`](IP-009-dsr-cascade-runner.md) | `oya-tenancy-dsr-cascade-*` — DSR ingestion + Workflow fan-out + receipt aggregation + proof-of-erasure Merkle root | pending | axis-tenancy + council-privacy | IP-002, IP-006 |
| [`IP-010-tenancy-rest-and-sdk.md`](IP-010-tenancy-rest-and-sdk.md) | `-rest` crates (OpenAPI) + `-sdk` crate (Rust) for programmatic admin | pending | axis-tenancy | IP-004, IP-007, IP-009 |
| [`IP-011-audit-chain-integration.md`](IP-011-audit-chain-integration.md) | Audit-chain Ed25519 seal emission on every lifecycle event + DSR receipt + RLS policy install | pending | axis-tenancy + audit-chain | IP-004, IP-006, IP-009 |
| [`IP-012-branch-protection-and-release-pointers.md`](IP-012-branch-protection-and-release-pointers.md) | branch-protection.yaml additions; tenancy release/<env> pointers established | pending | ops-sre-reliability | IP-007, IP-008 |
| [`IP-013-canary-cohort-and-rollback-wiring.md`](IP-013-canary-cohort-and-rollback-wiring.md) | Tenancy canary cohort wiring + rollback runbook + production-tier auto-rollback wire-up via observability gate | pending | ops-sre-reliability | IP-012 |
| [`IP-014-tests-load-drills-observability-slos.md`](IP-014-tests-load-drills-observability-slos.md) | k6 load tests; Patroni-failover drill; OpenSLO manifests at `microservices/tenancy/slos/*.openslo.yaml` | pending | ops-sre-reliability + axis-tenancy | IP-007, IP-008, IP-009 |
| [`IP-015-legacy-crates-migration.md`](IP-015-legacy-crates-migration.md) | Migrate existing `crates/oya-tenancy-{kernel,domain,api}` → `microservices/tenancy/src/crates/` per ADR-0131 + ADR-0105; preserve git history | pending | axis-tenancy | IP-002 (so target shape exists first) |

Coverage check vs. PRD §"Bounded Contexts": all 35 target crates landed by IPs 002–010; persistence stack by IP-001; cross-cutting (audit-chain, branch-protection, observability, migration) by IPs 011–015.

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
oya gate validate lean-a1 --microservice tenancy        # layer ordering
oya gate validate lean-a2 --microservice tenancy        # cross-product refusal
oya gate validate port-location --microservice tenancy  # ports in kernel
oya gate validate layer-correctness --microservice tenancy
oya gate validate per-microservice-layout --microservice tenancy  # ADR-0131
oya gate validate statelessness --microservice tenancy  # read path stateless
oya gate validate shardability --microservice tenancy
oya gate validate authority-cohesion                    # registers HG-TEN
oya gate validate hyperscaler-maturity-claims           # ADR-0123
```

### Substrate gates introduced by this phase

```bash
oya gate validate rls-no-superuser-bypass --microservice tenancy
oya gate validate rls-force-on-tenant-tables --microservice tenancy
oya gate validate jwt-key-fingerprint-advertised --microservice tenancy
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Tenant activation happy path | `cargo nextest run -p oya-tenancy-tenant-lifecycle-worker --test activation_end_to_end` | activation ≤ 5 min p99; RLS installed; cell assigned; events emitted |
| RLS cross-tenant refusal | `cargo nextest run -p oya-tenancy-isolation-policy-adapter-postgres --test rls_no_cross_tenant_rows` | zero rows returned in cross-tenant probe |
| Patroni failover availability | `tests/load/patroni-failover-availability.sh` | validate hot path ≥ 99.99% availability during primary loss |
| Citus rebalance integrity | `cargo nextest run -p oya-tenancy-cell-assignment-adapter-citus --test rebalance_integrity` | row checksums equal before/after rebalance |
| DSR cascade proof | `cargo nextest run -p oya-tenancy-dsr-cascade-worker --test dsr_cascade_proof` | every µservice emits receipt; Merkle root signed; tenant data unreachable |
| JWT rotation drill | `cargo nextest run -p oya-tenancy-isolation-policy-worker --test jwt_rotation` | fingerprint event fires; verifier caches refresh ≤ 30s |
| Validate-path load test | `k6 run tests/load/tenant-validate-100krps.js` | p99 ≤ 5 ms at 100k RPS sustained 10 min |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice tenancy
oya gate validate ontology-type-registry --microservice tenancy
```

### Compliance gates

```bash
# Per pack-kr launch overlay
oya gate validate compliance-evidence-recency --microservice tenancy
# Per-pack retention conformance
oya gate validate retention-conformance --microservice tenancy
```

## Clean Architecture Compliance

Layer assignments and dependency direction:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-tenancy-tenant-lifecycle-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-tenancy-tenant-lifecycle-domain` | `domain` | `kernel` | `usecase`, `adapter*`, `rest`, `worker`, `sdk`, `app` |
| `oya-tenancy-tenant-lifecycle-usecase` | `usecase` | `domain`, `kernel` | `adapter*`, `rest`, `worker`, `sdk`, `app` |
| `oya-tenancy-tenant-lifecycle-api` | `api` | `kernel` | all others |
| `oya-tenancy-tenant-lifecycle-adapter` | `adapter` | `usecase`, `domain`, `kernel`, `api` | `rest`, `worker`, `sdk`, `app` directly |
| `oya-tenancy-tenant-lifecycle-adapter-postgres` | `adapter` (backend-qualified) | `usecase`, `domain`, `kernel`, `api` | as above |
| `oya-tenancy-tenant-lifecycle-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter*` directly (uses ports) |
| `oya-tenancy-tenant-lifecycle-worker` | `worker` | `usecase`, `domain`, `kernel`, `api` | `adapter*` directly (uses ports) |
| `oya-tenancy-tenant-lifecycle-sdk` | `sdk` | `kernel`, `api` | all others |
| `oya-tenancy-tenant-lifecycle-app` | `app` | (composition-root wiring only) | none — but only wiring |
| `oya-tenancy-isolation-policy-*` | per BC enum | analogous |  |
| `oya-tenancy-cell-assignment-*` | per BC enum | analogous |  |
| `oya-tenancy-dsr-cascade-*` | per BC enum | analogous |  |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*`. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `tenancy` and any other product µservice's crates. All cross-product data flow uses Workflow events (`TenantActivated`, `TenantSuspended`, `TenantDeletionRequested`, `ErasureReceipt`, `JwtSigningKeyRotated`, `CellRebalanceStarted/Completed`, `RlsPolicyInstalled`) and Ontology reads/writes (`Tenant`, `TenantStatus`, `RlsPolicy`, `CellAssignment`, `DsrRequest`, `ProofOfErasure`).

## ChangeSet Contract per IP

Every IP emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). Min ChangeSet payload at `microservices/tenancy/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "tenancy",
  "milestone": "M01-foundation",
  "phase": "P01-tenancy-substrate-stable",
  "claim_paths": ["microservices/tenancy/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["tenancy/PRD.md§<section>", "Bominal ADR-0018"],
  "acceptance_lanes_green": [
    "cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny",
    "lean-a1", "lean-a2", "lean-a3", "lean-a4",
    "per-microservice-layout",
    "rls-no-superuser-bypass", "rls-force-on-tenant-tables", "jwt-key-fingerprint-advertised"
  ],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

Validated by `oya-governance-multispectrum-evidence` lane per `/specs/multispectrum-review.json` v2.4.0.

## Per-IP Test Coverage Threshold

Per-IP test thresholds match observability phase 01 (same canonical table):

| IP class | Min unit | Min integration | Min e2e | Coverage |
|---|---|---|---|---|
| kernel | 1 per public type + 1 per port trait | 0 | 0 | 90% line; 80% branch |
| domain | 1 per public function + property tests | 0 | 0 | 95% line; 90% branch |
| usecase | 1 per use case (happy + 2 sad) | ≥ 3 against mocked ports | 0 | 90% line; 80% branch |
| adapter / adapter-postgres / adapter-citus | 1 per port-impl method | ≥ 2 against real backend container | 0 | 85% line; 75% branch |
| rest | 1 per route (happy + auth-fail + tenant-mismatch) | ≥ 2 cross-route flows | 1 per route | 85% line; 75% branch |
| worker | 1 per orchestration arm | ≥ 1 long-lived loop | 1 e2e | 85% line; 75% branch |
| sdk | 1 per public method (happy + retry + auth-fail) | ≥ 2 against rest | 0 | 90% line; 80% branch |
| app | composition-root smoke | 0 | 1 startup-shutdown | 60% line |
| IaC | n/a | ≥ 1 helm-install + helm-test per chart | 1 against kind/k3d | n/a |

Enforced by:
- `cargo nextest run --workspace --all-features` exits 0.
- `cargo llvm-cov --workspace --fail-under-lines <threshold>` exits 0.
- Per-IP `[acceptance_lanes]` frontmatter declares thresholds.

## branch-protection.yaml diff preview

IP-012 updates `.github/branch-protection.yaml` per:

```yaml
branches:
  dev:
    required_status_checks:
      # existing checks...
      # ADDED by this phase (IP-012):
      - oya-governance-rls-no-superuser-bypass
      - oya-governance-rls-force-on-tenant-tables
      - oya-governance-jwt-key-fingerprint-advertised
      - oya-governance-tenancy-residency-conformance     # per data-residency.md
      - oya-governance-tenancy-cedar-coverage           # per policy/*.cedar
  staging:
    required_status_checks:
      - oya-governance-rls-no-superuser-bypass
      - oya-governance-rls-force-on-tenant-tables

  ? release/tenancy/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness
      - oya-governance-rls-no-superuser-bypass
      - oya-governance-rls-force-on-tenant-tables

  ? release/tenancy/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness
      - oya-governance-rls-no-superuser-bypass
      - oya-governance-rls-force-on-tenant-tables
```

## Oya VCS Symbol Locks

Per ADR-0116, this phase uses `oya vcs` primitives exclusively.

```bash
# Claim before beginning each IP
cargo run -p oya-dev-cli -- vcs claim \
  --agent <agent-id> \
  --intent "<IP-NNN-slug>: <one-line intent>" \
  --paths "microservices/tenancy/src/crates/<crate>/**"

# Verify after each IP's acceptance gates pass
cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>

# Done — triggers rebase/merge/release primitive
cargo run -p oya-dev-cli -- vcs done --agent <agent-id> --changeset <id>

# Promote — fast-forward release pointer through the gate
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/tenancy/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- Bominal ADR-0018: Tenancy + RLS posture (primary authority; inherited).
- ADR-0056 BNF v4.1; ADR-0105 13-layer enum; ADR-0106 usecase rename.
- ADR-0110 ChangeSet state machine.
- ADR-0117 Cloud-native infrastructure (residency).
- ADR-0123 Hyperscaler maturity claim gate (HG-TEN).
- ADR-0139 Agentic SLO-gated promotion (tenancy consumes the gate).
- ADR-0131 Per-microservice flat layout.
- ADR-0140 Cedar policy enforcement.
- `/specs/per-microservice-flat-layout.json`; `/specs/agentic-slo-gated-promotion.json`.
- `tenancy/PRD.md`.
- Memory: `feedback_milestone_phase_hierarchy.md`, `feedback_naming_justification.md`, `feedback_oya_vcs_canonical_2026_05_16.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
- Google SRE Workbook ch. 4–5 (SLO); ch. 11–14 (operational; postmortem).
- AWS Well-Architected Framework (Reliability + Security + Operational Excellence).
- Citus docs — `docs.citusdata.com`.
- Patroni docs — `patroni.readthedocs.io`.
