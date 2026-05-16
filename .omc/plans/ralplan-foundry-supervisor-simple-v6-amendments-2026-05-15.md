---
status: pending approval
mode: deliberate
iteration: 6
type: amendment-addendum
amends: ralplan-foundry-supervisor-simple-v4-2026-05-14.md + ralplan-foundry-supervisor-simple-v5-delta-settings-template-2026-05-15.md
supersedes_consensus: 11-facet v2.2.0 + F10/F11 (2-lens prior-doctrine v4 approval
  is grandfathered)
owner_phase: M02-P06
multispectrum_doctrine: v2.2.0 enforced; v2.3.0 A-family deferred via F-MULTISPECTRUM-A-FAMILY-BACKFILL-M02-P06
  (sunset 2026-07-15)
audit_chain_event: consensus_debate_complete @ 2026-05-15T03:30:00Z (event
synthesis_evidence: evidence/debate/M02-P06-FOUNDRY-SUPERVISOR-2026-05-15-synthesis.json
fixuptasks_filed: 58 (12 BLOCKER, 22 HIGH, 18 MED, 6 LOW) at registries/cross-cutting/fixuptasks.jsonl
verdict: ACCEPT-WITH-FIXUPS (12/13 facets concur; M1 REJECT-on-v5 procedurally resolved
  via 3 FixupTasks)
purpose: Auto-backfilled purpose for ralplan-foundry-supervisor-simple-v6-amendments-2026-05-15.md
---
# v6 Amendments — Surgical §-Anchored Diffs

This file is **not a re-plan**. It is the executable diff list that converts the prior-doctrine v4 + v5-delta approvals into v2.2.0-compliant artifacts before the first grit claim opens against M02-P06.

The 14 implementation tasks already created in team `foundry-supervisor` (Wave 1 + 2a-f + 3a-c + 4a-c + 5) remain VALID. This file adds **18 BLOCKING pre-scaffold edits** (12 BLOCKERs + 6 pre-scaffold conditions) that must apply against v4 + v5 before workers begin code mutation.

## A. Pre-scaffold conditions (6) — must land FIRST

### PRE-1 — Rename `oya-foundry-settings-template-adapter-fs` → `oya-foundry-settings-template-adapter`
**Source:** F-SETTINGS-ADAPTER-RENAME-BNF-CONFORMANT-1 (CONV-8, M2 + F1)
**Diff target:** v5 §B.1 crate decomposition + v5 §B.2.2 trait module path + every reference in v5.

**Before** (v5 §B.1):
```
oya-foundry-settings-template-adapter-fs
```

**After:**
```
oya-foundry-settings-template-adapter
```

**Rationale:** ADR-0056 v4.1 closed 12-layer enum admits `adapter` as final segment; `-fs` is not a recognized layer suffix. Sibling precedent: `oya-foundry-jsonl-supervisor-adapter` (BC-tokens=[jsonl,supervisor] + layer=adapter, no `-fs` suffix). Per project memory `feedback_naming_justification.md`.

---

### PRE-2 — Register 3 templates + 2 crates + 1 lane + 1 verifier in `artifact-capabilities-registry.json`
**Source:** F-ADR-0069-ARTIFACT-CAPABILITIES-REGISTER-1 (CONV-9, M2)
**Diff target:** Add Wave 4d grit unit `v5.19-register-artifact-capabilities` in v5 §B.7.

**New rows to add to `/registries/cross-cutting/artifact-capabilities-registry.json`:**
```json
{"artifact_id":"foundry-supervisor-template-claude","artifact_path":"templates/foundry-supervisor/claude.toml","artifact_profile":"template","capability_overrides":{...9 cap rows per ADR-0069...}},
{"artifact_id":"foundry-supervisor-template-codex","artifact_path":"templates/foundry-supervisor/codex.toml","artifact_profile":"template",...},
{"artifact_id":"foundry-supervisor-template-gemini","artifact_path":"templates/foundry-supervisor/gemini.toml","artifact_profile":"template",...},
{"artifact_id":"foundry-supervisor-kernel","artifact_path":"crates/oya-foundry-supervisor-kernel","artifact_profile":"kernel-crate",...},
{"artifact_id":"foundry-settings-template-kernel","artifact_path":"crates/oya-foundry-settings-template-kernel","artifact_profile":"kernel-crate",...},
{"artifact_id":"lean-settings-drift","artifact_path":"registry/quality/lanes/lean-settings-drift.json","artifact_profile":"ci-lane",...},
{"artifact_id":"oya-dev-cli-settings-drift","artifact_path":"crates/oya-dev-cli/src/commands/settings_drift.rs","artifact_profile":"verifier",...}
```

Each `capability_overrides` declares Enforcement / Verification / Validation / Autogen / Self-healing / Self-updating / Self-maintaining / Knowledge-graph-edges / Claim-matrix per ADR-0069 v3.0.0. `Autogen.idempotency_class = content-addressed-blake3` matches the renderer's blake3 hashing.

---

### PRE-3 — Register reusable building blocks
**Source:** F-REUSABLE-BUILDING-BLOCKS-REGISTER-1 (CONV-9, M2)
**Diff target:** Add grit unit `v5.20-register-building-blocks` in v5 §B.7.

**New rows in `/registries/cross-cutting/reusable-building-blocks-registry.json`:**
- `SettingsTemplate` @ `crates/oya-foundry-settings-template-kernel/src/lib.rs::SettingsTemplate` — consumers: [oya-foundry-supervisor-app], version: 1, deprecation: null
- `SettingsRenderer` @ same crate — consumers: [oya-foundry-supervisor-app, oya-foundry-settings-template-adapter]
- `HookEvent` @ same crate — consumers: [3 driver impls, settings-template-adapter]
- `McpServerRef` @ same crate — consumers: [settings-template-adapter, anyone wiring MCP servers per-account]

**Rationale:** ADR-0069 DRY enforcement. Workflow Studio settings rendering in M03+ will reuse these primitives.

---

### PRE-4 — Add 4 knowledge-graph edges
**Source:** F-KNOWLEDGE-GRAPH-EDGES-DECLARE-1 (CONV-4 + CONV-9, M2 + F11)
**Diff target:** Add grit unit `v5.21-knowledge-graph-edges` in v5 §B.7.

**New edges in `/registries/cross-cutting/knowledge-graph-dynamic.json`:**
```
(oya-foundry-settings-template-kernel) --generates--> (per-account on-disk settings files)
(oya-foundry-settings-template-kernel) --consumes--> (registry/accounts/*.toml)
(lean-settings-drift) --enforces--> (foundry-supervisor-template-*)
(SettingsRenderer) --reads--> (SecretStorePort @ oya-foundry-account-adapter-openbao)
```

---

### PRE-5 — Document v5 N-threshold for drift lane activation
**Source:** F-V5-N-THRESHOLD-DOCUMENT-1 (TEN-2, M1 REJECT resolution)
**Diff target:** v5 §A.1.2 + §C.26.

**Add to v5 §A.1.2:**
> "Drift-detection lane (`lean-settings-drift`) is **feature-flagged off in production** until `sum(rows across registry/accounts/*.toml) ≥ 10`. Before threshold: lane runs against fixture files only in CI (passes vacuously for empty fleet). Threshold is checked at lane start via `wc -l registry/accounts/*.toml | tail -1 | awk '{print $1}'`."

**Add to v5 §C.26 acceptance row:** "Lane exit 0 against an empty `registry/accounts/` is acceptable; not a false-positive — threshold gating documented in §A.1.2."

**Rationale:** M1-F1 CRITICAL — PREMISE-5 (drift across N accounts) has N=0 today. Feature-flag prevents lane from operating on imagined-future-scale.

---

### PRE-6 — Add Siigari/claude-heartbeat build-vs-adopt ADR
**Source:** F-CLAUDE-HEARTBEAT-BUILD-VS-ADOPT-ADR-1 (TEN-2, M1)
**Diff target:** v4 §A.0.1 (new) + new file `docs/decisions/ADR-NNNN-supervisor-language-rust-not-node.md`.

**Add to v4 §A.0.1:**
> "**Build-vs-adopt analysis for Siigari/claude-heartbeat** (cited as 'Reference shape' at v4 line 18): Per ADR-NNNN-supervisor-language-rust-not-node, the upstream Node implementation was considered as a sibling sidecar (Rust crates speak JSONL to Node inbox/outbox). Rejected because: (a) workspace-language-purity is an ADR-tracked principle — `oya-*` crates are Rust-native; (b) supervisor's deep composition with `RoutePolicy::select`, `UsageEnforcement::check_limit`, `validate_usage`, `finalize_line`, `check_silent_switch`, `enforce_for_tenant` requires sharing kernel types, which a Node sidecar cannot do without an IPC bridge; (c) autonomy_ceiling Cedar enforcement and audit-chain emission must execute in the same process as the supervisor for crash atomicity. Trade-off accepted: 4 new Rust crates (49+14 grit units) vs. adopting an upstream that doesn't compose with foundry kernels."

---

## B. BLOCKING fixuptasks (12) — surgical diffs

### BLOCKER-1 — Add SettingsRenderer::verify memoization + default-disabled mode
**Source:** F-VERIFY-MEMOIZATION-CACHE-DEBOUNCE-1 + F-VERIFY-DEFAULT-DISABLED-IN-PROD-1 (CONV-1: F8+F2+F1+F11)
**Diff target:** v5 §B.2.5 + §B.4 step 2.

**Add to v5 §B.2.5 SupervisorConfig:**
```rust
pub struct SupervisorConfig {
    // ... existing fields ...
    pub settings_renderer_mode: RendererMode,
    pub settings_verify_debounce_secs: u64,  // default 60s; 0 disables cache
}

pub enum RendererMode {
    Disabled,    // default — verify never invoked; existing tick_once behavior
    VerifyOnly,  // log drift but don't reconcile
    Reconcile,   // render-if-drift (was VerifyMode::AutoReconcile)
}
```

**Change v5 §B.4 step 2:** Snapshot consults per-(account_id, template_blake3) cache with TTL `settings_verify_debounce_secs`; cache miss = invoke verify; cache hit = skip. Add §C.31 acceptance: "verify snapshot ≤ 50ms p99 on no-change path under 100-account fixture."

---

### BLOCKER-2 — Bench harness multi-sample p95
**Source:** F-BENCH-HARNESS-MULTI-SAMPLE-P95-EXACT-1 (CONV-2: F8+F11)
**Diff target:** v4 §B.1 bench harness + §C.13a-d.

**Change `crates/oya-foundry-supervisor-app/benches/heartbeat.rs`:** Loop ≥200 iterations per metric; collect samples in `Vec<u64>`; `.sort()`; report `vec[(0.95 * len as f64) as usize]` as p95. Each metric emits one JSONL row carrying `samples_count`, `p50`, `p95`, `p99`, `max`. Acceptance rows C.13a-d updated to require `samples_count >= 200`.

---

### BLOCKER-3 — Audit-chain conforms to ADR-0003 + data_class annotations + complete capability paths + drift exclusion emission
**Source:** F-AUDIT-CHAIN-ADR-0003-CONFORMANCE-1 + F-DATA-CLASS-ANNOTATIONS-NEW-TYPES-1 + F-AUDIT-EMISSION-COMPLETE-CAPABILITY-PATHS-1 + F-DRIFT-EXCLUSION-AUDIT-EMISSION-1 (CONV-3: F9+F11+F7+F5)
**Diff target:** v4 §B.2.1 type docs + §B.4 audit emission sites + v5 §B.4.

**Change v4 §B.2.1** — add data_class annotations:
```rust
/// data_class: INTERNAL_ONLY
pub struct SessionTicket { ... }

/// data_class: INTERNAL_ONLY (state machine; no tenant payload)
pub enum InboxState { ... }

/// data_class: TENANT_SCOPED (bridges to tenant_id via spend_to_usage_record)
pub struct SpendRecord { ... }

/// data_class: INTERNAL_ONLY
pub struct UsageWindowSnapshot { ... }

/// data_class: INTERNAL_ONLY
pub struct MessageId(pub String);

/// data_class: INTERNAL_ONLY (request idempotency key; opaque to tenant)
pub struct RequestId(pub String);
```

**Change v4 §B.4** — replace every `emit_audit_event(name, &ticket)` call with `evidence::emit_audit_row(AuditEvent { event_id: Ulid::new(), tenant_shard: ticket.account_id.tenant_shard(), prior_block_hash: chain.tail_hash(), event_class: "<class>", principal: ticket.account_id.clone(), capability: <cap_id>, data_classes_touched: vec![<...>], regulatory_packs_consumed: vec![], autonomy_tier_at_decision: ticket.autonomy_tier, payload: <typed-event-body> })` per ADR-0003.

**Add audit emission at every missing site:**
- Step 8/9 `OverUsageLimit`/`ReserveBreached` → emit `foundry_supervisor_degrade_account`
- Step 11 `TierBlocked` → emit `foundry_supervisor_tier_blocked`
- Step 12 `Quarantine` → emit `foundry_supervisor_quarantine`
- Step 9 `WindowExpired` → emit `foundry_supervisor_rotate_window`
- v5 every `SettingsRenderer::render` invocation → emit `foundry_supervisor_settings_render` with rendered_file_hashes
- v5 every drift exclusion under FailOnDrift → emit `foundry_supervisor_settings_drift_exclude` (NOT only side-channel JSON)

**Update acceptance:** C.8 expands to enumerate all 6 capability_id paths; new C.32 verifies one audit row per drift exclusion under the FailOnDrift fixture.

---

### BLOCKER-4 — Observability: structured logs + OTel spans + ADR-0042 gen_ai semconv + TelemetryMiddleware
**Source:** F-OBSERVABILITY-TICK-ONCE-STRUCTURED-LOGS-1 + F-OTEL-SPANS-GEN-AI-SEMCONV-1 + F-TELEMETRY-MIDDLEWARE-MOUNT-IN-MIDDLEWARECHAIN-1 (CONV-4: F11+M2)
**Diff target:** v4 §B.4 + §B.7.

**Add to v4 §B.4 tick_once:** one structured log event per outcome:
```rust
tracing::info!(event = "foundry.supervisor.tick_outcome", outcome = ?outcome, account_id = %ticket.account_id, message_id = %ticket.message_id);
```
Add tracing fields for `Spawned`, `Saturated`, `Idle`, `Quarantined`. Saturation gets `level = "warn"`. Quarantined gets `level = "error"`.

**Add spans** per ADR-0042 around tick_once + driver.spawn_for_message:
```rust
let span = tracing::info_span!("foundry.supervisor.tick",
    gen_ai.system = %ticket.provider_family.as_str(),
    gen_ai.request.model = %ticket.model_hint,
    oya.foundry.capability = "foundry.supervisor.tick_once",
    oya.tenant.id = %ticket.account_id.tenant_shard()
);
// inside span: record gen_ai.usage.input_tokens + output_tokens after SpendRecord
```

**Change v4 §B.7 webhook MiddlewareChain** — explicit insertion of `oya_http_middleware_telemetry::TelemetryMiddleware::new()` BEFORE the supervisor route handlers; documented per-route metrics: `oya_foundry_supervisor_route_requests_total{route,method,status_class}`, `oya_foundry_supervisor_route_latency_seconds_p99{route,method}`.

**New dashboard-kernel projection rows** (declared in v4 §B.9 cross-cut to P02):
```
oya_foundry_supervisor_inbox_depth{account_id}  (gauge, cardinality_ceiling 1000)
oya_foundry_supervisor_outbox_tail{account_id}  (gauge, cardinality_ceiling 1000)
oya_foundry_supervisor_idle_ticks_total          (counter)
oya_foundry_supervisor_quarantine_total          (counter)
oya_foundry_supervisor_session_active            (gauge by provider_family)
oya_foundry_supervisor_settings_drift_excluded_total{provider_family}  (counter)
```

Cardinality policy: `provider_family` is the metric label (4-variant enum); `account_id` is span attribute only, never metric label.

---

### BLOCKER-5 — Eliminate `.unwrap()` in tick_once + relocate UsageWindowSnapshot enforcement
**Source:** F-UNWRAP-IN-TICK-ONCE-REPLACE-OK-OR-1 + F-USAGE-WINDOW-SNAPSHOT-RELOCATE-ENFORCEMENT-1 (CONV-5: F5+F3)
**Diff target:** v4 §B.4 step 5 + §B.2.1 UsageWindowSnapshot doc + §B.4 step 13.5.

**Change v4 §B.4 step 5:**
```rust
// BEFORE:
let acc = accounts.iter().find(|a| a.id == exp.chosen_account_id).unwrap();
// AFTER:
let acc = accounts.iter().find(|a| a.id == exp.chosen_account_id)
    .ok_or(SupervisorError::NoEligibleAccount {
        chosen: exp.chosen_account_id.clone(),
        snapshot_ids: accounts.iter().map(|a| a.id.clone()).collect(),
    })?;
```
Add unit test `tests/lifecycle.rs::stale_account_id_race_returns_no_eligible_not_panic` covering RoutePolicy::select returning an account_id absent from a concurrent snapshot.

**Change v4 §B.2.1 UsageWindowSnapshot doc** + extract enforcement projection:
```rust
/// data_class: INTERNAL_ONLY
/// AUDIT-ONLY snapshot for ticket transport across blocking-pool boundaries.
/// MUST NOT be read for enforcement; live `UsageWindow` (account-domain:252)
/// remains the enforcement+reconciliation source of truth.
pub struct UsageWindowSnapshot { /* unchanged fields */ }

/// data_class: INTERNAL_ONLY
/// Enforcement-only projection computed from live UsageWindow at tick_once step 7.5.
/// Used by step 13.5 cost-ceiling gate.
pub struct EnforcementProjection {
    pub projected_tokens_p95: u64,
    pub window_id: WindowId,
    pub computed_at_epoch_secs: u64,
}
```

**Change v4 §B.4 step 13.5** — read from `EnforcementProjection` not from `UsageWindowSnapshot`. Apply F-PROJECTED-P95-COLDSTART-FAIL-CLOSED-1 (MED): seed n<10 case with `cost_ceiling + 1` so the comparator fails-closed, OR add explicit `ColdStartPolicy::{Block, Allow}` selected via config.

---

### BLOCKER-6 — Settings-template renderer reversibility: feature flag + backup + TickOutcome::DriftExcluded + symlink defense
**Source:** F-SETTINGS-RENDERER-MODE-FEATURE-FLAG-1 + F-SETTINGS-RENDER-BACKUP-BEFORE-WRITE-1 + F-TICK-OUTCOME-DRIFT-EXCLUDED-VARIANT-1 + F-RENDERER-SYMLINK-DEFENSE-O-NOFOLLOW-1 (CONV-6: F10+F4+F7+F5)

**Already partially covered by BLOCKER-1 (RendererMode).** Additional diffs:

**Change v4 §B.2.5 TickOutcome:**
```rust
pub enum TickOutcome {
    Spawned(MessageId),
    Saturated,
    Idle,
    Quarantined(MessageId),
    DriftExcluded { excluded_accounts: Vec<AccountId>, eligible_count: usize },  // NEW
}
```
Add `SupervisorConfig.minimum_eligible_accounts: usize` (default 1); if `eligible_count < minimum`, return `DriftExcluded` and emit warning log.

**Change v5 §B.2.2 SettingsRenderer::render** — atomic-tempfile sequence:
```
1. open parent dir with O_NOFOLLOW; stat: assert dir + owner==current_uid + mode <= 0755; reject if symlink
2. open target with O_NOFOLLOW|O_CLOEXEC; reject if existing entry is symlink
3. read-merge existing content via that fd (NOT a fresh path-based open)
4. if existing target exists, cp to `{target}.omc-settings-bak.{epoch}`
5. write tempfile via fchmod 0600 BEFORE rename
6. rename(2)
```

**New file:** `tools/undo-settings-render.sh` — `mv` from `*.omc-settings-bak.*` back to target; logs to audit-chain.

**New acceptance:** C.33 — renderer refuses to write when target_path OR any parent dir is a symlink. C.34 — rendered file mode == 0o100600. C.35 — backup exists after every render; `undo-settings-render.sh` restores prior state.

---

### BLOCKER-7..12 — Failing fixtures for new CC-1 kernel public API + new CI lane
**Source:** F-FAILING-FIXTURES-BATCH-1..4 (CONV-7: F3+F5+F7+M2)
**Diff target:** v4 §B.12 + v5 §B.7 grit units + new `tests/fixtures/` files.

**Add to v4 §B.12 / v5 §B.7 as new grit claim units:**

| Unit | Test | Failing fixture |
|---|---|---|
| v4.50 | `tests/lifecycle.rs::dead_letter_on_unlocked_returns_invalid_transition` | unlocked MessageId |
| v4.51 | `tests/lifecycle.rs::peek_lock_ttl_expiry_then_commit_walks_race` | TTL=1s, sleep 2s, assert second peek_lock succeeds, first commit fails |
| v4.52 | `tests/lifecycle.rs::silent_switch_caught_when_account_degrades_between_snapshot_and_spawn` | snapshot [A,B] → A degrades → step 4 routes to B → assert check_silent_switch fires |
| v4.53 | `tests/lifecycle.rs::cost_ceiling_at_boundary` | parametric `(projected,ceiling)` in {(99,100),(100,100),(101,100),(1,1),(0,0)} |
| v4.54 | `tests/lifecycle.rs::projected_p95_warm_window_underestimate_does_not_bypass_ceiling` | tail-burst sample distribution |
| v4.55 | `tests/lifecycle.rs::watchdog_kill_returns_fds_to_baseline` | FD count delta == 0 |
| v4.56 | `tests/lifecycle.rs::hung_cli_emits_exactly_one_spend_record_after_kill` | SpendRecord uniqueness |
| v4.57 | `tests/crash_injection.rs::test_sigkill_during_concurrent_producer_rename` | 2 writers SIGKILL'd at different points |
| v5.22 | `tests/hook_event_mapping.rs::unknown_event_yields_hook_event_not_mapped` | `tests/fixtures/codex-hooks.invalid.json` |
| v5.23 | `tests/secret_ref_resolution.rs::unresolved_sref_returns_secret_ref_unresolved_error` | `registry/accounts/codex.unresolved-sref.example.toml` |
| v5.24 | `tests/parse_template_toml.rs::adversarial_inputs_yield_invalid_template` | `tests/fixtures/templates/{duplicate-keys,unterminated-string,mixed-indent}.toml` |
| v5.25 | `tests/drift_detection.rs::missing_file_reports_drift_state_missing` | delete rendered file |
| v5.26 | `tests/drift_detection.rs::stray_file_in_render_root_reports_drift_state_extra` | add unmanaged file |
| v5.27 | `tests/render_atomicity.rs::sigkill_between_codex_config_toml_and_hooks_json_renames` | dual-file atomicity |
| v5.28 | `tests/concurrent_reconcile.rs::two_auto_reconcile_snapshots_one_render_each_account` | concurrency idempotence |

Total: **15 new grit claim units + 15 failing-fixture files.** Acceptance: C.36 — every new fixture file triggers the documented diagnostic; meta-test `tests/fixture_pair.rs` asserts pass+fail pairs.

---

## C. Non-blocking follow-up FixupTasks (46)

The remaining HIGH/MED/LOW FixupTasks (22 HIGH + 18 MED + 6 LOW = 46) ship as bounded follow-up work:

- **HIGH (8 not in BLOCKER set):** F-SHARED-TOML-MINI, F-SHARED-JSON-MINI, F-PARSER-INPUT-CAPS-DOS-DEFENSE, F-BRANCH-Y-COST-BENEFIT-QUANTIFY-ADR, F-V5-SPLIT-RENDER-VERSUS-VERIFY, F-STOP-NOTIFY-PUSH-MECHANISM-EVALUATE-ADR, F-SESSIONTICKET-NO-ARC-REAL-ENFORCEMENT, F-INBOX-OUTBOX-SHARDING, F-WINDOW-FOR-STORAGE, F-HOOKREF-COMMAND-PATH-VALIDATOR, F-ACCOUNT-ID-PATH-SAFE-NEWTYPE
- **MEDIUM (18):** see synthesis fixuptask_summary
- **LOW (6):** see synthesis fixuptask_summary

All filed at `registries/cross-cutting/fixuptasks.jsonl` with `severity`, `facet_origin`, `conv`, `section_anchor`, and where applicable `sunset_at` / `trigger_to_arm` fields.

## D. v2.3.0 A-family deferral

`F-MULTISPECTRUM-A-FAMILY-BACKFILL-M02-P06` is filed with `sunset_at = 2026-07-15` (the same date the `consensus-debate-evidence` lane arms). 8-week runway. A1..A7 to be dispatched in a follow-up session before sunset.

## E. Resume plan

1. **Apply A.PRE-1..PRE-6 + B.BLOCKER-1..12** as the v4+v5 amendment patch list. Each maps to specific §-anchor + before/after text above.
2. **Open M02-P06 phase INDEX.md** at `.omc/plans/milestones/M02-foundry-preview/phases/P06-foundry-supervisor/INDEX.md` (declared in v4 §B.10; Wave 1 creates it).
3. **Resume team `foundry-supervisor`** on the 14 implementation tasks — they remain valid; the BLOCKER edits land via Wave 2b (kernel types) + Wave 2c (jsonl adapter) + Wave 2d (supervisor-app) + Wave 4b (Cedar + registry/accounts) which are the natural homes for each diff.
4. **Add new grit claim units 4.50–4.57 + 5.22–5.28 + 5.19–5.21** (registry registrations) to the implementation task descriptions before workers begin.
5. **Schedule** `F-MULTISPECTRUM-A-FAMILY-BACKFILL-M02-P06` for next session.

## F. Status

**Pending approval.** When approved:
- 14 existing implementation tasks remain pending
- 15 new failing-fixture tasks add to Wave 2b/2c/2d work scope
- 3 registry-registration units (PRE-2/3/4) add to Wave 4b
- 1 rename (PRE-1), 1 ADR addendum (PRE-6), 1 N-threshold doc (PRE-5) add to Wave 1 (scaffold)

Then team resumes implementation. ETA to first grit claim: ~5 min after approval (Wave 1 scaffolder begins).
