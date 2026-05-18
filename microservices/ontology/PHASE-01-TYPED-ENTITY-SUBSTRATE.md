---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02b-substrate-ready
phase: P01-typed-entity-substrate
status: Active
entry_gate: |
  ADR-0006, ADR-0055, ADR-0059, ADR-0106, ADR-0107, ADR-0122, ADR-0131 accepted;
  /specs/microservices/ontology.json + /specs/knowledge-graph-schema.json + /specs/per-microservice-flat-layout.json published;
  cargo workspace ready to accept the ~92 new crates under microservices/ontology/src/crates/.
exit_gate: |
  All 15 IPs merged; oya-foundry-fitness-ontology-tenancy-isolation, oya-foundry-fitness-ontology-tier-enforcement,
  oya-foundry-fitness-cedar-coverage, oya-foundry-fitness-audit-chain-emission, oya-foundry-fitness-ontology-dynamic-freshness
  CI lanes present in .github/branch-protection.yaml required_status_checks on dev and staging;
  release/ontology/{staging,production} pattern protection rules live;
  cargo nextest run --workspace exits 0;
  oya gate validate per-microservice-layout --microservice ontology exits 0;
  oya gate validate authority-cohesion exits 0;
  HG-ONT gate in /specs/hyperscaler-gates.json registers green;
  AC-01..AC-14 of PRD pass on dev.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: observability SLO gate must be live; ontology authors its own OpenSLO manifests under slos/.
  - milestone: M02b-substrate-ready
    phase: tenancy phase (tenant identity + Cedar entity types)
    reason: ontology depends on tenant resolver + JWT claims with tenant_id + pillar context.
owner_team: axis-ontology
related_adrs: [ADR-0006, ADR-0055, ADR-0059, ADR-0106, ADR-0107, ADR-0122, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_specs: [/specs/microservices/ontology.json, /specs/knowledge-graph-schema.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-typed-entity-substrate: Land the Palantir-Foundry-class Ontology end-to-end

## Purpose

This phase ships the full Ontology design — the Palantir-Foundry-class typed-entity substrate that every other oyatie µservice reads/writes through. It is the second of seven Tier-A µservice phases authored under ADR-0131 (per-microservice flat layout) and is the load-bearing precondition for every product because of `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`: cross-µservice data flow is impossible without it.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (Palantir parity on Function p99, AIP parity on agent gateway, AWS-Cedar parity on policy coverage).
- Nothing scheduled-for-distinct-tracked-work (full Cedar coverage; full audit-chain emission; full DSR cascade).
- No silent regression (RLS bypass / cross-tenant link / cross-pillar leak each fail the lane).
- Per-microservice flat layout (ADR-0131 native author).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `ontology` | `object-type-registry`, `link-type-registry`, `action-type-registry`, `function-type-registry`, `entity-store`, `link-store`, `function-engine`, `action-engine`, `cedar-fragment-coverage`, `query-engine`, `agent-gateway`, `audit-chain`, `pillar` | All under `microservices/ontology/` per ADR-0131 | `oya-ontology-<bc>-<layer>` per ADR-0056 v4.1 |

Cross-cutting repo artifacts:
- `.github/branch-protection.yaml` — add 5 new required_status_checks (see §"branch-protection.yaml diff preview" below).
- `Cargo.toml` workspace — register the ~92 new crates under `microservices/ontology/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-ONT gate per ADR-0123.
- `registry/catalog/oya-ontology-*.yaml` per crate (~92 files); generated catalog records.

### Out-of-scope

- Migration of legacy `oya-ontology-kernel` crate at repo root into `microservices/ontology/src/crates/` — owned by the M01-MIGR-* series; runs in parallel under separate phase.
- WASM plugin SDK distribution format — scheduled-for-distinct-tracked-work to a subsequent-to-M02b-completion ADR per Open Question 3 in PRD.
- Tenant-defined Function DSL (JSON-IR vs embedded-Rust) — scheduled-for-distinct-tracked-work to M02b/P02 successor-IP per Open Question 2.
- Cross-region DR pairs for Postgres + ClickHouse — scheduled-for-distinct-tracked-work to subsequent-to-M02b-completion ADR.

## Implementation Plans

Ordered. Each IP is one ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-ontology-iac-stack.md`](IP-001-ontology-iac-stack.md) | Helm/Kustomize charts for per-pack Postgres + Citus + ClickHouse + Cedar policy engine + Valkey under `microservices/ontology/iac/helm/` | pending | axis-ontology + cloud-iac | — |
| [`IP-002-object-type-registry-kernel-domain.md`](IP-002-object-type-registry-kernel-domain.md) | `oya-ontology-object-type-registry-{kernel,domain}` crates: port traits (`SchemaRegistry`, `PillarResolver`), entities (`ObjectTypeSchema`, `PropertyDescriptor`, `PropertyTier`, `PillarKind`, `JurisdictionOverlay`), pure schema-evolution logic | pending | axis-ontology | IP-001 |
| [`IP-003-link-action-function-type-registry.md`](IP-003-link-action-function-type-registry.md) | Sibling registries: `oya-ontology-{link-type-registry,action-type-registry,function-type-registry}-{kernel,domain,usecase}` | pending | axis-ontology | IP-002 |
| [`IP-004-entity-store-rls-citus.md`](IP-004-entity-store-rls-citus.md) | `oya-ontology-entity-store-{kernel,domain,usecase,adapter,adapter-postgres}`: Object Type instance persistence with Postgres `FORCE ROW LEVEL SECURITY`; Citus shard by `tenant_id` | pending | axis-ontology | IP-002 |
| [`IP-005-link-store-traversal.md`](IP-005-link-store-traversal.md) | `oya-ontology-link-store-{kernel,domain,usecase,adapter,adapter-postgres}`: typed Link Type persistence; cardinality enforcement; traversal API | pending | axis-ontology | IP-004 |
| [`IP-006-cedar-fragment-coverage-engine.md`](IP-006-cedar-fragment-coverage-engine.md) | `oya-ontology-cedar-fragment-coverage-{kernel,domain,usecase,api,adapter}`: Cedar v4 policy fragments + default-deny + autonomy-tier ceiling | pending | axis-ontology + ops-security | IP-002 |
| [`IP-007-action-engine-cedar-gated.md`](IP-007-action-engine-cedar-gated.md) | `oya-ontology-action-engine-{kernel,domain,usecase,adapter,worker}`: Action invocation with Cedar gate + idempotency + transaction receipt + audit emission | pending | axis-ontology | IP-004 + IP-006 |
| [`IP-008-function-engine-oltp-and-olap.md`](IP-008-function-engine-oltp-and-olap.md) | `oya-ontology-function-engine-{kernel,domain,usecase,adapter,worker}`: read-projection evaluator over Postgres OLTP | pending | axis-ontology | IP-004 |
| [`IP-009-clickhouse-history-mirror.md`](IP-009-clickhouse-history-mirror.md) | `oya-ontology-entity-store-adapter-clickhouse` + `oya-ontology-query-engine-adapter-clickhouse`: outbox → ClickHouse history-mirror; OLAP Function reads | pending | axis-ontology | IP-008 |
| [`IP-010-audit-chain-merkle-ed25519.md`](IP-010-audit-chain-merkle-ed25519.md) | `oya-ontology-audit-chain-{kernel,domain,usecase,adapter,worker}`: Merkle-tree per (tenant, period) + Ed25519 sealing via OpenBao | pending | axis-ontology | IP-007 |
| [`IP-011-query-engine-3layer-kg.md`](IP-011-query-engine-3layer-kg.md) | `oya-ontology-query-engine-{kernel,domain,usecase,adapter,adapter-clickhouse,worker}`: 3-layer Knowledge Graph (semantic / kinetic / dynamic per `/specs/knowledge-graph-schema.json`) | pending | axis-ontology | IP-009 |
| [`IP-012-agent-gateway-llm-tool-call.md`](IP-012-agent-gateway-llm-tool-call.md) | `oya-ontology-agent-gateway-{kernel,domain,usecase,adapter,rest}`: OpenAI-tool-spec auto-generation; Cedar autonomy-tier ceiling; LLM ingress | pending | axis-ontology + ops-security | IP-008 + IP-006 |
| [`IP-013-pillar-cross-pillar-grant.md`](IP-013-pillar-cross-pillar-grant.md) | `oya-ontology-pillar-{kernel,domain,usecase}`: org-pillar / person-pillar tier matrix + cross-pillar grant Cedar flow | pending | axis-ontology + council-privacy | IP-006 |
| [`IP-014-rest-and-sdk-surfaces.md`](IP-014-rest-and-sdk-surfaces.md) | `oya-ontology-{object-type-registry,link-type-registry,action-type-registry,function-type-registry,agent-gateway}-rest` + `-sdk` crates; OpenAPI 3.2 spec + Rust client; future TS/Python via bindgen | pending | axis-ontology + dx-sdk | IP-002..IP-013 |
| [`IP-015-app-binaries-and-branch-protection.md`](IP-015-app-binaries-and-branch-protection.md) | `oya-ontology-*-app` composition-root binaries; branch-protection.yaml update; HG-ONT registration in /specs/hyperscaler-gates.json; OpenSLO manifests at slos/ | pending | axis-ontology + axis-foundry | IP-014 |

Coverage check vs. ADR-0106 + ADR-0059 design: all 13 BCs covered; backend-qualified adapters (`-adapter-postgres`, `-adapter-clickhouse`) follow ADR-0105 Amendment 3.

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
oya gate validate lean-a1 --microservice ontology
oya gate validate lean-a2 --microservice ontology
oya gate validate port-location --microservice ontology
oya gate validate layer-correctness --microservice ontology
oya gate validate per-microservice-layout --microservice ontology
oya gate validate statelessness --microservice ontology
oya gate validate shardability --microservice ontology
oya gate validate ontology-tenancy-isolation --microservice ontology
oya gate validate ontology-tier-enforcement --microservice ontology
oya gate validate cedar-coverage --microservice ontology
oya gate validate audit-chain-emission --microservice ontology
oya gate validate ontology-dynamic-freshness --microservice ontology
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
oya gate validate ontology-cross-tenant-link --tenant-a <id> --tenant-b <id>
oya gate validate ontology-cross-pillar-grant --grant-id <id>
oya gate validate ontology-action-transaction-receipt --action-id <id>
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Tenant isolation | `cargo nextest run -p oya-ontology-entity-store-domain --test rls_isolation` | tenant-B query against tenant-A row returns empty; audit emit |
| Cross-tenant link refused | `cargo nextest run -p oya-ontology-link-store-domain --test cross_tenant_refused` | 403 with `CrossTenantLinkDenied`; audit emit |
| Function read p99 | `cargo bench -p oya-ontology-function-engine-domain -- function_read_p99` | p99 ≤ 50 ms at 10 k QPS |
| Action Cedar gate | `cargo nextest run -p oya-ontology-action-engine-domain --test cedar_gate` | deny → 403; permit → 200 + audit sealed |
| Agent gateway round-trip | `cargo nextest run --test agent_gateway_function_call` | LLM tool-call dispatched in ≤ 200 ms |
| DSR cascade | `cargo nextest run --test dsr_cascade` | erasure tombstones every matching Object Type within 30 d (test uses 1 s for CI) |
| Pillar isolation | `cargo nextest run -p oya-ontology-pillar-domain --test pillar_isolation` | org-pillar unreachable from person-pillar context |
| Audit chain verifiable | `cargo nextest run --test audit_chain_verify` | Merkle root verifies; tamper = verification failure |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice ontology
oya gate validate ontology-type-registry --microservice ontology
```

## Clean Architecture Compliance

Layer assignments and dependency direction per ADR-0105:

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-ontology-<bc>-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-ontology-<bc>-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `oya-ontology-<bc>-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-ontology-<bc>-api` | `api` | `kernel` | `domain` impl, `adapter`, `rest`, `worker`, `app` |
| `oya-ontology-<bc>-adapter` | `adapter` | `usecase`, `domain`, `kernel`, `api` | `rest`, `worker`, `app` |
| `oya-ontology-<bc>-adapter-<backend>` | `adapter` | as above + backend SDK | `rest`, `worker`, `app` |
| `oya-ontology-<bc>-rest` | `rest` | `usecase`, `domain`, `kernel`, `api` | `adapter` directly (uses ports) |
| `oya-ontology-<bc>-worker` | `worker` | `usecase`, `domain`, `kernel`, `api` | `adapter` directly (uses ports) |
| `oya-ontology-<bc>-sdk` | `sdk` | `api`, `kernel` | `usecase`/`domain`/`adapter` |
| `oya-ontology-<bc>-app` | `app` | (composition-root wiring only) | none — but wiring-only |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*`. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `ontology` and any other product µservice's crates. All cross-product data flow uses Workflow events (`ObjectTypeRegistered`, `ObjectInstanceMutated`, `ActionTypeInvoked`, `LinkTypeRegistered`, `AuditChainSealed`, `CrossPillarGrantRequested`) and Ontology reads/writes (the µservice itself is the authority).

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110. Minimum payload at `microservices/ontology/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "ontology",
  "milestone": "M02b-substrate-ready",
  "phase": "P01-typed-entity-substrate",
  "claim_paths": ["microservices/ontology/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/ontology/PRD.md§<section>", "/specs/microservices/ontology.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "lean-a3", "lean-a4", "per-microservice-layout", "ontology-tenancy-isolation", "ontology-tier-enforcement", "cedar-coverage", "audit-chain-emission"],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

## Per-IP Test Coverage Threshold

| IP class | Min unit tests | Min integration tests | Min e2e tests | Coverage threshold |
|---|---|---|---|---|
| kernel (`*-kernel`) | 1 per public type + 1 per port trait | 0 | 0 | 90 % line; 80 % branch |
| domain (`*-domain`) | 1 per public function + property tests for invariants | 0 | 0 | 95 % line; 90 % branch |
| usecase (`*-usecase`) | 1 per use case (happy + 2 sad paths) | ≥ 3 against mocked ports | 0 | 90 % line; 80 % branch |
| adapter (`*-adapter*`) | 1 per port-impl method | ≥ 2 against real backend (Postgres + ClickHouse testcontainers) | 0 | 85 % line; 75 % branch |
| rest (`*-rest`) | 1 per route (happy + auth-fail + tenant-mismatch) | ≥ 2 cross-route flows | 1 per route via REST integration test | 85 % line; 75 % branch |
| worker (`*-worker`) | 1 per orchestration arm | ≥ 1 long-lived loop integration test | 1 e2e | 85 % line; 75 % branch |
| sdk (`*-sdk`) | 1 per public client method | ≥ 2 against rest crate | 0 | 90 % line; 80 % branch |
| app (`*-app`) | composition smoke | 0 | 1 startup-and-shutdown | 60 % line |
| IaC IPs | n/a | ≥ 1 helm-install smoke per chart | 1 against kind cluster | n/a |

Enforced by `cargo llvm-cov --workspace --fail-under-lines <threshold>` + per-IP CI workflow.

## branch-protection.yaml diff preview

IP-015 updates `.github/branch-protection.yaml` with the diff below:

```yaml
branches:
  dev:
    required_status_checks:
      # existing checks (unchanged)
      ...
      # ADDED by this phase (IP-006 + IP-007 + IP-010 + IP-011 + IP-015):
      - oya-foundry-fitness-ontology-tenancy-isolation
      - oya-foundry-fitness-ontology-tier-enforcement
      - oya-foundry-fitness-cedar-coverage
      - oya-foundry-fitness-audit-chain-emission
      - oya-foundry-fitness-ontology-dynamic-freshness

  staging:
    required_status_checks:
      # same five lanes added

  ? release/ontology/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    required_status_checks:
      - oya-vcs-promotion-readiness
      - oya-foundry-fitness-ontology-tenancy-isolation
      - oya-foundry-fitness-cedar-coverage
      - oya-foundry-fitness-audit-chain-emission

  ? release/ontology/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    required_status_checks:
      - oya-vcs-promotion-readiness
      - oya-foundry-fitness-ontology-tenancy-isolation
      - oya-foundry-fitness-cedar-coverage
      - oya-foundry-fitness-audit-chain-emission
      - oya-foundry-fitness-ontology-dynamic-freshness
```

## Oya VCS Symbol Locks

Per ADR-0116, this phase uses `oya vcs` primitives exclusively.

```bash
cargo run -p oya-dev-cli -- vcs claim \
  --agent <agent-id> \
  --intent "<IP-NNN-slug>: <one-line>" \
  --paths "microservices/ontology/src/crates/<crate>/**"

cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

## References

- ADR-0006: Ontology typed-entity layer.
- ADR-0055 + ADR-0122: Ontology rename.
- ADR-0059: Workflow + Ontology = ecosystem adapter layer.
- ADR-0106 (Bominal): Ontology architecture.
- ADR-0107 (Bominal): Ontology agent gateway.
- ADR-0123: Hyperscaler maturity claim gate (HG-ONT).
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `/specs/microservices/ontology.json`.
- `/specs/knowledge-graph-schema.json`.
- `/specs/per-microservice-flat-layout.json`.
- `microservices/ontology/PRD.md`.
- Memory: `feedback_glossary_ontology_not_object_graph`, `feedback_workflow_objectgraph_adapter_layer`, `feedback_clean_architecture_requirements`, `feedback_quality_performance_scalability_bar`.
