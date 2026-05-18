---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-sheets-preview
phase: P01-sheets-foundation
status: Active
entry_gate: |
  PRD-sheets accepted; ADR-0126 net-new µservice scope accepted; cargo workspace ready to accept
  the new sheets crates under microservices/sheets/src/crates/; Layer-A IaC available via cloud-iac
  µservice (CDN + WebSocket gateway + Postgres + Redis + S3 + Arrow/Parquet via OCI Object Storage);
  foundry-runtime SDK available for AI-formula + smart-fill; tenancy SDK available for per-seat
  licensing; ontology SDK available for object-type descriptors; cell µservice SDK available for
  per-workbook cell substrate; audit-chain SDK available for Ed25519 seals.
exit_gate: |
  All 15 IPs merged; Sheets binary deployed to dev cluster (with WASM bundle on CDN);
  sheets-crdt-no-silent-loss + sheets-formula-engine-correctness + sheets-recalc-determinism +
  sheets-xlsx-roundtrip-best-effort + sheets-range-acl-cedar-required +
  sheets-import-sandboxed-and-avscan-required CI lanes present in .github/branch-protection.yaml
  required_status_checks on dev and staging; release/sheets/{staging,production} pattern
  protection live; XLSX best-effort round-trip drill passes (load 100 golden XLSX workbooks,
  export, byte-compatible per fidelity tier); collab CRDT merge drill passes (10 concurrent
  users, no silent loss); Cedar per-seat gate drill passes; recalc 1M-cell drill passes
  (p95 ≤ 10s); formula-engine ≥ 400 functions corpus-verified against LibreOffice Calc reference
  per ADR-SHEETS-0002; cargo nextest run --workspace exits 0; oya gate validate
  per-microservice-layout --microservice sheets exits 0; oya gate validate authority-cohesion
  exits 0; HG-SHEETS gate in /specs/hyperscaler-gates.json registers green.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: sheets SLO promotion gate must exist before sheets itself can be advanced past dev
  - milestone: M02b-substrate-ready
    phase: P01-cell-substrate
    reason: cell µservice substrate must be live before Sheets can persist cell rows to it
  - milestone: M02b-substrate-ready
    phase: prior phases per master-plan-sequencing
    reason: workspace + branch-protection + Cargo metadata authority must precede Sheets crate authoring
owner_team: axis-sheets + council-design-system
related_adrs: [ADR-0065, ADR-0103, ADR-0126, ADR-0130, ADR-0131, ADR-0140]
related_specs: [/specs/products/sheets.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-sheets-foundation: Land the sheets µservice end-to-end

## Purpose

This phase ships the full sheets µservice — net-new per ADR-0126. The cell-grid editor canvas; the formula engine with ≥400-function library and Excel-reference conformance (ADR-SHEETS-0002); the recalc engine with dependency-graph + parallel-task-graph architecture (ADR-SHEETS-0004); the collaborative CRDT editing with Loro 1.x aligned with workflow-studio ADR-WS-0001 (ADR-SHEETS-0001); the hybrid postgres+Arrow/Parquet large-sheet storage (ADR-SHEETS-0003); pivot tables, charts, conditional formatting, data validation, named ranges; XLSX/ODS/CSV/TSV/JSON-Sheet import/export with best-effort fidelity per ADR-SHEETS-0007 and gVisor + ClamAV/OPSWAT sandboxing; per-range named-ACL per ADR-SHEETS-0006; AI-formula + smart-fill bounded per ADR-SHEETS-0005; connected-sheets external-source queries; comments + version-history; embed-bridge into docs + slides; trigger-bridge to workflow-engine; and per-seat license-gate Cedar enforcement. Delivered as one phase in M03-sheets-preview because Sheets is a hero product surface.

This phase advances master-plan principles:
- Hyperscaler-grade in every practice (CDN-cached WASM bundle + per-tenant CRDT collab + Cedar per-seat + Arrow columnar storage for analytical workloads).
- Nothing deferred (every FUTURE-marked stub in any grid-aware product's authoring UX is decommissioned by this phase's Sheets SDK + emitter).
- No silent regression (sheets-crdt-no-silent-loss CI lane is BLOCKER day 1).
- Per-microservice flat layout (this phase ships natively under ADR-0131).

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected |
|---|---|---|
| `sheets` | `cell-grid`, `formula-engine`, `recalc-engine`, `formatting`, `pivot-tables`, `charts`, `data-validation`, `collab-crdt`, `import-export`, `large-sheet-storage`, `sharing-acl`, `comments`, `version-history`, `named-ranges`, `ai-formula`, `connected-sheets`, `trigger-bridge`, `embed-bridge`, `license-gate-cedar` | All under `microservices/sheets/` per ADR-0131; ~115 crates per PRD §"Layer mapping per BC" |

Plus these repo-wide artifacts (cross-cutting per ADR-0131):
- `.github/branch-protection.yaml` — add `oya-governance-sheets-crdt-no-silent-loss`, `oya-governance-sheets-formula-engine-correctness`, `oya-governance-sheets-recalc-determinism`, `oya-governance-sheets-xlsx-roundtrip-best-effort`, `oya-governance-sheets-range-acl-cedar-required`, `oya-governance-sheets-import-sandboxed-and-avscan-required` to required_status_checks on `dev`; add pattern protection for `release/sheets/{staging,production}`.
- `Cargo.toml` (workspace) — register the new crates under `microservices/sheets/src/crates/`.
- `/specs/hyperscaler-gates.json` — register HG-SHEETS gate per ADR-0123.
- `docs/standards/sheets-grid-canvas.md` (NEW) — cross-cutting standard for Leptos cell-grid authoring (declares deterministic-layout APIs; forbidden patterns: `innerHTML`, `eval`, non-keyed list rendering, server-side formula execution).

Naming justifications for the new crate families are in `microservices/sheets/PRD.md` §"Bounded Contexts" + `microservices/sheets/specs/naming-justifications.md`.

### Out-of-scope

- The cell µservice per-workbook cell substrate — separate µservice (`microservices/cell/`) per ADR-0126.
- Workbook template marketplace — deferred to a post-M03 phase.
- VBA / Apps-Script equivalent — explicitly excluded per ADR-SHEETS-0007 named-limit list; deferred to post-GA T2 review.
- Per-tenant branding (mid-render) — explicitly anti-pattern per `/specs/products/sheets.json` §anti_patterns.
- Strict OOXML round-trip — explicitly deferred per ADR-SHEETS-0007; best-effort tier only at M03.

## Implementation Plans

Ordered list. Each IP is an executable ChangeSet under this phase folder. Dependencies inline.

| IP file | Intent | Status | Owner | Depends on |
|---|---|---|---|---|
| [`IP-001-iac-bootstrap.md`](IP-001-iac-bootstrap.md) | Helm + Kustomize manifests for CDN (+ WAF), Postgres (Citus), Redis (ephemeral CRDT), WebSocket gateway, S3 (snapshots), OCI Object Storage (Arrow/Parquet large-sheet blocks), gVisor sandbox for XLSX export, AV-scan sidecars | pending | axis-sheets + cloud-iac | — |
| [`IP-002-cargo-workspace-cell-grid-kernel-domain.md`](IP-002-cargo-workspace-cell-grid-kernel-domain.md) | `oya-sheets-cell-grid-{kernel,domain}` crates: Workbook, Sheet, Cell, Range, Selection, ViewportState entities + pure cell-graph algebra | pending | axis-sheets + council-design-system | — |
| [`IP-003-formula-engine-kernel-domain-400-functions.md`](IP-003-formula-engine-kernel-domain-400-functions.md) | `oya-sheets-formula-engine-{kernel,domain,usecase,api,adapter,sdk}` with ≥400-function library covering math/logical/lookup/statistical/financial/text/date/array; Excel-reference conformance corpus (LibreOffice Calc reference per ADR-SHEETS-0002) | pending | axis-sheets | IP-002 |
| [`IP-004-recalc-engine-dep-graph-parallel.md`](IP-004-recalc-engine-dep-graph-parallel.md) | `oya-sheets-recalc-engine-{kernel,domain,usecase,api,adapter,worker,sdk}` — dep-graph builder + topological + parallel-task-graph (ADR-SHEETS-0004); 100k-cell ≤ 1s + 1M-cell ≤ 10s p95 | pending | axis-sheets | IP-003 |
| [`IP-005-collab-crdt-loro-aligned-ws-0001.md`](IP-005-collab-crdt-loro-aligned-ws-0001.md) | `oya-sheets-collab-crdt-{kernel,domain,usecase,api,adapter,adapter-loro,adapter-redis,worker,sdk}` — Loro 1.x CRDT merge + WebSocket gateway; align with workflow-studio ADR-WS-0001 | pending | axis-sheets | IP-004 |
| [`IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md`](IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md) | `oya-sheets-large-sheet-storage-{kernel,domain,usecase,api,adapter,adapter-arrow,adapter-parquet,adapter-s3}` per ADR-SHEETS-0003 | pending | axis-sheets | IP-001, IP-004 |
| [`IP-007-cell-grid-adapter-postgres-and-materialized-views.md`](IP-007-cell-grid-adapter-postgres-and-materialized-views.md) | `oya-sheets-cell-grid-adapter-postgres` — workbook metadata + cell storage with materialized-view caches for hot ranges | pending | axis-sheets | IP-002 |
| [`IP-008-formatting-pivot-charts-data-validation.md`](IP-008-formatting-pivot-charts-data-validation.md) | `oya-sheets-{formatting,pivot-tables,charts,data-validation}-*` crate families; custom Leptos canvas chart renderer | pending | axis-sheets + council-design-system | IP-003, IP-004 |
| [`IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md`](IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md) | `oya-sheets-import-export-{kernel,domain,usecase,api,adapter,adapter-calamine,adapter-rust-xlsxwriter,adapter-clamav,adapter-opswat,worker,sdk}` — XLSX pipeline in gVisor; AV-scan sidecars | pending | axis-sheets + ops-security | IP-001, IP-006 |
| [`IP-010-sharing-acl-named-range-cedar.md`](IP-010-sharing-acl-named-range-cedar.md) | `oya-sheets-sharing-acl-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}` + `oya-sheets-named-ranges-{kernel,domain,usecase,api,adapter}` per ADR-SHEETS-0006 | pending | axis-sheets + ops-security | IP-007 |
| [`IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md`](IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md) | `oya-sheets-ai-formula-{kernel,domain,usecase,api,adapter,sdk}` consuming foundry-runtime SDK; T1 advisory + T2 gated per ADR-SHEETS-0005 | pending | axis-sheets + foundry-runtime-team | IP-005, IP-008 |
| [`IP-012-connected-sheets-comments-version-history-trigger-embed-bridge.md`](IP-012-connected-sheets-comments-version-history-trigger-embed-bridge.md) | `oya-sheets-{connected-sheets,comments,version-history,trigger-bridge,embed-bridge}-*` crate families | pending | axis-sheets | IP-007 |
| [`IP-013-cell-grid-rest-leptos-wasm-app-license-gate.md`](IP-013-cell-grid-rest-leptos-wasm-app-license-gate.md) | `oya-sheets-cell-grid-{adapter-leptos-wasm,rest,sdk,app}` + `oya-sheets-license-gate-cedar-*` — Leptos browser-WASM components + editor REST + composition root + Cedar per-seat enforcement | pending | axis-sheets + council-design-system + ops-security | IP-008, IP-010, IP-011, IP-012 |
| [`IP-014-observability-slo-manifests-9-openslo.md`](IP-014-observability-slo-manifests-9-openslo.md) | 9 OpenSLO manifests for sheets self-SLOs (sheet-open, cell-edit-render, recalc-100k, recalc-1m, collab-cursor-sync, export-xlsx, chart-render, crdt-no-silent-loss, formula-engine-correctness); consumed by observability promotion gate | pending | axis-sheets + axis-observability | IP-013 |
| [`IP-015-hg-sheets-registration-and-branch-protection.md`](IP-015-hg-sheets-registration-and-branch-protection.md) | `.github/branch-protection.yaml` updates; `/specs/hyperscaler-gates.json` HG-SHEETS registration; release pointer creation; competitor-parity evidence pinning; end-to-end Sheets launch verification | pending | axis-sheets + council-architecture | IP-014 |

Coverage check vs. PRD §"Bounded Contexts" layer table: all ~115 crates accounted for across the 15 IPs.

## Acceptance Gates

All gates must pass before `exit_gate` is declared.

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features
cargo build --workspace --all-features --target wasm32-unknown-unknown -p oya-sheets-cell-grid-adapter-leptos-wasm
cargo build --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo deny check
cargo doc --workspace --no-deps
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --microservice sheets
oya gate validate lean-a2 --microservice sheets
oya gate validate port-location --microservice sheets
oya gate validate layer-correctness --microservice sheets
oya gate validate per-microservice-layout --microservice sheets
oya gate validate statelessness --microservice sheets
oya gate validate shardability --microservice sheets
oya gate validate authority-cohesion
oya gate validate hyperscaler-maturity-claims
```

### Substrate gates introduced by this phase

```bash
oya gate validate sheets-crdt-no-silent-loss --microservice sheets
oya gate validate sheets-formula-engine-correctness --microservice sheets --corpus microservices/sheets/capabilities/eval/formula-reference-corpus.jsonl
oya gate validate sheets-recalc-determinism --microservice sheets
oya gate validate sheets-xlsx-roundtrip-best-effort --microservice sheets
oya gate validate sheets-range-acl-cedar-required --microservice sheets
oya gate validate sheets-import-sandboxed-and-avscan-required --microservice sheets
oya gate validate cedar-preview-required --microservice sheets
oya gate validate editor-execution-forbidden --microservice sheets
oya gate validate wasm-bundle-sri --microservice sheets
```

### End-to-end drill gates

| Scenario | Command | Pass criterion |
|---|---|---|
| Cell value + SUM | `cargo nextest run -p oya-sheets-formula-engine-domain --test test_sum_basic` | 100×100 grid SUM correct |
| Formula-engine corpus | `cargo nextest run -p oya-sheets-formula-engine-domain --test test_excel_reference_corpus` | ≥ 400 functions; LibreOffice Calc reference behaviour matched |
| XLSX best-effort round-trip | `cargo nextest run -p oya-sheets-import-export-domain --test test_xlsx_best_effort_roundtrip` | 100 golden XLSX workbooks; best-effort tier per ADR-SHEETS-0007 |
| Concurrent collab no-loss | `cargo nextest run -p oya-sheets-collab-crdt-domain --test test_no_silent_overwrite` | 10 concurrent users; CRDT merge applied; explicit conflict for overlap |
| Per-range ACL hides PII | `cargo nextest run -p oya-sheets-sharing-acl-domain --test test_per_range_acl_hides_pii` | named-ACL Cedar policy enforced |
| Cedar per-seat gate | `cargo nextest run -p oya-sheets-license-gate-cedar-domain --test test_per_seat_cedar` | seat-overage refuses workbook open; audit emitted |
| Sheet-open budget | `tests/load/sheet-open-budget.js` | cold p95 ≤ 400ms; warm p95 ≤ 150ms |
| Cell-edit-render budget | `tests/load/cell-edit-render-budget.js` | p99 ≤ 50ms |
| Recalc 100k-cell budget | `tests/load/recalc-100k-budget.js` | p95 ≤ 1s |
| Recalc 1M-cell budget | `tests/load/recalc-1m-budget.js` | p95 ≤ 10s |
| XLSX export budget | `tests/load/xlsx-export-budget.js` | p95 ≤ 5s for 100k-cell workbook |
| Chart render budget | `tests/load/chart-render-budget.js` | p95 ≤ 200ms |
| Smart-fill corpus | `cargo nextest run -p oya-sheets-ai-formula-domain --test test_smart_fill_corpus` | ≥ 80% accuracy on 3-cell-seed corpus |
| Connected-sheets refresh budget | `tests/load/connected-sheets-budget.js` | p95 ≤ 5s for 10k-row materialize |
| AI-formula round-trip | `tests/e2e/ai-formula-validation.rs` | valid draft opens in editor; invalid produces precise per-cell error |
| Offline buffer durability | `tests/e2e/offline-buffer-resume.rs` | edits survive disconnect; no loss on reconnect |
| WASM bundle SRI | `cargo nextest run -p oya-sheets-cell-grid-adapter-leptos-wasm --test test_sri` | every chunk has SRI; mismatch refuses load |
| XLSX import AV-scan + gVisor | `cargo nextest run -p oya-sheets-import-export-adapter-clamav --test test_avscan_required` | ClamAV + OPSWAT scan + gVisor sandbox |

### Workflow + Ontology integration gates

```bash
oya gate validate workflow-event-registry --microservice sheets
oya gate validate ontology-type-registry --microservice sheets
```

## Clean Architecture Compliance

Layer assignments and dependency direction (one representative BC; same shape for the other eighteen BCs):

| Crate (BNF v4.1) | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-sheets-cell-grid-kernel` | `kernel` | (nothing project-internal) | all other layers |
| `oya-sheets-cell-grid-domain` | `domain` | `kernel` | `usecase`, `adapter`, `rest`, `worker`, `app` |
| `oya-sheets-cell-grid-usecase` | `usecase` | `domain`, `kernel` | `adapter`, `rest`, `worker`, `app` |
| `oya-sheets-cell-grid-api` | `api` | `kernel` | other layers |
| `oya-sheets-cell-grid-adapter` | `adapter` | `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-sheets-cell-grid-adapter-postgres` | `adapter-postgres` | `adapter`, `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-sheets-cell-grid-adapter-leptos-wasm` | `adapter-leptos-wasm` | `adapter`, `usecase`, `domain`, `kernel` | `rest`, `worker`, `app` directly |
| `oya-sheets-cell-grid-rest` | `rest` | `usecase`, `api`, `domain`, `kernel` | `adapter*` directly (uses ports) |
| `oya-sheets-cell-grid-sdk` | `sdk` | `api`, `kernel` | adapter/rest/worker/app |
| `oya-sheets-cell-grid-app` | `app` | (composition-root wiring only) | none — but only wiring |

Port traits live exclusively in `*-kernel` crates; implementations exclusively in `*-adapter*` crates. Domain calls through ports; domain never imports adapter.

Cross-product integration check: this phase introduces NO direct imports between `sheets` and any other product µservice's kernel/domain/usecase. All cross-product data flow uses SDK boundaries (cell-sdk, ontology-sdk, foundry-runtime-sdk, tenancy-sdk, audit-chain-sdk, workflow-engine-sdk, docs-sdk, slides-sdk, drive-sdk, forms-sdk, mail-sdk, community-sdk).

## ChangeSet Contract per IP

Every IP in this phase emits a ChangeSet per ADR-0110 (claimable + verifiable + bundleable + promotable). The minimum ChangeSet payload per IP, written at `microservices/sheets/evidence/multispectrum/<change_id>-<unix_ts>.json` on `oya vcs done`:

```json
{
  "change_id": "ULID",
  "ip_id": "IP-NNN-<slug>",
  "microservice": "sheets",
  "milestone": "M03-sheets-preview",
  "phase": "P01-sheets-foundation",
  "claim_paths": ["microservices/sheets/src/crates/<crate>/**", "..."],
  "intent": "<one-line>",
  "spec_refs": ["microservices/sheets/PRD.md§<section>", "/specs/products/sheets.json§<section>"],
  "acceptance_lanes_green": ["cargo-check", "cargo-build", "cargo-clippy", "cargo-nextest", "cargo-deny", "lean-a1", "lean-a2", "per-microservice-layout", "sheets-crdt-no-silent-loss", "sheets-formula-engine-correctness"],
  "test_count": {"unit": <int>, "integration": <int>, "e2e": <int>},
  "coverage_pct": <float>,
  "multispectrum_review_facets": ["F1..F9", "A1..A7", "M1..M2"],
  "signature": "Ed25519:<sig>",
  "executed_at": "ISO8601"
}
```

## Per-IP Test Coverage Threshold

Matches workflow-studio PHASE-01 thresholds (kernel 90% / domain 95% / usecase 90% / adapter 85% / adapter-leptos-wasm 80% / rest 85% / worker 85% / sdk 90% / app 60% / IaC ≥ 1 helm-install + helm-test smoke per chart).

## branch-protection.yaml diff preview

IP-015 updates `.github/branch-protection.yaml` with:

```yaml
branches:
  dev:
    required_status_checks:
      - oya-governance-sheets-crdt-no-silent-loss
      - oya-governance-sheets-formula-engine-correctness
      - oya-governance-sheets-recalc-determinism
      - oya-governance-sheets-xlsx-roundtrip-best-effort
      - oya-governance-sheets-range-acl-cedar-required
      - oya-governance-sheets-import-sandboxed-and-avscan-required
      - oya-governance-cedar-preview-required
      - oya-governance-editor-execution-forbidden
      - oya-governance-wasm-bundle-sri

  staging:
    required_status_checks:
      - oya-governance-sheets-crdt-no-silent-loss
      - oya-governance-sheets-formula-engine-correctness
      - oya-vcs-promotion-readiness

  ? release/sheets/staging
  :
    require_pull_request: false
    require_linear_history: true
    disallow_force_push: true
    require_signed_commits: true
    require_signed_tags: true
    required_status_checks:
      - oya-vcs-promotion-readiness

  ? release/sheets/production
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
  --paths "microservices/sheets/src/crates/<crate>/**"

cargo run -p oya-dev-cli -- vcs verify --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs done --agent <agent-id> --changeset <id>
cargo run -p oya-dev-cli -- vcs promote --changeset <id>
```

Multispectrum evidence per docs/AGENTS.md §changeset: each IP emits `microservices/sheets/evidence/multispectrum/<change_id>-<unix_ts>.json` per `/specs/multispectrum-review.json` v2.4.0.

## References

- ADR-0056: BNF v4.1.
- ADR-0065: Leptos for browser UI.
- ADR-0103 (Bominal): hexagonal migration; inherited.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0116: Retire external agent-coordination tooling.
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0126: Sheets net-new µservice (no legacy connect-sheets).
- ADR-0130: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- ADR-SHEETS-0001..0007 (local).
- `/specs/products/sheets.json`.
- `/specs/per-microservice-flat-layout.json`.
- `microservices/sheets/PRD.md`.
- Memory: `feedback_workflow_studio_scope.md`, `feedback_workflow_is_shared.md`, `feedback_workflow_objectgraph_adapter_layer.md`, `feedback_clean_architecture_requirements.md`, `feedback_quality_performance_scalability_bar.md`.
