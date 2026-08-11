---
doc_status: published
id: ADR-0715
title: "OS-layer retirement: node supervisor as OS; apex noun amend proposal; os/ harvest-then-retire"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-08-10
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0701]
amended_by: []
depends_on: []
related: [ADR-0520, ADR-0701, ADR-0712]
milestone: F1
deliverables:
  - id: ADR-0715-D1
    description: "Propose apex noun amend (text only; not applied while Proposed): k8s (projected) → node supervisor (owned PID1+runtime) → guest kernel, with upgrade actor and break-glass API as named siblings — not an OS product noun and not a Talos-shaped OS layer."
    exit_criteria: "Founder Accept authorizes a follow-on ADR-0701 amend PR to replace substrate-stack noun prose; Reject leaves live ADR-0701 noun unchanged. This ADR MUST NOT be read as having already amended the apex."
    verified_by: "oya-ci-required"
  - id: ADR-0715-D2
    description: "os/ disposition = harvest-then-retire. Harvest network/install/disk/time domains + init-app PID1 primitives into the supervisor substrate; delete dual-truth Talos simulation half (COSI, apid, trustd, config-v1alpha1, controller runtime) per-domain after harvest receipts."
    exit_criteria: "Accept records per-domain disposition obligation; encode PRs may not mass-delete os/ before harvest receipts and the D-3 preconditions."
    verified_by: "oya-ci-required"
  - id: ADR-0715-D3
    description: "Preconditions listed, not assumed done: (1) fleet-basis replacement for upstream-pin.json's dependence on infra/talos/installation-media/presets.yaml; (2) boot-marker contract replacement for BOOT_READY_MARKERS that currently include NixOS/systemd login strings invalidated by the userspace ban; (3) os/ meta-directory charter amendment away from Talos-class charter."
    exit_criteria: "Accept requires these three replacements to be designed and landed (or explicitly waived with dated founder exception) BEFORE apex noun amend encode and before os/ retire deletes."
    verified_by: "oya-ci-required"
---
# ADR-0715: OS-layer retirement — node supervisor as OS; apex noun amend proposal

## Status

**Proposed.** Deliberately not Accepted: clause D-1's apex noun amend and clause D-2's `os/`
retire are blocked on **D-3 preconditions** that are listed here and are **not** silently
assumed done. While Proposed this ADR carries **no implement authority** and does **not** amend
live ADR-0701 as fact.

Discovery input (not law): Round-2 synthesis in the local planning artifact
the Round-2 node forever-shape Discovery plan (local artifact id e6ec1a68) (founder F1(e); `apex-overrule-containerd` apex-noun
proposal surface — proposal text only).

## Context

ADR-0701 carries historical substrate-stack prose that still names a Talos-style OS layer and a
container-manager slot in the owned stack (provenance includes ADR-0520-era wording). Round-2
Discovery confirms the founder hypothesis that the **node supervisor is the OS**: PID1 duties,
machine-config ingestion, bootstrap networking, node identity/credential bootstrap, and host
telemetry producers fold into the supervisor; a separate Talos-shaped OS product noun dies.

Residuals that remain **siblings**, not an OS layer:

- **Upgrade actor** (A/B install + rollback) — cannot be owned by the thing being replaced.
- **Out-of-band break-glass node API**.
- NTP / disk-crypto **clients**.

`os/` today holds a large hand-written in-memory Talos simulation surface (init-app modeling
Talos `cmd/init` with in-memory fakes; runtime-cri-domain modeling a containerd client). It is
not an upstream-pinned PORT corpus with scope registry + ledger discipline, and it cannot boot
as the forever product. Harvest-then-retire is the proposed disposition.

**Naming ban (founder binding for this proposal text):** do not introduce public product nouns
from comparative prior art stacks. The proposed apex chain uses only Round-2 neutral nouns
below.

## Decision (proposed)

### D-1 — Apex noun amend proposal (text only)

On Accept, a **follow-on** amend to ADR-0701 (separate PR; not this file flipping Accepted law)
replaces the owned substrate stack noun with:

```text
k8s (projected) → node supervisor (owned PID1 + runtime) → guest kernel
```

Named siblings (not an OS noun):

- upgrade actor
- break-glass node API
- NTP / disk-crypto clients

This is an **apex-amend proposal**. Until Accept **and** the follow-on amend lands, live
ADR-0701 prose remains authoritative. Agents MUST NOT treat the noun above as already-amended
fact.

Comparative prior art that kept a distinct Talos-shaped OS product between k8s and the guest
kernel is **not adopted**.

### D-2 — `os/` harvest-then-retire

On Accept:

1. **Harvest** into the supervisor substrate: network / install / disk / time domains and
   `init-app` PID1 primitives (mount, reaper, switch_root, and related checked boot receipts).
2. **Retire / delete** the dual-truth half after harvest receipts: COSI, apid, trustd,
   `config-v1alpha1`, and the in-memory controller runtime that simulates a Talos control API
   without upstream pin discipline.
3. Disposition is **per-domain** across the existing domain set (~40) — not a single
   undeclared `rm -rf os/`.
4. A Talos *PORT* as second corpus is the wrong vehicle for the half this fold eliminates;
   W0-D second-corpus law stays with the ADR-0637/0704 designation, not a revival of the
   deleted simulation.

### D-3 — Preconditions (listed; not assumed done)

Before apex noun encode and before retire deletes, ALL of the following MUST land (or carry a
dated founder waiver):

| Precondition | Why |
|---|---|
| **Fleet-basis replacement** for `specs/k8s-port/upstream-pin.json` deriving k8s minor from `infra/talos/installation-media/presets.yaml` | That pin path dies with the Talos-shaped OS noun |
| **Boot-marker contract replacement** for `BOOT_READY_MARKERS` (today includes strings such as NixOS welcome / systemd PID1 markers) | Invalidated by the owned-userspace / supervisor-as-OS direction |
| **`os/` meta-directory charter amendment** | Chartered today as Talos-class; charter must match harvest-then-retire |

Silent assumption that these are already done is a **defect**.

## Consequences

- Positive: one owned node substrate; smaller TCB; honest apex noun; `os/` dual-truth removed
  after harvest.
- Negative: large migration; pin/boot-marker/charter blockers; upgrade/break-glass siblings need
  explicit owners.
- Operational: boot receipts become checked edges (mount → ABI stamp → config/net → runtime
  ready → kubelet register → CNI validate → taint remove); crash-loops escalate to NotReady.

## Rejected alternatives (proposed framing)

| Option | Why not |
|---|---|
| Keep Talos-shaped OS noun forever | Supervisor already owns PID1+runtime duties |
| Mass-delete `os/` before harvest | Loses network/install/disk/time + PID1 primitives |
| Treat this Proposed ADR as live apex amend | Violates gated-Proposed policy; ADR-0701 stays law until follow-on Accept encode |
| Rename the guest kernel product after comparative prior-art brands | Founder naming ban — use neutral `guest kernel` |

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept** | Authorizes follow-on ADR-0701 noun amend + `os/` harvest-then-retire program **after** D-3 preconditions; does not itself rewrite ADR-0701 body in this PR |
| **Reject** | Live ADR-0701 OS/substrate noun and `os/` charter unchanged |

## Citation contract

Proposed — **not implement authority**. Do not cite from authority surfaces as binding law while
`status: Proposed`. Do not claim the apex noun is already amended.
