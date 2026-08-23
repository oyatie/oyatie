---
doc_status: published
---
# Foundry Phase 00 Specification — Account-Auth Bootstrap

<!--
status: Accepted
date: 2026-05-12
related_adrs: ADR-0052, ADR-0053, ADR-0054, ADR-0055
-->

**Source:** Salvaged from `/Users/jasonlee/bominal/agents/ultragoal/` (foundry-agentic-substrate-master.md, 2026-05-12-foundry-ultragoal-mega-plan.md, brief.md, product-delivery-implementation-plan.md) — 2026-05-12.

**Landing rationale:** `docs/products/foundry/PRD.md` is product-level (75.5KB); Phase 00 contracts land separately so phases 01–06 specs can follow. See §H of foundry-salvage source.

**Source corpus cross-cites:** The canonical upstream planning corpus remains KEEP-classified in Bominal. This specification lands the Phase 00 contract surface from:
- `bominal/agents/ultragoal/2026-05-12-foundry-ultragoal-mega-plan.md`
- `bominal/agents/ultragoal/foundry-agentic-substrate-master.md`
- `bominal/agents/ultragoal/product-delivery-implementation-plan.md`
- `bominal/agents/ultragoal/brief.md` (supporting brief, not part of the P3.5 three-file PRD cite gate)

**Cross-link from main SPEC.md:**
> See [Foundry Phase 00 Specification](./PHASE-00-SPEC.md) for account-auth bootstrap contract, domain types, state machine, visibility surfaces, transport parity, and acceptance gates.

---

## A. Phase 00 spec — canonical contract surface

**Goal:** Add Codex, Claude, and Gemini account profiles so Foundry can authenticate the accounts it will utilize without hiding credentials or creating a second source of truth.

### Shape

**Domain:**
- `ProviderAccount`, `AccountProfile`, `AuthSession`, `UsageWindow`, `CapabilityGrant`, `PrivacyBoundary`, `SecretReference`, `ProviderAuthStatus`

**Application commands:**
- `RegisterProviderAccount`, `VerifyProviderAuth`, `ActivateProviderAccount`, `SetUsageBudget`, `RotateSecretReference`, `DisableProviderAccount`, `ImportRegularSession`

**Application queries:**
- `ListProviderAccounts`, `GetProviderAuthStatus`, `GetUsageWindows`, `ListActiveSessions`, `ExplainAccountRoute`

**Ports:**
- `ProviderAuthPort`, `SecretStorePort`, `UsageLedgerPort`, `AuditEventPort`, `ProviderSessionPort`

**Adapters:**
- Codex CLI, Claude Code, Gemini CLI, OS keychain/secret-manager, dashboard transport, GitHub Issue evidence adapters

### Security requirements
- No secrets in repo, fixtures, logs, or commits
- Account records store **redacted metadata and `SecretReference` only**
- Every route enforces provider, account/subscription, tenant, residency, data-class, privacy boundary, quota, usage window
- **No silent account switching** — every switch produces an audit event

### Regular session support
- External Codex, Claude, Gemini sessions remain supported
- **Foundry imports** their issue/context/evidence/account route on handoff
- Foundry remains the workflow source of truth

### Exclusive Foundry transition
- After Phase 00 + Phase 02 (self-hosting agent loop) are green, new project work starts through Foundry by default
- Direct sessions are **compatibility/import paths**, not competing workflow authorities

---

## B. Account-auth design — contracts and state machine

### Domain value types

#### `ProviderFamily` — allowlist-only
```
Allowlisted: AWS, OCI, Claude, OpenAIOrCodex, Gemini
Enforced: TryFrom<&str> rejects anything outside allowlist
Error: ProviderFamilyError::NotAllowlisted(String)
```

#### `ProviderAccount` — state machine
```
States: Draft → Verified → Active → Degraded → Disabled → Revoked (terminal)
Transitions:
  - Draft → Verified: on verify(SecretReference)
  - Verified → Active: on activate()
  - Active → Degraded: on degrade(reason)
  - Degraded → Active: on recover()
  - Any → Disabled: on disable(reason)
  - Any → Revoked: on revoke() (terminal)

Invariants:
  - Draft cannot activate directly; must verify first
  - Revoked is terminal; no transitions out
  - Silent account switch prevented: state machine validates no concurrent Active with same provider+subscription
```

#### `AuthSession`
```
Fields: account_id, provider_family, started_at, expires_at, capability_grant, privacy_boundary
Invariants: expires_at > started_at, privacy_boundary is non-empty
```

#### `UsageWindow`
```
Kind: FiveHour | OneWeek | Project
Fields: started_at, ends_at, tokens_in, tokens_out, cache_hits, estimated_cost_micros, usage_limit_pct, reserve_remaining_pct
Invariants:
  - started_at < ends_at
  - usage_limit_pct ∈ [0, 100]
```

#### `SecretReference`
```
Newtype: SecretReference(String)
Scheme: sref://
NEVER Display the inner value; only Debug shows scheme + redacted tail
Implement Redacted wrapper that always prints sref://…<hash>
```

#### Other required types
```
- AccountHealth: health status + reason + last_check_at
- Quota: limit (tokens/calls/cost), current, reserve_remaining_pct
- Subscription: tier, renewal_date, status
- Pricing: model, cost_per_mtok_in, cost_per_mtok_out, cache_cost_multiplier
- TokenCost: estimated_micros, actual_micros, model, tokens_in, tokens_out, cache_hit_pct
- RouteExplanation: chosen_provider, chosen_account, chosen_model, reason (quota/cost/privacy/model-match/failover)
- AuditEvidence: event_id, timestamp, account_id, action, old_state, new_state, actor, approved_by, evidence_bundle_path
```

### Application commands
```
RegisterProviderAccount(provider_family, account_metadata_redacted)
  → on_success: ProviderAccount(Draft)
  → audit event: "account_registered"

VerifyProviderAuth(account_id, secret_reference)
  → on_success: ProviderAccount(Verified)
  → audit event: "account_verified"

ActivateProviderAccount(account_id)
  → on_success: ProviderAccount(Active)
  → audit event: "account_activated"

SetUsageBudget(account_id, limit, reserve_pct)
  → updates Quota
  → audit event: "budget_updated"

RotateSecretReference(account_id, new_secret_reference)
  → invalidates old sref, issues new sref
  → audit event: "secret_rotated"

DisableProviderAccount(account_id, reason)
  → on_success: ProviderAccount(Disabled)
  → audit event: "account_disabled"

ImportRegularSession(session_metadata, account_route)
  → creates AuthSession linked to imported account
  → audit event: "session_imported"
```

---

## C. Visibility / read-only operator-plane surface

### Foundry-owned read-only capabilities
```
1. Issue management: list Milestones, Issues, sub-issues, dependencies, labels, priority, state
2. Agentic orchestration: list ready tasks, active sessions, session status
5. Token and usage management: list usage by account, window, project, model
6. Multi-account/subscription management: list accounts, quotas, budgets, routing policies
7. Model routing: list available routes and optimal effort suggestions
```

### Read-only surfaces required for Phase 00

**G004 — Read-only visibility surface:**
- Account health dashboard: account status, last verified, subscription status, quota, usage windows
- Session dashboard: active sessions, provider, account route, started_at, context, evidence path
- Usage dashboard: 5h window (tokens in/out, cache hits, cost estimate), 1wk window, project usage, usage_limit_pct
- Routing dashboard: account/model suggestions, route explanations, failover order, privacy constraints
- Dry-run surface: what-if analysis for route changes, budget changes, policy changes (no mutations)

**No write-capable ops yet** — Phase 05 gates write-capable ops behind approval.

---

## D. Targeted gates / validators / evidence templates

### Phase 00 acceptance gates

**P00-01: Clean Architecture skeleton**
```
Acceptance:
  - cargo check for 7 new crates: kernel/domain/app/adapter-codex-cli/adapter-claude-code/adapter-gemini-cli/adapter-openbao/runtime
  - Dependency boundary check: domain does not import app/adapter/runtime; app does not import adapter/runtime
  - All crates in workspace [members] glob

Validation command:
  bash presubmit (retired CLI gate validate) architecture-boundaries crates/intelligence-account-*
```

**P00-02: Domain types + application commands + ports**
```
Acceptance:
  - State transition tests: Draft→Verified→Active→Degraded→Disabled→Revoked (6 transitions, terminal Revoked)
  - Negative tests: cannot activate from Draft, cannot transition out of Revoked
  - Allowlist enforcement: ProviderFamily rejects non-allowlisted names
  - Silent account switch detection: Active account cannot be replaced without audit
  - 40+ unit tests covering all value types and state invariants

Validation command:
  cargo test -p intelligence-account-kernel -p intelligence-account-domain -p intelligence-account-app
```

**P00-03: Secret persistence — SecretStorePort adapter with local OpenBao default**
```
Acceptance:
  - ADR-0XXX: Local OpenBao as default SecretStorePort adapter (Phase 00)
    * OpenBao is one concrete implementation behind SecretStorePort
    * Blast radius, rollback, future OS-keychain/HSM adapters documented
    * No raw secrets in ADR itself
  - Integration tests (gated by OPENBAO_TEST_ADDR env)
  - Fake in-memory implementation for unit tests (no-credential fake)
  - Secret redaction tests: never log/commit raw credentials
  - Persistence roundtrip: store and retrieve SecretReference without exposing secret material

Validation command:
  cargo test -p intelligence-account-adapter-openbao --test integration_local
  cargo test -p intelligence-account-adapter-openbao --test fake_in_memory
  node scripts/hooks/guard-secrets.mjs --scan crates/intelligence-account-*
```

**P00-04: Provider adapters and provider-gateway parity**
```
Tests required:
  - Codex, Claude, Gemini capability-detection contract tests
  - Provider-gateway parity tests: each provider adapter exposes the same account route, capability, auth-status, and audit-event contract through the provider gateway
  - No provider adapter bypasses ProviderAuthPort, UsageLedgerPort, SecretStorePort, or AuditEventPort
```

**P00-05: Usage windows and quotas**
```
Tests required:
  - Usage window tests: 5h/1wk/project windows, usage_limit_pct enforcement, reserve_remaining_pct validation
  - Usage ledger evidence includes input, output, cache-hit, cost-estimate, and cooldown/failover counters
```

**P00-06: Account route policy and transport parity foundations**
```
Tests required:
  - Account route policy tests: budget ceilings, reserve budget, no silent account switch, privacy/residency constraints
  - Failover/cooldown tests: fallback order, rate-limit recovery, model degradation
```

**P00-07: Regular session import**
```
Acceptance:
  - Import test links: issue id, account route, context/evidence path, usage entry
  - Imported session records audit event: "session_imported"
  - Imported session inherits account quota, usage window constraints

Validation command:
  cargo test -p intelligence-account-app --test regular_session_import
```

**P00-08: Phase E2E and CI gate**
```
Acceptance:
  - Rust-owned `oya lint foundry-phase00-evidence` validation passes
  - Local fast: `cargo test --locked -p intelligence-account-*`
  - GitHub Actions Foundry lane: `cargo test --workspace`, secret scans, architecture boundary checks
  - Evidence bundle: account-auth slice delivered or exact gaps honestly stated
  - No stubs, placeholders, fake paths, TODO/TBD markers in acceptance paths

Validator: Rust-owned `oya lint foundry-phase00-evidence`
  Checks:
    - Files present: crates, tests, adapters
    - Executed local commands: cargo test results
    - Live/manual smoke requirements: OpenBao connectivity, if required
    - Credential-gated gaps documented: e.g., "browser OAuth handoff pending browser UI implementation"
    - SecretReference-only posture verified
    - Clean Architecture boundary evidence present
```

---

## E. Transport parity / write-gate foundations

### Transport Layer parity — P00-06

**Requirement:** REST, gRPC, WebSocket, SSE, Webhooks, and Kafka/MQ adapters call the same application/use-case ports or event ports.

**Phase 00 scope:** REST/SSE/WebSocket command input and status subscription foundations.

**Phase 05 scope:** gRPC, Webhook, Kafka/MQ write-capable ops.

### Provider-gateway parity — P00-04/P00-06

**Requirement:** Codex, Claude, Gemini, and future provider adapters must enter through one provider gateway contract, with identical account-auth, route explanation, usage-window, SecretReference, and audit-event semantics.

**Phase 00 scope:** provider gateway parity is validated for account registration, auth verification, activation, route explanation, status visibility, and account-route policy checks.

**Non-goal:** provider-specific features may exist behind adapters, but they cannot bypass provider-gateway parity or write a separate authority path.

### Smart usage/token management (Phase 00 baseline)

**Tracks per account per window:**
```
5h window: tokens_in, tokens_out, cache_hits, estimated_cost_micros, usage_limit_pct, reserve_remaining_pct
1wk window: same fields
project usage: cumulative across all windows, usage_limit_pct, model effort tracking
usage limit %: percent of budget consumed, triggers failover/cooldown if > threshold
reserve remaining %: safety margin to prevent stranding project if budget exhausted
input/output/cache tokens: raw counts per model
estimated cost: cost before actual billing
model effort: Haiku < Sonnet < Opus < GPT-5.5 medium < GPT-5.5 xhigh (effort ranking)
route explanation: human-readable reason for chosen account/model/provider
account/subscription health: verified/degraded/disabled state
cooldown/failover state: in effect until reset, blocks new work
```

**Policy requirements:**
```
- Maximize productive work inside current usage windows
- Preserve reserve_remaining_pct so agents do not strand project
- Prefer cheaper/lower-effort models for low-risk work
- Require cross-model validation for high-risk work
- Forbid silent account switching
- Allow fallback only through explicit route policy and audit
- Retain raw context/logs before minified summaries
```

### Write-gate foundations (Phase 05)

Phase 00 documents the gates; Phase 05 implements:
```
- SetUsageBudget: update account quota and reserve
- RotateSecretReference: issue new sref, invalidate old
- DisableProviderAccount: move account to Disabled state
- UpdateRoutePolicy: change failover order, cost thresholds
- TriggerFailover: manually move to fallback account (audit + approval required)
- CooldownReset: reset rate-limit cooldown (audit + approval required)
```

All write operations require:
- Audit event recording (immutable append-only log)
- Approval evidence (who approved, when, reason)
- Rollback capability: reverse the state change if approval revoked within time window
- Evidence bundle: link to issue, decision context, impact prediction

---

## F. Cross-references to non-foundry artifacts

1. **Object Graph primitives** — Foundry defines `ProviderAccount`, `AuthSession`, `UsageWindow` as domain entities; Object Graph adds `Provider`, `ProviderRegion`, `ProviderQuota` object types. Account-auth workflow is independent; Object Graph reads account state.

2. **Workflow primitives** — Account-auth is self-contained workflow. Foundry self-hosting loop (Phase 02) orchestrates workflows using Foundry account routes. No workflow mutations depend on account-auth.

3. **Data Use Boundary** — Foundry accounts enforce privacy boundary and data-class constraints. `ProviderPrivacyBoundary` and `UsageWindow` track residency. No cross-tenant account sharing.

4. **Autonomy ceiling** — Account activation/disable/failover require explicit policy or approval. Autonomy tier: agents may register/verify/monitor; humans approve activation/disable/failover. Phase 05 gates write-capable ops behind approval.

5. **Audit chain** — Every account-auth transition produces `AuditEvidence`. Immutable append-only ledger. Linked to GitHub Issue, human actor, approval evidence.

6. **Foundry dashboard/command center** — Account-auth provides read-only visibility for dashboard. Dashboard displays account health, usage windows, route explanations. Dashboard does NOT provide write-capable account operations in Phase 00.

7. **Codex/Claude/Gemini provider adapters** — Codex CLI adapter: register/verify/import sessions via CLI. Claude Code adapter: register/verify within Claude Code sessions. Gemini CLI adapter: register/verify via Gemini CLI. All adapters use same domain model and state machine.

---

## G. Items NOT salvaged (orchestration glue — deletion targets)

| File | Reason | Action |
|---|---|---|
| `ledger.jsonl` | Operational ledger of ultragoal checkpoint events; not spec content | Delete after archiving |
| `codex-goal-*.json` | Codex session state snapshots; not durable spec | Delete |
| `goals.json` | GitHub issue tracker state; superseded by actual GitHub Issues | Delete |
| `PAUSE.md` | Temporary pause notes during ultragoal execution | Delete |
| `goals.before-stale-*.json` | Stale backup; recovery artifact only | Delete |
| `final-readiness-*.json` | Validator output snapshot; not authoritative | Delete |
| `implementation-docs-*.json` | Validator output; re-generate in oyatie | Delete |
| `G004-reconciliation-blocker.md` | Session-specific blocker (bominal execution) | Delete |

---

## H. Consolidation path for future phases

```
oyatie/docs/products/foundry/
├── PHASE-00-SPEC.md  ← this file (account-auth bootstrap)
├── PHASE-01-SPEC.md  (kernel control plane — future)
├── PHASE-02-SPEC.md  (self-hosting agent loop — future)
├── PHASE-03-SPEC.md  (dashboard command center — future)
├── PHASE-04-SPEC.md  (ops visibility and dry-run — future)
├── PHASE-05-SPEC.md  (gated write-capable ops — future)
├── PHASE-06-SPEC.md  (delivery hardening — future)
└── README.md         (index, architecture overview, roadmap)
```

**Salvage total from bominal sources:** ~245KB of spec, architecture, and contract content.
**Source files read:** 20+ markdown and JSON files from `/Users/jasonlee/bominal/agents/ultragoal/`.
**Next step:** Create GitHub issues P00-01..P00-08 under Foundry Phase 00 milestone, begin implementation using Clean Architecture vertical slices with TDD and anti-slop gates.
