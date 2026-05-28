---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
status: Active
entry_gate: |
  PRD-plugin-app-store accepted; ADR-0213 accepted; sibling developer-sdk µservice scaffolded;
  cargo workspace ready to accept the new plugin-app-store crates under microservices/plugin-app-store/src/;
  Postgres + Valkey + Cedar + Cosign + Trivy + Wasmtime Layer-A IaC available via cloud-iac µservice.
exit_gate: |
  All 15 IPs merged; plugin-app-store binary deployed to dev cluster; vetting-pipeline + per-plugin-permissions
  + per-plugin-rate-limit + subscription-billing wired end-to-end; .github/branch-protection.yaml updated
  with HG-PAS gates on dev + staging; release/plugin-app-store/{staging,production} pattern protection live;
  end-to-end install drill passes (browse → install → grant perms → run → uninstall); cargo nextest run --workspace exits 0;
  oya gate validate per-microservice-layout --microservice plugin-app-store exits 0;
  oya gate validate authority-cohesion exits 0; HG-PAS gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: plugin-app-store SLO promotion gate must exist before this µservice can be promoted past dev
  - milestone: M02b-substrate-ready
    phase: P01-durable-execution-substrate
    reason: workflow-engine event bus is the cross-product routing fabric for plugin events
  - milestone: M02b-substrate-ready
    phase: cloud-iac substrate
    reason: Postgres + Valkey + Cedar substrate must precede plugin-app-store crate authoring
owner_team: axis-ecosystem
related_adrs: [ADR-0213, ADR-0131, ADR-0132, ADR-0139, ADR-0147, ADR-0181, ADR-0200, ADR-0211]
related_specs: [/specs/microservices/plugin-app-store.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-18
doc_status: published
---

# P01-plugin-app-store-substrate: Land the plugin/app store end-to-end

## Purpose

This phase ships the full plugin-app-store substrate — Apple-App-Store-parity discovery + install + per-plugin-permission grant + vetting pipeline + per-plugin rate limiting + subscription billing aggregation + per-plugin action audit trail. It is delivered as one phase in M04-ecosystem-substrate because every plugin in the catalog depends on the whole pipeline; partial delivery (e.g., install without vetting) would create a security regression.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (Apple-App-Store-class per-plugin permission model + Stripe-Connect-parity billing aggregation).
- Nothing scheduled-for-distinct-tracked-work (every FUTURE-marked stub from ADR-0037 plugin-substrate is decommissioned).
- No silent regression (vetting-pipeline-correctness CI lane is BLOCKER day 1).
- Per-microservice flat layout (ships natively under ADR-0131; sibling = developer-sdk).
- In-house from day one (per ADR-0211; no external marketplace SaaS, no external vetting SaaS).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `plugin-app-store` | `plugin-catalog`, `plugin-install`, `plugin-lifecycle`, `vetting-pipeline`, `per-plugin-permissions`, `per-plugin-rate-limit`, `subscription-billing`, `audit-stream` | All under `microservices/plugin-app-store/` per ADR-0131 | 51 crates per PRD §"Bounded Contexts" |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-governance-vetting-pipeline-correctness`, `oya-governance-per-plugin-permission-enforcement` to required_status_checks on `dev`; add pattern protection for `release/plugin-app-store/{staging,production}`.
- `Cargo.toml` (workspace) — register the 51 new crates under `microservices/plugin-app-store/src/crates/`. **DEFERRED to parent-wiring-todo per scope-lock.**
- `crates/oya-foundry-microservices/src/lib.rs` MICROSERVICES const — register `plugin-app-store`. **DEFERRED to parent-wiring-todo per scope-lock.**
- `/specs/hyperscaler-gates.json` — register HG-PAS gate per ADR-0123.
- `docs/standards/plugin-vetting-pipeline.md` (NEW) — cross-cutting standard for vetting stage authoring; declares the 8 deterministic stages + their pass/fail criteria.

Naming justifications for the new crate families are in `microservices/plugin-app-store/PRD.md` §"Bounded Contexts".

### Out-of-scope

- B2C commerce marketplace (Amazon/Shopify class) — RESERVED for a future `marketplace` µservice per ADR-0213 §Disambiguation.
- LinkedIn-class community surface — owned by sibling `community` µservice; out of scope per ADR-0213 §Disambiguation.
- Developer-facing surface (SDK + portal + sandbox + onboarding + payout) — owned by sibling `developer-sdk` µservice.
- Cross-tenant plugin marketplace mode — scheduled-for-distinct-tracked-work to a subsequent ADR.

## Implementation Plans

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| IP-001-layer-a-postgres-valkey-cedar-cosign-trivy-iac | Helm + Kustomize charts for Postgres, Valkey, Cedar evaluator binding, Cosign, Trivy, Wasmtime runtime under `microservices/plugin-app-store/iac/helm/` | pending | axis-ecosystem | — |
| IP-002-plugin-catalog-kernel-domain | `oya-plugin-app-store-plugin-catalog-{kernel,domain}` crates: Plugin, PluginVersion, PluginRating entities + pure search/filter logic | pending | axis-ecosystem | — |
| IP-003-plugin-catalog-usecase-api-adapter-rest-sdk-app | plugin-catalog remaining layers (read-heavy; Postgres `tsvector` index + Cilium L4 cache) | pending | axis-ecosystem | IP-002 |
| IP-004-plugin-lifecycle-state-machine | Lifecycle state machine: draft → submitted → vetting → published → deprecated → retired + revoked | pending | axis-ecosystem | IP-002 |
| IP-005-plugin-install-kernel-domain-usecase | plugin-install kernel + domain + usecase | pending | axis-ecosystem | IP-004 |
| IP-006-plugin-install-rest-sdk-app | plugin-install rest + sdk + app + adapter-postgres | pending | axis-ecosystem | IP-005 |
| IP-007-vetting-pipeline-kernel-domain | Vetting pipeline kernel + domain (deterministic 8-stage orchestration) | pending | axis-ecosystem + council-security | IP-004 |
| IP-008-vetting-pipeline-cosign-trivy-wasmtime | Cosign + Trivy + Wasmtime-isolation validators (the heart of the security gate) | pending | axis-ecosystem + council-security | IP-007 |
| IP-009-per-plugin-permissions-cedar | Per-plugin Cedar policy fragment generator + install-time grant capture | pending | axis-ecosystem + council-security | IP-005 |
| IP-010-per-plugin-rate-limit | Per-installation rate-limit (default 100 req/s; per-plugin override; Valkey-backed token bucket) | pending | axis-ecosystem | IP-009 |
| IP-011-subscription-billing-aggregation | Subscription state machine + billing aggregation feeding finops-portal | pending | axis-ecosystem + axis-finops | IP-006 |
| IP-012-audit-stream-per-plugin-action | Per-plugin action audit trail seal via audit-chain | pending | axis-ecosystem + axis-audit | IP-009 |
| IP-013-observability-slo-manifests | plugin-app-store OpenSLO manifests + observability self-SLOs | pending | axis-ecosystem + axis-observability | IP-011 |
| IP-014-branch-protection-and-hyperscaler-gates | branch-protection.yaml updates + HG-PAS gate registration | pending | axis-ecosystem + ops-sre-reliability | IP-013 |
| IP-015-discovery-install-leptos-app | Discovery + install Leptos app (tenant-facing UI; design-system parity with workflow-studio) | pending | axis-ecosystem + council-design-system | IP-006 |

## Acceptance Gates

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps

oya gate validate lean-a1 --microservice plugin-app-store
oya gate validate lean-a2 --microservice plugin-app-store
oya gate validate port-location --microservice plugin-app-store
oya gate validate layer-correctness --microservice plugin-app-store
oya gate validate per-microservice-layout --microservice plugin-app-store
oya gate validate statelessness --microservice plugin-app-store
oya gate validate shardability --microservice plugin-app-store
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims

oya gate validate vetting-pipeline-correctness --microservice plugin-app-store
oya gate validate per-plugin-permission-enforcement --microservice plugin-app-store
oya gate validate per-plugin-rate-limit-correctness --microservice plugin-app-store
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Install + revoke flow | scripted e2e: browse → install → grant perms → run plugin action → revoke | revoke propagates ≤ 30s p99; audit chain intact |
| Vetting pipeline correctness | `cargo nextest run -p oya-plugin-app-store-vetting-pipeline-domain --test deterministic_decision` | identical input → identical decision |
| Per-plugin Cedar denial | scripted e2e: install plugin with declared cap=read; attempt cap=write | denied + audit-logged |
| 10k concurrent installs | `k6 run tests/load/install-10k.js` | p99 install ≤ 15s |
| Billing aggregation correctness | `cargo nextest run -p oya-plugin-app-store-subscription-billing-domain --test aggregation_correctness` | byte-equal totals to finops-portal stub |
| Kill-switch propagation | scripted e2e: revoke malicious plugin; verify all installations stop | propagation ≤ 30s p99 |

## Clean Architecture Compliance

Layer assignments and dependency direction (one representative BC; same shape for the other seven):

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-plugin-app-store-vetting-pipeline-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-plugin-app-store-vetting-pipeline-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `oya-plugin-app-store-vetting-pipeline-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-plugin-app-store-vetting-pipeline-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-plugin-app-store-vetting-pipeline-adapter-cosign` | `adapter-cosign` | `adapter`, `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-plugin-app-store-vetting-pipeline-adapter-trivy` | `adapter-trivy` | `adapter`, `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-plugin-app-store-vetting-pipeline-worker` | `worker` | `usecase`, `api`, `domain`, `kernel` | `adapter*` directly (uses ports) |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*` crates. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `plugin-app-store` and any other product µservice's crates. All cross-product data flow uses the workflow-engine event-bus (Workflow + Ontology adapter layer).

## ChangeSet Contract per IP

Every IP emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload per IP, written at `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "plugin-app-store",
  "milestone": "M04-ecosystem-substrate",
  "phase": "P01-plugin-app-store-substrate",
  "claim_paths": ["microservices/plugin-app-store/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/plugin-app-store/PRD.md§<section>", "/specs/microservices/plugin-app-store.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "per-microservice-layout", "vetting-pipeline-correctness", "per-plugin-permission-enforcement"],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

## Per-IP Test Coverage Threshold

| IP class | Minimum unit-test count | Minimum integration-test count | Minimum e2e-test count | Coverage threshold |
|---|---|---|---|---|
| kernel crate | 1 per public type + 1 per port trait | 0 (pure) | 0 | 90% line; 80% branch |
| domain crate | 1 per public function + deterministic property tests | 0 | 0 | 95% line; 90% branch |
| usecase crate | 1 per use case (happy + 2 sad paths) | ≥ 3 against mocked ports | 0 | 90% line; 80% branch |
| adapter crate | 1 per port-impl method | ≥ 2 against real backend (Postgres / Valkey / Cosign / Trivy / Wasmtime test container) | 0 | 85% line; 75% branch |
| rest crate | 1 per route (happy + auth-fail + tenant-mismatch) | ≥ 2 cross-route flows | 1 per route via REST integration test | 85% line; 75% branch |
| worker crate | 1 per orchestration arm | ≥ 1 long-lived loop integration test | 1 e2e (vetting pipeline drill) | 85% line; 75% branch |
| sdk crate | 1 per public client method (happy + retry + auth-fail) | ≥ 2 against rest crate | 0 | 90% line; 80% branch |
| app crate | composition-root smoke tests | 0 (delegates to worker/rest tests) | 1 startup-and-shutdown smoke | 60% line (mostly wiring) |
| IaC IPs | n/a | ≥ 1 helm-install + helm-test smoke per chart | 1 against kind/k3d cluster | n/a |

## branch-protection.yaml diff preview

IP-014 updates `.github/branch-protection.yaml` with:

```yaml
branches:
  dev:
    required_status_checks:
      - oya-governance-vetting-pipeline-correctness        # NEW; from IP-008
      - oya-governance-per-plugin-permission-enforcement   # NEW; from IP-009
      - oya-governance-per-plugin-rate-limit-correctness   # NEW; from IP-010
  staging:
    required_status_checks:
      - oya-governance-vetting-pipeline-correctness
      - oya-governance-promotion-readiness
  ? release/plugin-app-store/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness
  ? release/plugin-app-store/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-governance-promotion-readiness
```

## Oya VCS Symbol Locks

Per ADR-0116, this phase uses `oya vcs` primitives exclusively. Grit and ICM are explicitly NOT used.

```bash
cargo run -p oya-dev-cli -- vcs claim --agent <agent-id> --intent "<IP-NNN-slug>: <one-line intent>" --paths "microservices/plugin-app-store/src/crates/<crate>/**"
cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

## References

- ADR-0213 (Ecosystem-as-a-Service architecture — Plugin/App Store substrate).
- ADR-0037 (Bominal plugin substrate — superseded for new work).
- ADR-0131 (per-microservice flat layout); ADR-0132 (no-suite policy); ADR-0139 (agentic SLO-gated promotion).
- ADR-0147 (Wasmtime sandbox baseline); ADR-0181 (Cosign signing); ADR-0200 (Wasmtime canonical); ADR-0211 (in-house tech policy).
- `microservices/plugin-app-store/PRD.md`.
- `/specs/microservices/plugin-app-store.json` (to be authored as follow-up CR).
- `/specs/per-microservice-flat-layout.json`.
- Memory: `feedback_quality_performance_scalability_bar.md`; `feedback_workflow_objectgraph_adapter_layer.md`; `feedback_canonical_base_localization.md`; `feedback_naming_justification.md`.
