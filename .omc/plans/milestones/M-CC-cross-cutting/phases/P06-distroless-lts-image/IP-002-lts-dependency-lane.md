---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P06-IP-002
title: Dependency-seam discipline + tech-debt ledger + LTS roster
status: expanded-from-round-5-findings
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
source_plan: ../../../../ralplan-dep-seam-phaseout-round-5.md
purpose: LTS roster plus dependency-seam discipline: release-critical deps may ship first, but only behind layer seams, ledger ownership, machine-evaluable triggers, replacement parity, and CI enforcement.
---

# M-CC-P06-IP-002 — Dependency-seam discipline + tech-debt ledger + LTS roster

## Purpose

Fold `ralplan-dep-seam-phaseout-round-5.md` into the canonical M-CC hierarchy. The rule is: ship the 5 release-critical product paths on Hyper/Tokio/Serde-family deps first, but prevent accidental debt growth by enforcing wrapper/newtype seams, layer metadata, tech-debt ledger rows, replacement triggers, CVE acceleration, and parity evidence now.

This IP owns the dependency-seam and phase-out control surface. It does **not** remove Hyper/Tokio/Serde/etc. from `[workspace.dependencies]`; removals are out of scope until the trigger-based phase-out path arms after Ontology v1 stability or CVE acceleration.

## Source findings folded

- Source: `.omc/plans/ralplan-dep-seam-phaseout-round-5.md` (round 5 final)
- Chosen option: triggered incremental phase-out with GREEN/AMBER/RED parity bands.
- Defect closed: `oya-http-sse-domain` must not become `runtime` because that would create adapter→runtime confusion. It becomes `oya-http-sse-kernel` now; future `oya-http-sse-runtime` is ADR-gated only when reconnect/keepalive/backpressure orchestration exists.
- Defect closed: trigger policy example inherits top-level `default_evaluator_policies` instead of repeating all 4 fields per row.
- Defect closed: `oya-foundry-trigger-dsl` splits into `kernel` (pure AST/policy/status graph) and `runtime` (file + CI evidence resolution).
- Defect closed: `ReadinessGate` flip is pinned and testable.

## Concrete file targets

| Path | Action | Description |
|---|---|---|
| `.omc/registries/tech-debt-ledger.json` | create | Object-map ledger with 11 seed deps, DRI handles, statuses, replacement targets, trigger DSL, CVE acceleration, ADR citations, default evaluator policies. |
| `.omc/schemas/tech-debt-ledger.schema.json` or `.omc/specs/tech-debt-ledger.schema.json` | create | Schema enforcing top-level defaults, optional per-row overrides, `never` trigger rules, monotonic status graph, and cross-row acyclicity. |
| `crates/oya-check-dependency-seam-discipline/` | create | Composite runtime lane with 8 sub-checks. |
| `crates/oya-foundry-trigger-dsl-kernel/` | create | Pure AST parser, predicate registry types, evaluator policy enums, status-transition graph validator. Zero I/O. |
| `crates/oya-foundry-trigger-dsl-runtime/` | create | Git-toplevel file URI resolver, CI evidence fetcher, composite evaluator; consumes kernel crate. |
| `crates/oya-http-sse-kernel/` | rename/create | Rename from `oya-http-sse-domain`; pure `SseEvent` types + serializer; layer=`kernel`. |
| `crates/oya-http-{deadline,telemetry,tenant}-middleware-runtime/` | rename/refactor | Rename from `*-domain`; remove `bytes` and hyper-adapter deps; consume kernel newtypes only. |
| `crates/oya-http-router-kernel/` | update | Add `HttpHeaderMap`, `HttpHeaderName`, `HttpHeaderValue` newtypes. |
| `crates/oya-http-runtime-hyper-adapter/Cargo.toml` | update | Depend on `oya-http-sse-kernel`; remain the only crate declaring hyper-family deps and `bytes`. |
| `Cargo.toml` + `Cargo.lock` | update | Workspace member renames and lockfile regeneration in one atomic grit claim. |
| `.omc/reports/dependency-seam-latest.json` | create | Composite lane report schema enumerating all 8 sub-checks. |
| `.omc/fitness-lanes/INDEX.md` | update | Add 3 top-level lanes after SHA-anchored baseline recompute: dependency seam, replacement parity, distroless deployment bar. |
| `docs/decisions/ADR-0091-workspace-dependency-seam-debt-ledger-phaseout.md` | create | Accepted after consensus; covers ledger, triggers, phase-out, walk-away. |
| `docs/decisions/ADR-0092-*` | create | Lane runner vs kernel naming split. |
| `docs/decisions/ADR-0093-*` | create/update | CI-only read-side gh carve-out; Proposed at Step 4, Accepted only at Step 6 with CODEOWNERS + same-PR guard. |
| `docs/decisions/ADR-0094-*` | create | SSE kernel/runtime split rationale and future runtime creation criteria. |
| `docs/standards/lts-versions.md` | create/update | LTS roster and ADR exception link surface. |

## Dependency seam contract

### Layer enum for this IP

`{kernel, runtime, adapter, api, app}`. No `domain` layer in this dependency-seam plan.

### Allowed edges

- `adapter -> kernel` (e.g. hyper adapter consumes router/middleware/SSE kernels)
- `runtime -> kernel` (middleware runtimes consume kernel newtypes/traits)
- `adapter -> runtime` (adapter may compose runtime middleware)

### Banned edges

- `runtime -> adapter`
- `kernel -> runtime|adapter|api|app`
- Public API signatures exposing `hyper::*`, `tokio::*`, `bytes::*`, or `http_body::*` outside adapter-owned boundaries.

## Tech-debt ledger requirements

Seed rows: `hyper`, `hyper-util`, `http-body-util`, `tokio`, `bytes`, `tracing`, `tracing-subscriber`, `serde`, `serde_json`, `toml`, `toml_edit`.

Required top-level fields:

```json
{
  "version": "1.2.0",
  "reviewed_quarterly": true,
  "workspace_member_count_source": "cargo metadata --no-deps (dynamic)",
  "workspace_dependencies_source": "file:Cargo.toml:/workspace/dependencies",
  "default_evaluator_policies": {
    "staleness_policy": "fail-on-stale",
    "evidence_immutability_policy": "run-id-required",
    "pointer_missing_policy": "not-yet-armed",
    "monotonic_transitions_only": true
  },
  "entries": {}
}
```

Rows inherit `default_evaluator_policies`; per-row overrides are allowed but emit structured warnings. `{"never": true}` short-circuits the 4 trigger policies and is permitted only for `keep` or `replacement-attempted-abandoned` statuses.

Status graph: `active -> scheduled|replacement-armed|replacement-armed-by-cve|keep`; `scheduled -> replacement-armed|replacement-armed-by-cve|keep`; armed states -> `replaced|replacement-attempted-abandoned`; terminal states require fresh ADR to reopen.

Exactly one cross-row predicate is allowed: `dependent_wave_status`; evaluator rejects cycles via topological sort.

## Composite lane: `oya-check-dependency-seam-discipline`

Sub-checks:

1. Seam import/public-signature check.
2. Ledger coverage check.
3. Ledger freshness check.
4. Vendor residue check.
5. CVE-watch check.
6. Ledger review-contract check.
7. Layer-metadata check.
8. Ledger-transition-monotonicity check.

Comparator for monotonic transitions: PR parent merge-base on `main` (`git merge-base origin/main HEAD`) inside the lane implementation only. Agents still do not call git directly; the lane/tool owns the read-side primitive.

Severity ramp: Day 0-30 report-only; Day 30+ error/BLOCKER after soak and green evidence.

## W0 execution steps

0. Layer-metadata bootstrap across workspace; every member declares `[package.metadata.oyatie.layer]` in `{kernel,runtime,adapter,api,app}`; no `domain` in this plan.
1. SSE kernel split + middleware-runtime refactor + crate renames + adapter dependency removal; update `Cargo.toml` and `Cargo.lock` atomically.
2. Author `oya-check-dependency-seam-discipline` composite lane and fixtures.
3. Commit ledger, schema, generator, self-heal, object-map envelope, inheritance defaults.
4. Author `oya-foundry-trigger-dsl-{kernel,runtime}` and draft ADR-0091..ADR-0094.
5. Add distroless smoke, cold-start harness, and `oya-check-distroless-deployment-bar`.
6. Add `dri.json`, `role-roster.json`, CODEOWNERS guard, same-PR self-promotion fixture, and accept ADR-0093.
7. Flip lane to error after 30-day soak; update `.omc/fitness-lanes/INDEX.md`; add quarterly review template.
8. Apply ops-binary cloud-native changes and `ReadinessGate` tests.

Hard dependency: Step 0 before Step 1. Steps 2-8 fan out after Step 1; Step 7 depends on Steps 2-6.

## Acceptance criteria

- [ ] `tech-debt-ledger.json` exists with 11 entries, top-level `default_evaluator_policies`, schema validation, and quarterly review metadata.
- [ ] Per-row `evaluator_policies` optional; inheritance fixture passes; override fixture emits structured warning.
- [ ] `{"never": true}` trigger short-circuits policies and is accepted only for terminal keep/abandoned rows.
- [ ] Monotonic status transition sub-check rejects backward moves; `monotonic_transitions_only: false` emits `monotonicity-disabled:<dep>` warning.
- [ ] `oya-check-dependency-seam-discipline` exists with all 8 sub-checks.
- [ ] `oya-foundry-trigger-dsl-kernel` and `oya-foundry-trigger-dsl-runtime` exist with correct layer metadata and fixtures.
- [ ] `oya-http-sse-domain` renamed to `oya-http-sse-kernel`; no `oya-http-sse-runtime` in W0.
- [ ] `oya-http-{deadline,telemetry,tenant}-middleware-runtime` crates remove `bytes` and hyper-adapter deps.
- [ ] Only `oya-http-runtime-hyper-adapter` declares hyper-family deps and `bytes` after W0.
- [ ] ADR-0091, ADR-0092, ADR-0094 accepted/indexed; ADR-0093 accepted only at Step 6.
- [ ] `ReadinessGate` test proves `/readyz` returns 200 within cold-start budget after catalog loaded/bind success and 503 within 500ms of SIGTERM.
- [ ] Replacement parity and distroless deployment bar lanes exist; top-level lane count delta is SHA-anchored.
- [ ] No `[workspace.dependencies]` removal in W0; ADR-0090 not superseded.
- [ ] Walk-away path is grit-mediated: `replacement_trigger={"never": true}` + ADR-0091 amendment + `replacement-attempted-abandoned` enum.

## Symbols-to-grit-claim

```text
.omc/registries/tech-debt-ledger.json::entries
crates/oya-check-dependency-seam-discipline/src/lib.rs::check
crates/oya-foundry-trigger-dsl-kernel/src/lib.rs::TriggerDslAst
crates/oya-foundry-trigger-dsl-runtime/src/lib.rs::evaluate
crates/oya-http-sse-kernel/src/lib.rs::SseEvent
crates/oya-http-runtime-hyper-adapter/Cargo.toml::dependencies
Cargo.toml::workspace.members
.omc/fitness-lanes/INDEX.md::lanes
```

Scaffold-lock via ADR-0054 if symbols are not registered.

## Acceptance-test-commands

```bash
cargo test -p oya-check-dependency-seam-discipline --all-features
cargo test -p oya-foundry-trigger-dsl-kernel --all-features
cargo test -p oya-foundry-trigger-dsl-runtime --all-features
cargo check --workspace --all-features
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo deny check
oya-dev-cli gate validate dependency-seam --mode=composite --offline
oya-dev-cli gate validate ledger-coverage
oya-dev-cli gate validate fitness-lane-index --baseline-sha <merge-base-or-emitted-baseline>
scripts/check.sh
```

## Done-criteria

- All acceptance-test commands return 0 or are recorded as blocked/gap with reason.
- No provider/runtime dependency leaks outside adapter-approved crates.
- All direct deps current LTS or ADR/ledger-tracked exception.
- PR good-taste audit cites removed special cases and rejected over-engineering.
- ICM completion row records lane status, ledger row count, ADR states, and grit done closeout.

## Rollback / walk-away

`grit done` is atomic per-symbol. Replacement abandonment uses `replacement-attempted-abandoned` plus ADR-0091 amendment and `{"never": true}`; no direct `git`, `gh`, or manual rollback PR.

## Icm-store-payload

```bash
icm store -t context-oyatie -c 'M-CC-P06-IP-002 dependency-seam discipline shipped; tech-debt ledger rows=11; composite lane green; trigger DSL split; ADR-0091..0094 indexed; LTS roster updated' -i high -k 'M-CC-P06-IP-002,dependency-seam,lts,complete'
```

## Decision-log (Linus good-taste row)

Special cases eliminated by this IP:

- Per-row trigger policy boilerplate replaced by registry-level defaults with explicit override warnings.
- SSE runtime misclassification rejected; current code is pure serializer kernel, future runtime is ADR-gated.
- Direct calendar phase-out rejected; machine-evaluable triggers drive transitions.
- New coordination kernel rejected; grit remains the state-transition primitive, ledger is evidence/phaseout data only.
