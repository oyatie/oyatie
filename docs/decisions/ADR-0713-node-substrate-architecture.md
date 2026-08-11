---
doc_status: published
id: ADR-0713
title: "Node Substrate Architecture: PID1 stub + restartable supervisor child; severable owned-runtime and os/-retirement Accept"
status: Proposed
planning_impact: true
deciders: founder
date: 2026-08-10
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0701, ADR-0704]
amended_by: []
depends_on: []
related: [ADR-0520, ADR-0637, ADR-0638, ADR-0704, ADR-0712]
milestone: F1
masterplan_work_item: MPV2-0054
deliverables:
  - id: ADR-0713-A
    description: "Severable Accept (a) — owned runtime shape: minimal PID1 reaper/launcher stub; projected kubelet + runtime-controller in restartable NON-PID1 supervisor child (one process); in-process CRI-shaped transport with CRI semantics canonical; external Unix socket behind versioned compatibility profile v1; no long-lived container-manager daemon; OWN token + Go containerd bootstrap CONSUME proposals; overrule containerd product PORT. State-machine/recovery DoD + kill-9-supervisor continuity and upgrade-reconnect tests required before Accept of (a)."
    exit_criteria: "Founder checks Accept (a) independently of (b). Accept (a) forbidden until state-machine/recovery DoD is recorded and required tests (kill-9 supervisor continuity + upgrade reconnect asserting kubelet-level reconvergence) are named as mandatory encode evidence. Reject (a) restores CONSUME-external-runtimes posture without owned-runtime encode."
    verified_by: "oya-ci-required"
  - id: ADR-0713-B
    description: "Severable Accept (b) — os/-layer retirement encode: apex noun amend proposal (k8s projected → node supervisor → guest kernel); os/ harvest-then-retire; D-3 preconditions (machine-config harvest receipt before config-v1alpha1 delete; fleet-basis pin replacement; boot-marker contract; os/ charter amendment)."
    exit_criteria: "Founder checks Accept (b) independently of (a). Accept (b) forbidden until D-3 preconditions land or carry dated founder waiver. Encode PRs may not mass-delete os/ before harvest receipts."
    verified_by: "oya-ci-required"
---
# ADR-0713: Node Substrate Architecture — PID1 stub + restartable supervisor; severable Accept

## Status

**Proposed.** Deliberately not Accepted. This ADR merges the former owned-runtime and
`os/`-retirement founder packages into one **Node Substrate Architecture** with **severable**
Accept checkboxes:

- **(a) Owned runtime shape** — waits on founder Accept/Reject **and** on the
  **state-machine / recovery Definition of Done** plus required continuity tests named below.
  Founder choice alone is insufficient.
- **(b) `os/`-layer retirement encode** — waits on **D-3 preconditions** (machine-config harvest,
  fleet-basis pin replacement, boot-marker contract, `os/` charter), listed and not assumed done.

The founder MAY Accept (a) while deferring (b), or Accept (b) while deferring (a), or Accept /
Reject each independently. Contradictory Accepted apexes from separate ADRs are avoided by
this merge (review thread 3 `depends_on` concern is **moot**).

While Proposed this ADR carries **no implement authority** and MUST NOT flip `scope.json`, the
divergence ledger, the closed capability registry, or live ADR-0701 apex nouns.

Live masterplan anchor (planning only, not implement authority):
[`/specs/masterplan.json#masterplan_v2.work_items[MPV2-0054]`](../../specs/masterplan.json)
(F1 Node Substrate Architecture package). Local Discovery artifact `e6ec1a68` is provenance
input only.

### Severable Accept checkboxes (founder)

- [ ] **(a) Accept owned runtime shape** (D-A1..D-A4) — after state-machine/recovery DoD + named tests
- [ ] **(b) Accept `os/`-layer retirement encode** (D-B1..D-B3) — after D-3 preconditions / waiver

## Context

Live `specs/k8s-port/scope.json` disposition vocabulary is exactly
`[PORT, CONSUME, EXCLUDE]`. Rule `SCP-CONSUME-EXTERNAL-RUNTIMES` (specificity_rank 95) currently
classes `containerd_or_cri_runtime` (among others) as CONSUME. An owned first-party node runtime
is **inexpressible** in that vocabulary today.

Round-4 architecture (founder-proxy approved; non-binding until Accept) selects:

- **PID1** = minimal reaper/launcher stub (subreaper, spawn/restart, boot marker, upgrade actor
  hook, break-glass hook) — not the full kubelet/runtime brain.
- **Projected kubelet + runtime-controller** live in a **restartable NON-PID1 supervisor child**
  sharing **one process** (in-process modules), with **CRI semantics canonical** on an
  in-process CRI-shaped transport.
- An **external Unix socket** exposes CRI behind a **versioned compatibility profile v1**
  (not a closed binary-name allowlist).
- Shims survive via **subreaper + durable-record adopt-or-kill reconciliation**.
- Supervisor upgrades are **restart + reconnect**; required tests assert kubelet-level
  reconvergence.

ADR-0701 still carries historical substrate-stack prose naming a Talos-style OS layer. Round-2/4
Discovery proposes the node supervisor *as* the OS (after harvest), with upgrade actor and
break-glass as siblings — severable as Accept (b).

Bominal inheritance: no Bominal equivalent — oyatie override for owned node substrate.

## Decision

### Accept (a) — Owned runtime shape

#### D-A1 — Process topology

On Accept (a), the forever node runtime topology is:

```text
PID1 — minimal reaper/launcher stub
  duties: subreaper; spawn/restart supervisor child; boot marker; upgrade actor; break-glass
  NOT: kubelet logic; NOT: full runtime controller

NON-PID1 supervisor child (restartable; one process)
  ├─ projected kubelet (in-process)
  ├─ runtime-controller (in-process)
  ├─ CAS image store (library)
  ├─ short-lived sandboxed pull workers
  ├─ per-sandbox shims (genuine process boundary; reparent via PID1 subreaper)
  │    └─ owned executor LIBRARY (OCI runtime-spec; youki/runc/crun = differential oracles only)
  ├─ in-process CRI-shaped transport (CRI semantics canonical; internal path NEVER sockets out)
  └─ external Unix socket — CRI compatibility profile v1 (see D-A2)
```

**BAN while/after Accept (a):** shipping a long-lived container-manager daemon as the product
forever shape. **BAN:** hand mini-CRI as forever semantics (bootstrap only if ever used —
Round-2 prefers dated Go containerd CONSUME instead).

#### D-A2 — CRI compatibility profile v1 (replaces binary-name allowlist)

CRI remains an **external compatibility face** only. Internal owned components MUST NOT route
through the external socket.

The external Unix socket is gated by a **versioned compatibility profile `v1`**, **NOT** a
closed binary-name allowlist. Profile `v1` MUST enumerate and contract-test:

| Profile `v1` element | Requirement |
|---|---|
| RPCs | Closed set of supported CRI RPC methods; unlisted = REFUSE |
| Streaming server | Supported streaming RPCs + backpressure / deadline rules |
| Evented PLEG | Event delivery contract compatible with projected kubelet expectations |
| Error-code semantics | Stable mapping for refused / unavailable / invalid |
| Peer credentials | SO_PEERCRED (or platform equivalent) authentication rules |
| Rate limits | Per-peer and global limits |
| Read-only RPC set | Explicit subset safe for read-only external consumers |

Unlisted profile elements and callers that fail peer-cred / rate-limit / RPC-set checks are
**REFUSED**. Implementers MUST NOT invent additional authorized consumers by binary name;
authorization is profile + peer cred, version-negotiated.

#### D-A3 — Shim survival, upgrades, and required tests

On Accept (a):

1. **Shim survival:** PID1 subreaper + durable on-disk sandbox records; restart path is
   **adopt-or-kill** reconciliation against those records (not "manager daemon remembers").
2. **Supervisor upgrades:** restart the NON-PID1 supervisor child + **reconnect**; PID1 stub
   stays up across the restart window as specified by the recovery state machine.
3. **Required tests (encode evidence; mandatory before claiming Accept (a) DoD met):**
   - **kill -9 supervisor continuity** — shims/workloads survive; durable-record reconciliation
     restores or kills correctly; **kubelet-level reconvergence** is asserted (not merely
     "process still running").
   - **Upgrade reconnect** — supervisor restart+reconnect completes with **kubelet-level
     reconvergence** asserted.

#### D-A4 — State-machine / recovery DoD (gate for Accept (a))

Accept (a) of "no manager daemon" is **forbidden** until a recorded **state-machine / recovery
Definition of Done** exists that names:

1. Boot marker → supervisor up → runtime ready → kubelet register → CNI validate → taint remove
   (checked edges).
2. Crash / kill-9 / upgrade restart transitions and adopt-or-kill outcomes.
3. Escalation to NotReady on unrecoverable loops.
4. Mapping of the two required tests in D-A3 to those transitions.

Without that DoD, Accept (a) would assert an untestable forever shape.

#### D-A5 — Scope OWN token + Go containerd bootstrap CONSUME (proposal text only)

This clause **authors the required follow-on edits**; it does **not** apply them.

On Accept (a), a subsequent scoped PR MUST:

1. Extend `disposition_vocabulary` in `specs/k8s-port/scope.json` with **`OWN`**.
2. Add a higher-rank rule for owned node runtime libraries disposition **OWN**.
3. Add a **bootstrap** rule: pinned **Go containerd CONSUME** with **dated expiry**, digest pin,
   and divergence-ledger row intent `DVG-OWNED-NODE-RUNTIME`.
4. Keep **CNI plugin binaries CONSUME**.
5. Decide **capability home**: register top-level `runtime/` **or** record destination ruling
   under an existing capability — blocked until Accept (a).

#### D-A6 — Overrule containerd product PORT

On Accept (a):

- **Forbidden:** CONSUME→PORT of containerd as product forever shape.
- **Forbidden:** creating `specs/containerd-port/` as a product sibling registry for that purpose.
- **Required:** D5 third-corpus neutrality proof uses **ttrpc and/or go-cni** (satellite), not
  containerd product PORT.
- Mechanical **k8s** PORT (including kubelet) remains under ADR-0704.

### Accept (b) — `os/`-layer retirement encode

#### D-B1 — Apex noun amend proposal (text only)

On Accept (b), a **follow-on** amend to ADR-0701 (separate PR) replaces the owned substrate
stack noun with:

```text
k8s (projected) → node supervisor (owned PID1 stub + restartable supervisor child) → guest kernel
```

Named siblings (not an OS noun): upgrade actor; break-glass node API; NTP / disk-crypto clients.

Until Accept (b) **and** the follow-on amend lands, live ADR-0701 prose remains authoritative.

#### D-B2 — `os/` harvest-then-retire

On Accept (b):

1. **Harvest** into the supervisor substrate: network / install / disk / time domains and
   `init-app` PID1 primitives (mount, reaper, switch_root, and related checked boot receipts),
   **and machine-config semantics** (see D-B3).
2. **Retire / delete** the dual-truth half after harvest receipts: COSI, apid, trustd,
   `config-v1alpha1`, and the in-memory controller runtime — **only after** the machine-config
   harvest receipt in D-B3.
3. Disposition is **per-domain** — not a single undeclared `rm -rf os/`.

#### D-B3 — Preconditions for Accept (b) / retire deletes (listed; not assumed done)

Before apex noun encode and before retire deletes, ALL of the following MUST land (or carry a
dated founder waiver):

| Precondition | Why |
|---|---|
| **Machine-config harvest receipt** — owned replacement contract for machine-config semantics currently modeled under `config-v1alpha1`, with a recorded harvest receipt **before** any delete of `config-v1alpha1` | Supervisor assumes machine-config ingestion; deleting the only modeled schema without a replacement leaves nodes unable to consume boot configuration (review thread 8) |
| **Fleet-basis replacement** for `specs/k8s-port/upstream-pin.json` deriving k8s minor from `infra/talos/installation-media/presets.yaml` | That pin path dies with the Talos-shaped OS noun |
| **Boot-marker contract replacement** for `BOOT_READY_MARKERS` (today includes NixOS/systemd login strings invalidated by the userspace ban) | Invalidated by supervisor-as-OS direction |
| **`os/` meta-directory charter amendment** | Chartered today as Talos-class; charter must match harvest-then-retire |

Silent assumption that these are already done is a **defect**.

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | Notes |
|---|---|---|
| PID1 stub + supervisor child crates (destination capability TBD) | create | Accept (a): full owned-runtime shape. Accept (b)-only: PID1 stub + harvest/retire surfaces required by D-B2 may land without Accept (a); full NON-PID1 kubelet/runtime-controller child remains Accept (a)-gated |
| CRI compatibility profile `v1` contract + tests | create | RPCs, streaming, PLEG, errors, peer cred, rate limits, read-only set — Accept (a) |
| kill-9 continuity + upgrade reconnect tests | create | Mandatory encode evidence for Accept (a) DoD |
| `specs/k8s-port/scope.json` | update | OWN token + bootstrap CONSUME — Accept (a) follow-on only |
| `os/` domains | harvest then delete | Accept (b) only; after D-B3 receipts |
| `docs/decisions/ADR-0701-*.md` | amend | Apex noun — Accept (b) follow-on only |

### Integration via Workflow + Ontology

Not applicable directly. Node boot/recovery emits operational readiness signals consumed by
cluster control plane after Accept encode; Object/Link types land in the owning capability PRD.

### Positive

- Severable Accept avoids contradictory apexes; founder can ship runtime shape without forcing
  `os/` retire, and vice versa.
- Smaller privileged surface; shim survival without manager daemon; honest CRI profile contract.
- Machine-config harvest is an explicit precondition, not an implied delete.

### Negative

- Large migration; bootstrap CONSUME window must be dated; OWN token is a schema change.
- Recovery DoD is on the critical path before Accept (a).

### Operational

- Boot receipts become checked edges; crash-loops escalate to NotReady.
- CI: Accept evidence rides `oya-ci-required`; no authority-surface citation while Proposed.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Affected after Accept (a) encode | New supervisor crates declare layers |
| `cross-product-refusal` (LEAN-A2) | Not affected | none |
| `port-location` | Affected after Accept (a) | CRI/runtime ports in kernel layer of owning capability |
| `layer-correctness` | Affected after Accept (a)/(b) | Capability home ruling |
| `composition-root-only` | Affected after Accept (a) | Node composition root hosts PID1+child wiring |
| `sdk-kernel-only` | Not affected | none |

Port traits (after Accept encode; illustrative — not implement authority now):

```rust
// Destination capability kernel ports (Accept (a) follow-on) — ZERO I/O in kernel
#[async_trait::async_trait]
pub trait SandboxRecordStore: Send + Sync {
    async fn load(&self, id: &SandboxId) -> Result<SandboxRecord, StoreError>;
    async fn adopt_or_kill(&self, id: &SandboxId) -> Result<ReconcileOutcome, StoreError>;
}
```

## Alternatives considered

**Alternative 1 — Mechanical containerd PORT as forever product**
- Pros: familiar CNCF shape.
- Cons: daemon fails existence test; worst neutrality corpus; false conformance premise.
- Reason rejected: D-A6.

**Alternative 2 — Keep separate ADR-0712 + ADR-0715 with empty depends_on**
- Pros: smaller diffs.
- Cons: contradictory Accept risk (thread 3).
- Reason rejected: merged severable ADR.

**Alternative 3 — Closed binary-name CRI allowlist**
- Pros: simple deny list.
- Cons: unenumerable ellipsis; cannot contract-test (thread 7).
- Reason rejected: compatibility profile `v1`.

**Alternative 4 — Mass-delete `os/` before machine-config harvest**
- Pros: faster retire.
- Cons: deletes boot config semantics with no replacement (thread 8).
- Reason rejected: D-B3.

**Alternative 5 — Accept (a) on founder choice alone without recovery DoD**
- Pros: faster pause-and-pair close.
- Cons: untestable "no manager daemon" claim.
- Reason rejected: D-A4.

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept (a) only** | Owned runtime shape + OWN/bootstrap proposals become amend authority; `os/` noun/retire unchanged |
| **Accept (b) only** | Apex noun amend + `os/` harvest-then-retire authorized after D-B3; no owned-runtime encode from (a) |
| **Accept (a)+(b)** | Full Node Substrate Architecture program |
| **Reject (a)** | Live `SCP-CONSUME-EXTERNAL-RUNTIMES` posture remains for runtime |
| **Reject (b)** | Live ADR-0701 OS/substrate noun and `os/` charter unchanged |
| **Reject ADR** | Neither package becomes encode authority |

## Citation contract

Proposed — **not implement authority**. Do not cite from authority surfaces as binding law while
`status: Proposed`. Do not claim the apex noun is already amended. Do not claim "no manager
daemon" is Accepted without checkbox (a) and the recovery DoD.

## References

- Live masterplan: `MPV2-0054` in `/specs/masterplan.json#masterplan_v2.work_items`
- ADR-0701 / ADR-0704 (live substrate); ADR-0637 / ADR-0638 (port-engine); ADR-0520 provenance
- ADR-0712 (pool matrix; related SKU physics)
- Round-2/4 Discovery local artifact `e6ec1a68` — provenance only
- PR #1929 Round-4 amend; merges former owned-runtime + os/-retirement draft topics; vacates the colliding draft number reserved for PR #1644
