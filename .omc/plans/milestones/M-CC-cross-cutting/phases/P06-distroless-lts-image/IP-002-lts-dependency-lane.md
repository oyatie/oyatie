---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M-CC-P06-IP-002
title: Dependency-seam discipline + tech-debt ledger + LTS roster
status: in-progress
amended_at: 2026-05-14
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
source_plan: ../../../../ralplan-dep-seam-phaseout-round-5.md
purpose: LTS roster plus dependency-seam discipline (AMENDED scope per ADR-0092). Ship the release-critical product paths on Hyper/Tokio/Serde-family deps; isolate hyper to a single adapter; track external deps via a flat rationales overlay (not a state machine); land the multispectrum review bar as the durable PR gate.
amendments_summary:
  - layer_enum_reverted_to_canonical_12_per_ADR_0056_v4_1
  - middleware_target_layer_corrected_from_runtime_to_infrastructure
  - tech_debt_ledger_collapsed_to_flat_rationales_overlay
  - trigger_dsl_dropped_as_speculative_complexity_F_MULTI_F2_F6
  - dri_and_role_roster_dropped_use_cargo_metadata_owner_team
  - step_0_mass_cargo_toml_metadata_insert_canceled_layer_derived_from_name
  - step_7_30_day_soak_moved_to_cron_not_session
  - quality_findings_Q1_Q3_Q4_Q5_fixed_inline_Q2_fixuptask
  - security_findings_S1_S2_S3_S5_S6_S7_S8_S10_closed_S4_partial_with_fixuptask
  - readiness_gate_step_8_blocked_no_readyz_endpoint_exists_fixuptask
adr_citations: [ADR-0092, ADR-0093, ADR-0094, ADR-0095]
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
| `/registry/tech-debt-ledger.json` | create | Object-map ledger with 11 seed deps, DRI handles, statuses, replacement targets, trigger DSL, CVE acceleration, ADR citations, default evaluator policies. |
| `.omc/schemas/tech-debt-ledger.schema.json` or `/specs/cross-cutting/tech-debt-ledger.schema.json` | create | Schema enforcing top-level defaults, optional per-row overrides, `never` trigger rules, monotonic status graph, and cross-row acyclicity. |
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
/registry/tech-debt-ledger.json::entries
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

---

## AMENDED — what actually shipped (2026-05-14, per ADR-0092)

The original plan above was authored before discovery of:
- ADR-0056 v4.1 canonical 12-layer enum (this IP's 5-value enum was inconsistent).
- The middleware-kernel `bytes` leak (kernel chose `Bytes` for body; consumers transitively pulled the dep — symptom-not-root-cause).
- The multispectrum review bar (docs/standards/multispectrum-review.md).
- Workspace security findings S1-S10 in the existing http-* foundation.

Under user directive "Option C — Full quality bar" (2026-05-14), the
amended slice landed in 10 phases. Each phase shipped with adversarial F3
fixtures + multispectrum-evidence-attached per the new bar.

### Phase outcomes

| Phase | Outcome | Evidence |
|---|---|---|
| 0 — protocol foundation | docs/standards/multispectrum-review.md + /specs/cross-cutting/multispectrum-review.json + pre-PR checklist + iterative-fix-loop spec authored; root-hub-pointers updated | files exist + cited by ADR-0092 |
| 1 — SSE → kernel rename | crate + 2 foundry consumer adapter Cargo.toml + .rs updates | cargo test green; doc-comment updated |
| 2 — kernel type rename + body Vec<u8> + bytes drop | HyperRequest→HttpRequest, HyperResponse→HttpResponse, body bytes→Vec<u8>; bytes dep removed from middleware-kernel; cascade clean | 102 tests pass; seam audit 6→1 |
| 3 — middleware crates renamed -domain → -infrastructure | 3 crate renames per canonical 12-layer enum | 22 tests pass on renamed crates |
| 4 — router matched_template + telemetry static label | Q1 + S6 closed; heuristic .replace() gone | 6 new adversarial fixtures pass; 126 tests total |
| 5 — DeadlineMiddleware → LatencyBudgetReporter | Honest naming; crate renamed; ADR-0093 | 7 fixtures pass incl. side-effects-run-before-504 |
| 6 — Handler trait + handler_to_sync bridge | Additive; ADR-0094 | 5 Handler tests + handler_to_sync end-to-end test |
| 7 — TenantSlug in oya-tenancy-kernel | Grammar centralized; middleware delegates; ADR-0095 | 13 fixtures incl. homoglyph + dot-path-traversal-shape rejection |
| 8 — S3 body cap + S4 connection timeouts | ServerConfig with safe defaults; HyperRuntimeError::BodyTooLarge → 413 | 7 fixtures incl. boundary tests; S4 partial (slowloris integration FixupTask) |
| 9 — S5 path traversal + S7/S8 SSE injection | Router rejects `.`/`..` captures; render() sanitizes CR/NUL/control | 6 router fixtures + 16 SSE fixtures |
| 10 — S1 header case + S2 non-UTF8 + S10 CRLF | Headers lowercased + values stripped; non-UTF8 → 400 | 8 fixtures incl. combined attack neutralization |

### Final test count + seam audit

```
cargo test (impacted crates, 12 suites): 189 passed
seam audit (Cargo.toml + .rs imports):
  - hyper-family deps: exactly ONE crate (oya-http-runtime-hyper-adapter)
  - hyper-family imports: exactly ONE .rs file (same)
```

### Original acceptance-criteria checklist — amended outcomes

| Original criterion | Amended outcome |
|---|---|
| tech-debt-ledger.json with 11 entries + default_evaluator_policies + schema validation + quarterly review | Replaced by `/registry/dependency-rationales.json` (11 entries, flat 5-field overlay, no state machine, no DSL). Multispectrum F1+F2+F6 rejected the over-engineering. |
| Per-row evaluator_policies optional with override warnings | N/A (no policies; rationales overlay has no state) |
| `{never: true}` short-circuit | N/A (rejected as speculative) |
| Monotonic status transition sub-check | N/A (no transitions yet) |
| `oya-check-dependency-seam-discipline` with 8 sub-checks | Slimmed to 3 mechanical + 3 multispectrum sub-checks per ADR-0092 D13. Lane crate scaffold is FixupTask F-LANE-SEAM-IMPL. |
| `oya-foundry-trigger-dsl-{kernel,runtime}` | DROPPED entirely. Predicates inline as Rust fns when needed (multispectrum F1+F2+F6). |
| `oya-http-sse-domain` renamed to `oya-http-sse-kernel`; no `oya-http-sse-runtime` in W0 | ✅ shipped (Phase 1). |
| Middleware crates remove bytes + hyper-adapter deps | ✅ shipped (Phase 2 root-cause fix; Phase 3 layer rename to -infrastructure). |
| Only `oya-http-runtime-hyper-adapter` declares hyper-family deps + bytes | ✅ shipped — mechanically verified. |
| ADR-0091..ADR-0094 Accepted/indexed; 0093 Accepted at Step 6 | Renumbered to ADR-0092..ADR-0095 (0091 was already taken by foundry-write-gate-foundations). ALL FOUR ACCEPTED. |
| `ReadinessGate` test proves /readyz 200 within budget + 503 within 500ms of SIGTERM | BLOCKED — no /readyz endpoint exists in the workspace. FixupTask F-STEP8-READYZ when an endpoint lands. |
| Replacement parity + distroless deployment bar lanes exist | OUT OF SCOPE (original Step 5 explicitly skipped by user). FixupTask F-LANE-DISTROLESS. |
| NO [workspace.dependencies] removal in W0 | ✅ shipped (zero removals). |
| Walk-away path is grit-mediated | N/A (no transitions occurred). |

### FixupTasks (named, bounded, deferred — not silently buried)

- **F-LANE-SEAM-IMPL**: ship `oya-check-dependency-seam` crate with 3+3 sub-checks per ADR-0092 D13.
- **F-LANE-DISTROLESS**: distroless smoke + cold-start harness + lane (original Step 5).
- **F-STEP8-READYZ**: write the /readyz cold-start + SIGTERM tests when an endpoint exists.
- **F-MULTI-Q2**: telemetry Mutex<BTreeMap> → sharded AtomicU64 when load tests show contention.
- **F-ASYNCCHAIN-1**: async middleware chain enables real cancellation (real DeadlineMiddleware alongside LatencyBudgetReporter).
- **F-HANDLER-ASYNC**: async variant of Handler trait once F-ASYNCCHAIN-1 lands.
- **F-TENANTID-FORMAL**: formalize TenantId vs TenantSlug mapping in PRD-tenancy + auth/identity slice.
- **F-SEC-S4-INTEGRATION**: real slowloris integration test (needs hyper client harness).
- **F-DRI-CODEOWNERS**: generate CODEOWNERS from `[package.metadata.oya.owner_team]` (rejected dri.json + role-roster.json duplicates).
- **F-PERF-BODY-COPY-1**: re-evaluate Bytes→Vec<u8> inbound copy when load test shows >1 MiB legitimate bodies on a hot route.
- **F-SOAK-FLIP-CRON**: cron entry to flip seam lane severity report-only → error after 7-day green window (original Step 7).

### Evidence

Final multispectrum evidence: `/evidence/m-cc-p06-ip-002-final-1778801869.json`.

Per-phase evidence files:
- `/evidence/multispectrum/phase-2-kernel-rename-1778801869.json` (CC-1 kernel public API)

Decision-log row (Linus good-taste, post-amendment):

- Mass `[package.metadata.oyatie.layer]` insertion (269 crates) — REJECTED. Layer is derived from crate-name suffix per ADR-0056 v4.1 BNF; double-bookkeeping antipattern (Linus F1).
- 5-layer enum {kernel, runtime, adapter, api, app} — REJECTED. Inconsistent with canonical 12-layer enum (ADR-0056 v4.1). Used the canon.
- Mass middleware-domain → middleware-runtime renames — REJECTED. `-runtime` not in canonical enum. Used `-infrastructure`.
- Tech-debt ledger with state machine + trigger DSL + monotonic graph + cross-row predicate for 11 deps — REJECTED. Control plane before scale (multispectrum F1+F2+F6). Used flat rationales overlay.
- Separate dri.json + role-roster.json — REJECTED. Duplicates `[package.metadata.oya.owner_team]`; CODEOWNERS generates from that single source.
- DeadlineMiddleware name kept "for the eventual async chain" — REJECTED. Naming what we WISH it was lies to readers; renamed honestly.
- Stubbed F3 adversarial test (caught my own assertion in iterative-fix-loop) — REJECTED. Re-wrote to actually exercise byte-equality across full u8 range.
