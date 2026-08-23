---
doc_class: Standard
purpose: "Overview of the Foundry supervisor lane: architecture, components, and integration"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Foundry Supervisor — Lane Overview

**Wave:** M02-P06 (2b-5, spanning Waves 2-5)  
**Crates:** 5 (supervisor-kernel, supervisor-app, jsonl-supervisor-adapter, settings-template-kernel, settings-template-adapter)  
**Scope:** Multi-account, multi-provider session supervision with settings consistency  
**Status:** Under implementation (Wave 2d pending)

## What is the Supervisor?

The supervisor is a **daemon that orchestrates long-running sessions across multiple CLI accounts** (Claude, Codex, Gemini) with:

- **Message queuing** via file-backed inbox (JSONL)
- **Multi-account routing** via `RoutePolicy` + `UsageEnforcement`
- **Per-provider session management** (spawn, inject, drain, kill)
- **Atomic message acknowledgment** (commit/rollback/dead-letter)
- **Settings consistency** across accounts (verify + auto-reconcile)
- **Observability** (structured logs, OTel spans, audit chain per ADR-0003)
- **Crash safety** (atomic tempfile + rename, fsync'd durability)

## Why the Supervisor?

**User pain point (v5 §A.0):**

> "how are we patching the settings with the appropriate hooks / settings etc for each agent across accounts, so they are consistent and share the same skills, hooks, tools etc"

**Problem:** Without a supervisor, each account drifts independently:
- Hook installation varies per account
- Skills unavailable in some accounts
- MCP servers misconfigured per account
- Permissions rules diverge silently

**Solution:** Canonical `SettingsTemplate` + per-provider `SettingsRenderer` adapters ensure all accounts are in sync.

## Architecture

### 12-Layer Stack

```
L1: Kernel      ← supervisor-kernel (port traits, value types)
                ← settings-template-kernel (value types, SettingsRenderer trait)

L4: Adapter     ← jsonl-supervisor-adapter (InboxStore, OutboxSink)
                ← settings-template-adapter (Claude/Codex/Gemini renderers)
                ← account-adapter-{claude,codex,gemini} (SessionDriver impls)

L5: Application ← supervisor-app (daemon, tick_once call chain, hyper webhook)
```

### Data Flow

```
┌──────────────────────────────────────┐
│  Inbox (file-backed JSONL)           │
│  ~/.oya/inbox/msg-*.json             │
└────────────────┬─────────────────────┘
                 │ peek_lock
                 ↓
         ┌───────────────────┐
         │ tick_once()       │
         │ (17-step call)    │
         ├───────────────────┤
         │ 1. snapshot       │
         │ 2. peek_lock      │
         │ 3. route          │
         │ 4. enforce        │
         │ 5. spawn          │
         │ 6. inject         │
         │ 7. drain          │
         │ 8-14. process     │
         │ 15. commit        │
         │ 16. audit         │
         │ 17. return        │
         └────────┬──────────┘
                  │ append + fsync
                  ↓
         ┌──────────────────────┐
         │ Outbox (JSONL)       │
         │ ~/.oya/outbox/       │
         │  spend-records.jsonl │
         └──────────────────────┘
```

## Per-Crate Documentation

Each crate has 5 docs in `/docs/products/foundry/supervisor/<crate>/`:

### `supervisor-kernel/`
- **README** — value types, port traits, session lifecycle
- **ARCHITECTURE** — 12-layer placement, inward-only flow, port locations
- **OPERATIONS** — port trait implementation checklist, debugging
- **SECURITY** — OpenBao secret handling, Cedar enforcement, request-ID idempotency
- **BENCHMARKS** — performance budgets (session spawn, routing, tick_once)

### `supervisor-app/`
- **README** — daemon entry point, config, HTTP webhook, observability
- **ARCHITECTURE** — composition, async architecture, 17-step call chain, signals
- **OPERATIONS** — starting daemon, watchdog tuning, dead-letter triage, settings drift triage
- **SECURITY** — signal safety, audit conformance, idempotency cache
- **BENCHMARKS** — daemon latency budgets, heartbeat harness, profiling

### `jsonl-supervisor-adapter/`
- **README** — file-backed inbox/outbox, peek-lock, TTL, dead-letter, idempotency cache
- **ARCHITECTURE** — atomicity model, fsync placement, crash safety
- **OPERATIONS** — initialization, monitoring, cleanup, recovery
- **SECURITY** — file permissions, race conditions, audit trail
- **BENCHMARKS** — I/O latency, fsync cost, storage characteristics

### `settings-template-kernel/`
- **README** — canonical template type, SettingsRenderer trait, reusable building blocks
- **ARCHITECTURE** — kernel placement, value-only invariant, adapter composition
- **OPERATIONS** — template validation, drift detection workflow, maintenance
- **SECURITY** — sref:// secret references, data class annotations
- **BENCHMARKS** — template load/clone, drift detection latency

### `settings-template-adapter/`
- **README** — per-provider renderers, atomic write pattern (v6 BLOCKER-6)
- **ARCHITECTURE** — renderer architecture, format dialects, hook event mapping
- **OPERATIONS** — manual render/verify, reconciliation, backup management
- **SECURITY** — symlink defense (v6 BLOCKER-6), file permissions, secret safety
- **BENCHMARKS** — render latency, verify latency, idempotency, memoization

## Key Features

### 1. Atomic Message Processing

```
peek_lock → spawn → inject → drain → commit
↓
If crash between any steps → message returns to inbox or dead-letter
↓
No duplicate processing (idempotency cache + request_id)
```

### 2. Multi-Account Routing

```
AccountSnapshotProvider::snapshot()
  ├─ settings drift check (includes SettingsRenderer::verify per v5)
  └─ returns eligible accounts

RoutePolicy::select(eligible_accounts)
  └─ returns chosen account_id (round-robin, least-loaded, etc)

UsageEnforcement::check_limit(ticket, spend_record)
  └─ Cedar policy decision → allow or quarantine
```

### 3. Settings Consistency

```
SettingsTemplate (canonical)
  ├─ ClaudeRenderer → ~/.claude/settings.json
  ├─ CodexRenderer → ~/.codex/config.toml + hooks.json
  └─ GeminiRenderer → ~/.gemini/settings.json

At every snapshot:
  verify(template, account) → DriftReport
    ├─ Match → account eligible
    ├─ Modified/Missing/Extra → exclude (or auto-reconcile)
```

### 4. Observability

- **Structured logs** (JSON) — every tick_once outcome
- **OTel spans** (ADR-0042) — per-message tracing
- **Audit chain** (ADR-0003) — every state transition
- **Metrics** (Prometheus) — inbox depth, outbox tail, quarantine counter

## Integration Points

### Inbox Source

The inbox can be fed by:
1. **HTTP webhook** (`/inbox` endpoint) — supervisors running remotely inject messages
2. **File copy** — scripts copy JSONL files to `~/.oya/inbox/`
3. **Queue adapter** — Kafka, RabbitMQ, etc. via future adapters

### Outbox Consumer

The outbox can be drained by:
1. **Telemetry pipeline** — spend records → usage accounting system
2. **Audit system** — audit events → evidence store (ADR-0003)
3. **File export** — scripts archive spend-records.jsonl monthly

## Operational Readiness

### Acceptance Criteria (v4 §C)

- [ ] All 4 crates build cleanly
- [ ] Kernel types are value-only (no Arc/Box in fields)
- [ ] Adapters implement port traits correctly
- [ ] Dead-letter, peek_lock, commit operations are atomic
- [ ] Settings drift detection works
- [ ] Audit events emitted per ADR-0003
- [ ] Latency budgets met (p95 <= 250ms per tick)
- [ ] Benchmark harness produces multi-sample p95
- [ ] Symlink defense active (v6 BLOCKER-6)

### CI Lanes

- **lean-doc-coverage** — all 26 docs present + registered in DOC-CATALOG
- **lean-settings-drift** — drift detection passes on 3-provider fixture
- **governance-*** — naming, dependency, schema, algorithm checks
- **performance** — latency budgets enforced; no regressions

## References

- **v5 Delta:** `ralplan-foundry-supervisor-simple-v5-delta-settings-template-2026-05-15.md` (settings-template, +18 units)
- **v6 Amendments:** `ralplan-foundry-supervisor-simple-v6-amendments-2026-05-15.md` (BLOCKER edits, BLOCKERs 1-12)
- **Key ADRs:** ADR-0056 (12-layer), ADR-0003 (audit), ADR-0024 (autonomy), ADR-0042 (OTel)
- **Design:** `docs/DESIGN.md` § 10 (foundry supervisor axis contract)

## Getting Started

1. **Read this overview** (you are here)
2. **Understand the kernel** — `supervisor-kernel/README.md`
3. **Understand the adapters** — `jsonl-supervisor-adapter/README.md`, `settings-template-adapter/README.md`
4. **Run the daemon** — `supervisor-app/OPERATIONS.md` (Quick Start)
5. **Tune performance** — `supervisor-app/OPERATIONS.md` (Watchdog Tuning), `**/BENCHMARKS.md`

## Questions?

- **Architecture questions:** See ARCHITECTURE.md files
- **How to run:** See OPERATIONS.md files
- **Security concerns:** See SECURITY.md files
- **Performance tuning:** See BENCHMARKS.md files
