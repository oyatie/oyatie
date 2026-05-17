---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
status: Active
entry_gate: |
  PRD-workflow-studio accepted; ADR-0131 unbundle accepted; sibling workflow-engine µservice substrate-ready
  (engine PHASE-01 exit_gate green); cargo workspace ready to accept the 52 new crates under
  microservices/workflow-studio/src/crates/; Layer-A IaC available via cloud-iac µservice (CDN +
  WebSocket gateway + Postgres + Redis); foundry-providers SDK available for LLM-assist; tenancy
  SDK available for per-seat licensing; ontology SDK available for object-type descriptors.
exit_gate: |
  All 15 IPs merged; Studio binary deployed to dev cluster (with WASM bundle on CDN); workflow-spec-roundtrip
  CI lane present in .github/branch-protection.yaml required_status_checks on dev and staging;
  release/workflow-studio/{staging,production} pattern protection live; round-trip byte-equality drill
  passes (load 100 golden specs, emit, byte-equal); collab CRDT merge drill passes (10 concurrent users,
  no silent loss); Cedar per-seat gate drill passes; cargo nextest run --workspace exits 0; oya gate
  validate per-microservice-layout --microservice workflow-studio exits 0; oya gate validate
  authority-cohesion exits 0; HG-WORKFLOW-STUDIO gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: workflow-studio SLO promotion gate must exist before workflow-studio itself can be advanced past dev
  - milestone: M02b-substrate-ready
    phase: P01-durable-execution-substrate
    reason: engine substrate must be live before Studio can emit specs to it
  - milestone: M02b-substrate-ready
    phase: prior phases per master-plan-sequencing
    reason: workspace + branch-protection + Cargo metadata authority must precede Studio crate authoring
owner_team: axis-workflow + council-design-system
related_adrs: [ADR-0065, ADR-0103, ADR-0130, ADR-0131, ADR-0140]
related_specs: [/specs/products/workflow-studio.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-visual-authoring-substrate: Land the workflow Studio end-to-end

## Purpose

This phase ships the full workflow-studio µservice — Studio half of the ADR-0131 workflow unbundle. The visual canvas, DSL emitter/loader with 100% round-trip byte-equality, collaborative CRDT editing, per-pack node libraries, jurisdiction-overlay renderer, replay-debugger frontend, LLM-assist authoring, and per-seat license-gate Cedar enforcement. Delivered as one phase in M03-studio-preview because Studio is the hero product surface — every other oyatie µservice's workflow authoring routes through it.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (CDN-cached WASM bundle + per-tenant CRDT collab + Cedar per-seat).
- Nothing deferred (every FUTURE-marked stub in any workflow-aware product's authoring UX is decommissioned by this phase's Studio SDK + emitter).
- No silent regression (workflow-spec-roundtrip CI lane is BLOCKER day 1).
- Per-microservice flat layout (this phase ships natively under ADR-0131; sibling = workflow-engine).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `workflow-studio` | `visual-canvas`, `dsl-emitter`, `dsl-loader`, `collab-crdt`, `node-library-registry`, `jurisdiction-overlay-renderer`, `replay-debugger-frontend`, `license-gate-cedar` | All under `microservices/workflow-studio/` per ADR-0131 | 52 crates per PRD §"Layer mapping per BC" |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-governance-workflow-spec-roundtrip`, `oya-governance-cedar-preview-required`, `oya-governance-editor-execution-forbidden`, `oya-governance-node-library-determinism`, `oya-governance-wasm-bundle-sri` to required_status_checks on `dev`; add pattern protection for `release/workflow-studio/{staging,production}`.
- `Cargo.toml` (workspace) — register the 52 new crates under `microservices/workflow-studio/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-WORKFLOW-STUDIO gate per ADR-0123.
- `docs/standards/workflow-studio-canvas.md` (NEW) — cross-cutting standard for Leptos visual-canvas authoring (declares deterministic-layout APIs; forbidden patterns: `innerHTML`, `eval`, non-keyed list rendering).

Naming justifications for the new crate families are in `microservices/workflow-studio/PRD.md` §"Bounded Contexts".

### Out-of-scope

- The engine execution substrate — separate µservice (`microservices/workflow-engine/`) per ADR-0131.
- Definition marketplace — deferred to a post-M03 phase.
- Per-tenant branding (mid-render) — explicitly anti-pattern per `/specs/products/workflow-studio.json` §anti_patterns.
- Multi-domain node libraries beyond the 6 declared (Agentic / Dev / Business / Healthcare / Supply-Chain / Delivery) — extensions per follow-up domain-pack phases.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-layer-a-cdn-waf-postgres-redis-ws-gateway-iac.md`](IP-001-layer-a-cdn-waf-postgres-redis-ws-gateway-iac.md) | Helm + Kustomize manifests for CDN (+ WAF), Postgres (editor session store), Redis (ephemeral CRDT state), WebSocket gateway deployment | pending | axis-workflow + cloud-iac | — |
| [`IP-002-visual-canvas-kernel-domain.md`](IP-002-visual-canvas-kernel-domain.md) | `oya-workflow-studio-visual-canvas-{kernel,domain}` crates: Canvas, Node, Edge, Selection, ViewportState entities + pure layout algebra | pending | axis-workflow + council-design-system | — |
| [`IP-003-dsl-emitter-loader-kernel-domain.md`](IP-003-dsl-emitter-loader-kernel-domain.md) | `oya-workflow-studio-dsl-emitter-{kernel,domain}` + `oya-workflow-studio-dsl-loader-{kernel,domain}`: pure visual ↔ workflow_spec.v1.json mapping; round-trip byte-equality invariant authored as property test | pending | axis-workflow | IP-002 |
| [`IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md`](IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md) | dsl-emitter + dsl-loader remaining layers (usecase + api + adapter + sdk) | pending | axis-workflow | IP-003 |
| [`IP-005-collab-crdt-kernel-domain-adapter.md`](IP-005-collab-crdt-kernel-domain-adapter.md) | `oya-workflow-studio-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-redis}` — CRDT merge engine + ephemeral session state | pending | axis-workflow | IP-004 |
| [`IP-006-collab-crdt-worker-sdk.md`](IP-006-collab-crdt-worker-sdk.md) | `oya-workflow-studio-collab-crdt-{worker,sdk}` — WebSocket gateway long-lived process + tenant SDK | pending | axis-workflow + cloud-iac | IP-005 |
| [`IP-007-node-library-registry-full.md`](IP-007-node-library-registry-full.md) | `oya-workflow-studio-node-library-registry-{kernel,domain,usecase,api,adapter,adapter-cdn,rest,sdk,app}` — signed per-pack library distribution via CDN | pending | axis-workflow | IP-001 |
| [`IP-008-llm-assist-adapter.md`](IP-008-llm-assist-adapter.md) | LLM-assist bridge crate `oya-workflow-studio-visual-canvas-adapter` extension consuming foundry-providers SDK; streaming draft response back to browser via WS | pending | axis-workflow + foundry-providers-team | IP-005 |
| [`IP-009-license-gate-cedar-full.md`](IP-009-license-gate-cedar-full.md) | `oya-workflow-studio-license-gate-cedar-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` — per-seat Cedar enforcement + audit emission | pending | axis-workflow + ops-security | IP-004 |
| [`IP-010-jurisdiction-overlay-renderer-full.md`](IP-010-jurisdiction-overlay-renderer-full.md) | `oya-workflow-studio-jurisdiction-overlay-renderer-{kernel,domain,usecase,api,adapter}` — pure overlay resolution + visual diff algebra | pending | axis-workflow | IP-004 |
| [`IP-011-replay-debugger-frontend-full.md`](IP-011-replay-debugger-frontend-full.md) | `oya-workflow-studio-replay-debugger-frontend-{kernel,domain,usecase,api,adapter,sdk}` — consumes engine replay-debugger-backend stream; renders timeline | pending | axis-workflow | IP-004 |
| [`IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md`](IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md) | `oya-workflow-studio-visual-canvas-{usecase,api,adapter,adapter-leptos-wasm,rest,sdk,app}` — Leptos browser-WASM components + editor REST + composition root | pending | axis-workflow + council-design-system | IP-007, IP-009, IP-010, IP-011 |
| [`IP-013-observability-slo-manifests.md`](IP-013-observability-slo-manifests.md) | 4 OpenSLO manifests for workflow-studio self-SLOs (TTI, save-latency, collab-merge, license-gate); consumed by observability promotion gate | pending | axis-workflow + axis-observability | IP-012 |
| [`IP-014-branch-protection-and-hyperscaler-gates.md`](IP-014-branch-protection-and-hyperscaler-gates.md) | `.github/branch-protection.yaml` updates; `/specs/hyperscaler-gates.json` HG-WORKFLOW-STUDIO registration; release pointer creation | pending | axis-workflow + ops-sre-reliability | IP-013 |
| [`IP-015-hg-workflow-studio-registration-final.md`](IP-015-hg-workflow-studio-registration-final.md) | Final HG-WORKFLOW-STUDIO gate registration in `/specs/hyperscaler-gates.json` + competitor-parity-matrix evidence pinning; end-to-end Studio launch verification | pending | axis-workflow + council-architecture | IP-014 |

Coverage check vs. PRD §"Bounded Contexts" layer table: all 52 crates accounted for (9 visual-canvas + 6 dsl-emitter + 6 dsl-loader + 8 collab-crdt + 9 node-library-registry + 5 jurisdiction-overlay-renderer + 6 replay-debugger-frontend + 7 license-gate-cedar; minus 4 redundant counts because some crates serve multiple BCs at composition-root layer = 52 net).

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features --target wasm32-unknown-unknown -p oya-workflow-studio-visual-canvas-adapter-leptos-wasm
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --microservice workflow-studio
oya gate validate lean-a2 --microservice workflow-studio
oya gate validate port-location --microservice workflow-studio
oya gate validate layer-correctness --microservice workflow-studio
oya gate validate per-microservice-layout --microservice workflow-studio
oya gate validate statelessness --microservice workflow-studio
oya gate validate shardability --microservice workflow-studio
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
oya gate validate workflow-spec-roundtrip --microservice workflow-studio --spec-corpus microservices/workflow-studio/capabilities/eval/round-trip-golden-corpus.jsonl
oya gate validate cedar-preview-required --microservice workflow-studio
oya gate validate editor-execution-forbidden --microservice workflow-studio
oya gate validate node-library-determinism --microservice workflow-studio
oya gate validate wasm-bundle-sri --microservice workflow-studio
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Round-trip byte-equality | `cargo nextest run -p oya-workflow-studio-dsl-loader-domain --test test_load_emit_byte_equal` | 100% byte-equal over 100 golden specs |
| Offline buffer durability | `tests/e2e/offline-buffer-resume.rs` | edits survive disconnect; no loss on reconnect |
| Concurrent collab no-loss | `cargo nextest run -p oya-workflow-studio-collab-crdt-domain --test test_no_silent_overwrite` | 10 concurrent users; CRDT merge applied; explicit conflict for overlap |
| Cedar per-seat gate | `cargo nextest run -p oya-workflow-studio-license-gate-cedar-domain --test test_per_seat_cedar` | seat-overage refuses editor open; audit emitted |
| TTI budget | `tests/load/tti-budget.js` (Lighthouse-style synthetic) | p99 ≤ 2s GA |
| Save round-trip latency | `tests/load/save-roundtrip.js` | p99 ≤ 200ms stable; 100ms GA |
| Node-library determinism | `cargo nextest run -p oya-workflow-studio-node-library-registry-domain --test test_load_determinism` | 3x re-load byte-identical |
| WASM bundle SRI | `cargo nextest run -p oya-workflow-studio-visual-canvas-adapter-leptos-wasm --test test_sri` | every chunk has SRI; mismatch refuses load |
| Jurisdiction overlay switch | `cargo nextest run -p oya-workflow-studio-jurisdiction-overlay-renderer-domain --test test_jurisdiction_view_switch` | switch renders overlay; base reachable |
| LLM-assist round-trip | `tests/e2e/llm-assist-validation.rs` | valid draft opens in editor; invalid produces precise per-line error |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice workflow-studio
oya gate validate ontology-type-registry --microservice workflow-studio
```

## Clean Architecture Compliance

Layer assignments and dependency direction (one representative BC; same shape for the other seven BCs):

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-workflow-studio-visual-canvas-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-workflow-studio-visual-canvas-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `oya-workflow-studio-visual-canvas-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-workflow-studio-visual-canvas-api` | `api` | `kernel` | other layers |
| `oya-workflow-studio-visual-canvas-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-workflow-studio-visual-canvas-adapter-leptos-wasm` | `adapter-leptos-wasm` | `adapter`, `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-workflow-studio-visual-canvas-rest` | `rest` | `usecase`, `api`, `domain`, `kernel` | `adapter*` directly (uses ports) |
| `oya-workflow-studio-visual-canvas-sdk` | `sdk` | `api`, `kernel` | adapter/rest/worker/app |
| `oya-workflow-studio-visual-canvas-app` | `app` | (composition-root wiring only) | none — but only wiring |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*` crates. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `workflow-studio` and any other product µservice's kernel/domain/usecase. All cross-product data flow uses SDK boundaries (workflow-engine-sdk, ontology-sdk, foundry-providers-sdk, tenancy-sdk).

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload per IP, written at `microservices/workflow-studio/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "workflow-studio",
  "milestone": "M03-studio-preview",
  "phase": "P01-visual-authoring-substrate",
  "claim_paths": ["microservices/workflow-studio/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/workflow-studio/PRD.md§<section>", "/specs/products/workflow-studio.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "per-microservice-layout", "workflow-spec-roundtrip"],
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
| kernel crate (`*-kernel`) | 1 per public type + 1 per port trait | 0 (pure) | 0 | 90% line; 80% branch |
| domain crate (`*-domain`) | 1 per public function + property tests for round-trip + CRDT invariants | 0 | 0 | 95% line; 90% branch |
| usecase crate (`*-usecase`) | 1 per use case (happy + 2 sad paths) | ≥ 3 against mocked ports | 0 | 90% line; 80% branch |
| adapter crate (`*-adapter*`) | 1 per port-impl method | ≥ 2 against real backend (Postgres / Redis / CDN test container) | 0 | 85% line; 75% branch |
| adapter-leptos-wasm crate | 1 per Leptos component | ≥ 2 component-render tests via wasm-bindgen-test | 0 | 80% line |
| rest crate (`*-rest`) | 1 per route (happy + auth-fail + tenant-mismatch) | ≥ 2 cross-route flows | 1 per route via REST integration test | 85% line; 75% branch |
| worker crate (`*-worker`) | 1 per orchestration arm | ≥ 1 long-lived loop integration test (WS gateway) | 1 e2e (10-user collab drill) | 85% line; 75% branch |
| sdk crate (`*-sdk`) | 1 per public client method (happy + retry + auth-fail) | ≥ 2 against rest crate | 0 | 90% line; 80% branch |
| app crate (`*-app`) | composition-root smoke tests | 0 (delegates to worker/rest tests) | 1 startup-and-shutdown smoke | 60% line (mostly wiring) |
| IaC IPs (Helm / Terraform) | n/a | ≥ 1 helm-install + helm-test smoke per chart | 1 against kind/k3d cluster | n/a |

## branch-protection.yaml diff preview

IP-014 (branch-protection + hyperscaler-gates) updates `.github/branch-protection.yaml` with:

```yaml
branches:
  dev:
    required_status_checks:
      # existing checks plus:
      - oya-governance-workflow-spec-roundtrip            # NEW; from this phase's IP-003 + IP-015
      - oya-governance-cedar-preview-required             # NEW; from this phase's IP-009
      - oya-governance-editor-execution-forbidden         # NEW; Studio never executes
      - oya-governance-node-library-determinism           # NEW; 3x re-load assertion
      - oya-governance-wasm-bundle-sri                    # NEW; from this phase's IP-012

  staging:
    required_status_checks:
      # ADDED by this phase:
      - oya-governance-workflow-spec-roundtrip
      - oya-vcs-promotion-readiness

  # ADDED — pattern-based protection for workflow-studio release pointers
  ? release/workflow-studio/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-vcs-promotion-readiness

  ? release/workflow-studio/production
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-vcs-promotion-readiness
```

## Oya VCS Symbol Locks

Per ADR-0116, this phase uses `oya vcs` primitives exclusively. Grit and ICM are explicitly NOT used.

```bash
cargo run -p oya-dev-cli -- vcs claim \
  --agent <agent-id> \
  --intent "<IP-NNN-slug>: <one-line intent>" \
  --paths "microservices/workflow-studio/src/crates/<crate>/**"

cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/workflow-studio/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0164 (Bominal): Workflow canonical spec format; inherited.
- ADR-0103 (Bominal): Workflow hexagonal migration; inherited.
- ADR-0037 (Bominal): Plugin substrate (WASM); inherited; node-library scaffolding.
- ADR-0056: BNF v4.1.
- ADR-0065: Leptos for browser UI.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0116: Retire external agent-coordination tooling.
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0130: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout + workflow unbundle.
- ADR-0140: Cedar policy enforcement.
- `/specs/products/workflow-studio.json`.
- `/specs/per-microservice-flat-layout.json`.
- `microservices/workflow-studio/PRD.md`.
- Memory: `feedback_workflow_studio_scope.md`, `feedback_workflow_is_shared.md`, `feedback_workflow_objectgraph_adapter_layer.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
