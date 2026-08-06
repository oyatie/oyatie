---
id: ADR-0099
title: "Cedar policy extension — foundry supervisor capabilities in docs/policies/foundry-supervisor.cedar"
status: Superseded
superseded_by: [ADR-700]
doc_status: published
owner: council-architecture
date: 2026-05-15
owner_phase: M02-P06
deciders:
  - Architect (ADR-0007 Cedar runtime enforcement mandate)
  - Critic (v6 Wave 4b policy design)
supersedes: []
related:
  - ADR-0007   # Cedar policy engine for RBAC/ABAC + persona-tier autonomy ceiling (T1–T4)
  - ADR-0022   # Autonomy ceiling — runtime enforcement via Cedar policy
  - ADR-0096   # Supervisor language: Rust (establishes the supervisor as Cedar-enforced surface)
  - ADR-0098   # Supervisor dep-policy Branch Y (crash atomicity context)
---

# ADR-0099: Cedar Policy Extension: Foundry Supervisor Capabilities at T1–T4

## Status

Accepted (ADR-0007 Cedar mandate; M02-P06 Wave 4b policy design; source: Task #12).

## Context

The foundry supervisor (M02-P06) exposes five distinct capabilities at runtime:

| Capability | Description |
|---|---|
| `foundry.supervisor.inject_message` | Write a message to a session's inbox JSONL |
| `foundry.supervisor.idle_tick` | Trigger an idle heartbeat tick for a session |
| `foundry.supervisor.restart_session` | Stop + re-spawn a session process |
| `foundry.supervisor.dead_letter` | Peek or drain the dead-letter queue |
| `foundry.supervisor.read` | Read inbox/outbox/dead-letter file contents |

ADR-0007 and ADR-0022 mandate that **every capability invocation is checked against a Cedar
policy at runtime** via `CeilingPolicy::enforce_for_tenant`. The existing seed file
`docs/policies/autonomy-ceiling.cedar` establishes the top-level T4-actuation forbid and
T1/T2 permit pattern (3 lines):

```cedar
// M02-P05-IP-002 autonomy ceiling — T4 actuation disabled by default
forbid (principal, action == Action::"actuate-t4", resource);
permit (principal, action in [Action::"read-t1", Action::"suggest-t2"], resource);
```

The supervisor capabilities are operational (not user-facing agent actions), but they still
require tier-gated authorization: an unauthenticated caller or a T1 principal must not be
able to `restart_session` or drain `dead_letter` without explicit elevation.

The Cedar policy for supervisor capabilities belongs in a **separate file**
(`docs/policies/foundry-supervisor.cedar`) rather than appended to the ceiling seed, for two
reasons: (1) the seed is the canonical T4-ceiling shape shared across all surfaces; (2) the
supervisor policy has a distinct action namespace (`foundry.supervisor.*`) that warrants its
own logical boundary.

**Wave 4b** (Task #12) creates the actual `docs/policies/foundry-supervisor.cedar` file.
This ADR records the policy design.

## Decision

Add `docs/policies/foundry-supervisor.cedar` containing tier-gated policies for the five
supervisor capabilities. The file is created by Wave 4b (Task #12); this ADR records the
design.

### Policy design

Autonomy tiers (from ADR-0007):

| Tier | Label | Principal class |
|---|---|---|
| T1 | `read-only-observer` | Monitoring/observability systems, read-only operators |
| T2 | `suggest-only` | Workflow automation, low-trust service accounts |
| T3 | `act-with-approval` | Internal services with human-in-the-loop escalation |
| T4 | `full-autonomy` | Supervisor daemon process itself, privileged SRE on-call |

### Capability-to-tier mapping

| Capability | Min tier | Rationale |
|---|---|---|
| `foundry.supervisor.read` | T1 | Non-mutating; safe for monitoring |
| `foundry.supervisor.inject_message` | T3 | Mutates inbox; requires approval-tier |
| `foundry.supervisor.idle_tick` | T3 | Triggers session activity; requires approval-tier |
| `foundry.supervisor.restart_session` | T4 | Destructive; supervisor daemon only |
| `foundry.supervisor.dead_letter` | T4 | Draining dead-letter is operationally sensitive |

### Cedar policy content (to be materialised by Wave 4b)

```cedar
// docs/policies/foundry-supervisor.cedar
// Foundry supervisor capability policies — M02-P06 Wave 4b
// Extends: docs/policies/autonomy-ceiling.cedar (global T4-actuation forbid applies)
// ADR: ADR-0099

namespace foundry::supervisor {

  // T1 — read-only observer: may read inbox/outbox/dead-letter (non-mutating)
  permit (
    principal in AutonomyTier::"T1",
    action == Action::"foundry.supervisor.read",
    resource
  );

  // T2 — suggest-only: read permitted (inherits T1); no mutation
  permit (
    principal in AutonomyTier::"T2",
    action == Action::"foundry.supervisor.read",
    resource
  );

  // T3 — act-with-approval: inject_message + idle_tick + read
  permit (
    principal in AutonomyTier::"T3",
    action in [
      Action::"foundry.supervisor.read",
      Action::"foundry.supervisor.inject_message",
      Action::"foundry.supervisor.idle_tick"
    ],
    resource
  );

  // T4 — full-autonomy: all supervisor capabilities
  permit (
    principal in AutonomyTier::"T4",
    action in [
      Action::"foundry.supervisor.read",
      Action::"foundry.supervisor.inject_message",
      Action::"foundry.supervisor.idle_tick",
      Action::"foundry.supervisor.restart_session",
      Action::"foundry.supervisor.dead_letter"
    ],
    resource
  );

  // Default deny: any action not explicitly permitted above is forbidden
  // (Cedar's default-deny semantics apply; this comment is documentation only)
}
```

### Interaction with the global ceiling seed

The global `autonomy-ceiling.cedar` forbid on `actuate-t4` applies to agent-facing autonomy
actions. Supervisor capabilities are operational (daemon-internal) rather than agent-facing,
so they use the `foundry.supervisor.*` namespace, which does not overlap with the
`actuate-t4` action in the ceiling seed. The Cedar runtime evaluates both files; the global
forbid does not suppress the T4 supervisor permits because the action namespaces are disjoint.

## Decision Drivers

1. **ADR-0007 + ADR-0022 mandate** — Every capability invocation must be Cedar-checked at
   runtime. Supervisor capabilities are no exception; they mutate session state and the
   dead-letter queue.

2. **Separate file, not appended to ceiling seed** — The ceiling seed is a 3-line global
   pattern. Appending supervisor-specific policy to it would conflate operational capability
   governance with the top-level autonomy ceiling contract. Separate file = separate logical
   boundary = separate review scope.

3. **T4-only for destructive operations** — `restart_session` terminates and re-spawns a
   CLI process. `dead_letter` drains the dead-letter queue. Both are destructive and
   irreversible without a backup. T4 restriction mirrors the global `actuate-t4` pattern
   from the ceiling seed.

4. **T3 for inbox mutation** — `inject_message` and `idle_tick` modify session state but
   do not terminate processes. T3 (act-with-approval) provides a middle tier: internal
   services can trigger ticks with human-in-the-loop escalation, without requiring the
   same elevation as process restart.

5. **T1 for read** — `foundry.supervisor.read` is observability-grade. Monitoring systems
   at T1 must be able to inspect inbox/outbox without elevation. This matches the global
   ceiling seed's `permit (…, action in [Action::"read-t1", …])` pattern.

## Alternatives Considered

### Alt A — Append to `docs/policies/autonomy-ceiling.cedar`

**Pros:** Single policy file; simpler Cedar evaluation graph.

**Cons:** Conflates global autonomy ceiling (agent-facing) with operational supervisor
capability governance. The ceiling seed is shared across all surfaces; adding supervisor
specifics makes it a mixed-concern file. Future surfaces (billing, audit-chain) would
pile policies into the same file.

**Verdict: REJECTED** — Separation of concerns; logical boundary per-surface.

### Alt B — Inline policy in Rust code (no .cedar file)

**Shape:** `CeilingPolicy::enforce_for_tenant` uses a hardcoded Rust match arm instead
of loading Cedar from disk.

**Pros:** No Cedar runtime dep for supervisor; simpler deployment.

**Cons:** Bypasses the Cedar runtime enforcement mandate (ADR-0007 + ADR-0022). Policy
cannot be updated without recompiling and redeploying. Breaks the audit-chain requirement:
Cedar policy changes must be versionable and auditable independently of binary releases.

**Verdict: REJECTED** — ADR-0007 mandate is categorical; Cedar file is required.

### Alt C — Single flat policy without namespace scoping (chosen shape simplified)

**Pros:** Fewer Cedar namespacing constructs.

**Cons:** Action names `inject_message`, `idle_tick` etc. collide with identically-named
actions in other surfaces if Cedar files are merged into a single namespace at evaluation
time.

**Verdict: REJECTED** — `foundry::supervisor` namespace prevents action-name collisions
as the Cedar runtime expands to cover more surfaces.

## Consequences

### Positive
- All five supervisor capabilities are Cedar-gated at runtime from day one.
- Policy is versionable and auditable independently of the Rust binary.
- T1 read access enables monitoring/OTel probes without privilege escalation.
- T4 restriction on `restart_session` + `dead_letter` matches operator on-call privilege model.

### Negative / Trade-offs
- Wave 4b (Task #12) must materialise the `.cedar` file; the ADR is accepted before the file
  exists. If Wave 4b is delayed, the supervisor ships without Cedar enforcement for these
  capabilities — which is a BLOCKER per ADR-0022.
- Cedar namespace syntax (`foundry::supervisor`) requires the Cedar SDK version in use to
  support namespace declarations. Verify against `oya-governance-autonomy-ceiling-app` Cedar
  SDK dep before Wave 4b authoring.

### Follow-up dependency

Wave 4b must be unblocked before the supervisor app binary can pass the autonomy-ceiling
enforcement gate. Task #12 tracks this.

## Follow-ups

1. **Wave 4b (Task #12)** — Create `docs/policies/foundry-supervisor.cedar` with the exact
   content specified in §Decision above.
2. **Cedar SDK version check** — Confirm namespace syntax compatibility with the Cedar SDK
   version pinned in `Cargo.toml` before Wave 4b authoring.
3. **Integration test** — `oya-governance-autonomy-ceiling-app` must include a test asserting
   that `inject_message` at T2 is denied and at T3 is permitted, using the supervisor
   Cedar policy file loaded from `docs/policies/`.
4. **audit-chain emission** — Each Cedar enforcement call for supervisor capabilities must
   emit an `audit_row` per ADR-0003 before the supervisor binary exits M02-P06 acceptance.

## References

- ADR-0007 — Cedar policy engine for RBAC/ABAC + persona-tier autonomy ceiling
- ADR-0022 — Autonomy ceiling runtime enforcement via Cedar
- ADR-0096 — Supervisor language: Rust (crash-atomicity + Cedar in-process requirement)
- ADR-0098 — Supervisor dep-policy Branch Y (blocking pool, sync I/O)
- ADR-0003 — Audit chain emission (Cedar enforcement must emit audit row)
- `docs/policies/autonomy-ceiling.cedar` — global ceiling seed (3 lines; T4 actuation forbid)
