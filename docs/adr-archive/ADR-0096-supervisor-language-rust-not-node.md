---
id: ADR-0096
title: "Supervisor language: Rust, not Node (build-vs-adopt Siigari/claude-heartbeat)"
status: Superseded
superseded_by: [ADR-709]
doc_status: published
owner: council-architecture
date: 2026-05-15
owner_phase: M02-P06
deciders:
  - Architect (v4 consensus)
  - Critic (v6 PRE-6 mandate)
supersedes: []
related:
  - ADR-0056  # 12-layer enum + port-in-kernel
  - ADR-0092  # workspace dependency seam policy (Branch Y)
  - ADR-0003  # audit-chain emission
  - ADR-0042  # OTel gen_ai semconv
---

# ADR-0096: Supervisor Language: Rust, not Node

## Status

Accepted (v6 PRE-6 mandate; source: F-CLAUDE-HEARTBEAT-BUILD-VS-ADOPT-ADR-1, TEN-2, `M1`).

## Context

The foundry supervisor (M02-P06) needs a session supervisor implementing hook +
inbox/outbox JSONL for multi-account, multi-provider Claude Code / Codex / Gemini
session management. The reference shape is **Siigari/claude-heartbeat** — a Node.js
implementation providing a supervisor daemon, stop-hook, JSONL inbox/outbox, per-message
subprocess restart, and idle heartbeat tick.

The build-vs-adopt question: should `oya-*` reuse the upstream Node implementation
(as a sidecar or fork) or build an equivalent Rust implementation inside the workspace?

## Decision

**Build in Rust.** The upstream Node implementation is rejected as a runtime dependency.

## Decision Drivers

1. **Workspace language purity** — All `oya-*` crates are Rust-native (ADR-0056).
   Introducing a Node runtime as a sidecar creates a second language dependency chain
   with separate toolchain, CVE surface, and CI requirements. The workspace has no
   precedent for non-Rust runtime artifacts in `crates/`.

2. **Kernel type composition** — The supervisor's call chain composes directly with:
   - `RoutePolicy::select` (oya-intelligence-route-policy-kernel)
   - `UsageEnforcement::check_limit` (oya-intelligence-usage-window-kernel)
   - `validate_usage` / `finalize_line` (oya-cloud-billing-kernel)
   - `check_silent_switch` / `ProviderAccount` state machine (oya-intelligence-account-domain)
   - `CeilingPolicy::enforce_for_tenant` (oya-governance-autonomy-ceiling-app)

   A Node sidecar cannot share these kernel types without an IPC bridge (JSON/CBOR
   serialization across process boundary). The IPC bridge would: (a) add latency on
   every tick, (b) require schema versioning for every kernel type change, and (c)
   become a coordination point that re-introduces the SPoF that Option B was rejected
   for (v4 §A.3).

3. **Crash atomicity for audit + Cedar enforcement** — Autonomy ceiling Cedar
   enforcement and audit-chain emission (ADR-0003) must execute in the **same process**
   as the supervisor decision logic for crash atomicity. An out-of-process Node sidecar
   that crashes between a Cedar `enforce_for_tenant` call and the corresponding
   `emit_audit_row` creates an undetectable inconsistency: spend recorded but audit
   event lost. Rust in-process guarantees either both succeed or the entire tick fails.

## Alternatives Considered

### Option N1 — Adopt Siigari/claude-heartbeat as Node sidecar

**Shape:** Rust supervisor delegates spawn decisions to a Node process via stdin/stdout
JSONL. Node process handles CLI process management.

**Pros:**
- Reuse upstream implementation immediately (hours vs. weeks to implement)
- Claude CLI stop-hook integration already proven in production (Siigari repo)
- Reduces initial scope

**Cons:**
- Node runtime not in workspace toolchain → second CI lane, second dependency audit
- IPC bridge required for every `SessionTicket` field crossing the process boundary
- Crash atomicity lost: audit-chain emission and Cedar enforcement cannot be atomic
  with spawn decision across process boundary
- Cannot share `ProviderFamily`, `AccountId`, `AutonomyTier`, `UsageWindowSnapshot`
  newtype wrappers without serialization overhead on every tick
- Dead-letter / peek-lock contract must be re-implemented or duplicated on Node side

**Verdict: REJECTED** — IPC bridge cost + atomicity loss + workspace language purity.

### Option N2 — Fork Siigari/claude-heartbeat into `tools/` as Node script

**Shape:** Copy the Node implementation into `tools/claude-heartbeat/` and call it
from a thin Rust wrapper.

**Pros:**
- Preserves upstream logic verbatim; no re-implementation risk

**Cons:**
- Same atomicity and type-sharing problems as N1
- Fork diverges from upstream immediately; maintenance burden with no upside
- `tools/` is for Rust binaries and shell scripts per workspace convention

**Verdict: REJECTED** — Same root problems as N1, plus fork maintenance cost.

### Option D (chosen) — Rust-native supervisor (4 new crates)

See v4 §A.3. Composes existing kernel ports without new IPC seams. 49 + 14 grit units
estimated across 4 crates. Language purity maintained. Kernel type composition is direct
(no serialization overhead). Audit atomicity guaranteed.

## Consequences

### Positive
- Supervisor shares `SessionTicket`, `ProviderFamily`, `AccountId`, `AutonomyTier`
  directly (zero-copy, no serialization)
- Cedar `enforce_for_tenant` + `emit_audit_row` in same process → crash atomic
- Single CI toolchain (Rust + cargo)
- Dead-letter / peek-lock contract lives in `oya-intelligence-jsonl-supervisor-adapter`
  with the same crash-injection test harness used by other adapter crates

### Negative / Trade-offs
- 4 new Rust crates (49 + 14 grit units) instead of adopting upstream (≈0 units)
- Claude CLI stop-hook integration must be re-verified against current Claude Code
  hook event vocabulary (open-question per v4 §A.4 + v5 C.27)
- Siigari/claude-heartbeat's battle-tested Node implementation cannot be directly
  reused; functional equivalence must be established via test matrix (v4 §A.5
  matrix_3x2x2.rs)

### Accepted non-durability
Per v4 §A.1 principle 4 (dep-branch-Y-commitment): best-effort durability; no
`fsync(parent_dir)`; non-durability-across-power-loss admitted in this ADR.

## References

- v4 §A.0.1 (addendum per PRE-6): build-vs-adopt analysis paragraph
- v6 amendments PRE-6: F-CLAUDE-HEARTBEAT-BUILD-VS-ADOPT-ADR-1 (TEN-2, `M1`)
- Siigari/claude-heartbeat: reference shape (v4 line 18)
- ADR-0056: 12-layer enum + port-in-kernel (workspace language governance)
- ADR-0092: workspace dependency seam policy (Branch Y — zero net-new external deps)
