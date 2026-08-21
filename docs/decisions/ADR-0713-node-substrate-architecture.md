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
    description: "Severable Accept (a) — owned runtime shape: minimal PID1 reaper/launcher stub; projected kubelet + runtime-controller in restartable NON-PID1 supervisor child (one process); in-process CRI-shaped transport with CRI semantics canonical; external Unix socket behind versioned compatibility profile v1 (bootstrap-minimal vs promotion-complete); no long-lived container-manager daemon; OWN token + Go containerd bootstrap CONSUME proposals; overrule containerd product PORT. Round-5 Node Substrate DoD package (stub respawn constant-work; exclusive supervisor lease + dual-supervisor race; telemetry-first + NodeReady flap SLO/OpenSLO-or-EV0-owner; zero-trust checklist; FinOps K-stage exits; CAS crash semantics; stub LOC ratchet) + Round-4 recovery tests required before Accept of (a). Process-law exits recorded in body: K1-owned gated on owned-executor security-response; flake/rerun policy before first promotion-gate claim."
    exit_criteria: "Founder checks Accept (a) independently of (b). Accept (a) forbidden until state-machine/recovery DoD (incl. Round-5 package D-A3..D-A4 / D-A7..D-A9) is recorded and required tests (kill-9 continuity + upgrade reconnect + dual-supervisor race + stub respawn budget escalate) asserting kubelet-level reconvergence are named as mandatory encode evidence. Reject (a) restores CONSUME-external-runtimes posture without owned-runtime encode."
    verified_by: "oya-ci-required"
  - id: ADR-0713-B
    description: "Severable Accept (b) — os/-layer retirement encode: apex noun amend proposal (k8s projected → node supervisor → guest kernel); os/ harvest-then-retire; D-3 preconditions (machine-config harvest receipt before config-v1alpha1 delete; fleet-basis pin replacement; boot-marker contract; os/ charter amendment)."
    exit_criteria: "Founder checks Accept (b) independently of (a) for harvest/charter work. Accept (b) forbidden until D-3/D-B3 preconditions land or carry dated founder waiver. Retire/delete of destination-assumed os/ halves forbidden until Accept (a) or dated waiver. Encode PRs may not mass-delete os/ before harvest receipts."
    verified_by: "oya-ci-required"
---
# ADR-0713: Node Substrate Architecture — PID1 stub + restartable supervisor; severable Accept

## Status

**Proposed.** Deliberately not Accepted. This ADR merges the former owned-runtime and
`os/`-retirement founder packages into one **Node Substrate Architecture** with **severable**
Accept checkboxes:

- **(a) Owned runtime shape** — waits on founder Accept/Reject **and** on the
  **state-machine / recovery Definition of Done** (Round-4 continuity + **Round-5** stub
  respawn / exclusive lease / telemetry / zero-trust / FinOps package) plus required tests
  named below. Founder choice alone is insufficient.
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

- [ ] **(a) Accept owned runtime shape** (D-A1..D-A9) — after state-machine/recovery DoD incl. Round-5 package + named tests
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

| Profile `v1` element | Requirement | Completeness tier |
|---|---|---|
| RPCs | Closed set of supported CRI RPC methods; unlisted = REFUSE | **bootstrap-minimal** |
| Error-code semantics | Stable mapping for refused / unavailable / invalid | **bootstrap-minimal** |
| Peer credentials | SO_PEERCRED (or platform equivalent) authentication rules | **bootstrap-minimal** |
| Rate limits | Per-peer and global limits | **bootstrap-minimal** |
| Streaming server | Supported streaming RPCs + backpressure / deadline rules | **promotion-complete** |
| Evented PLEG | Event delivery contract compatible with projected kubelet expectations | **promotion-complete** |
| Read-only RPC set | Explicit subset safe for read-only external consumers | **promotion-complete** |

**Round-5 (cheap):** every profile `v1` section MUST be marked **bootstrap-minimal** or
**promotion-complete**. **K1-reference** MUST NOT be blocked on a full streaming server —
bootstrap-minimal is the minimal RPC/error/peer-cred surface needed to schedule a pod;
promotion-complete adds streaming, evented PLEG, and the full read-only observability set
before any promotion/release claim.

Unlisted profile elements and callers that fail peer-cred / rate-limit / RPC-set checks are
**REFUSED**. Implementers MUST NOT invent additional authorized consumers by binary name;
authorization is profile + peer cred, version-negotiated.

#### D-A3 — Shim survival, upgrades, leases, and required tests

On Accept (a):

1. **Shim survival:** PID1 subreaper + durable on-disk sandbox records; restart path is
   **adopt-or-kill** reconciliation against those records (not "manager daemon remembers").
2. **Supervisor upgrades:** restart the NON-PID1 supervisor child + **reconnect**; PID1 stub
   stays up across the restart window as specified by the recovery state machine.
3. **Stub respawn constant-work law (Round-5 MAJOR):** PID1 stub **MUST rate-limit** supervisor
   restarts — **exponential backoff + jitter**, **max restarts / window**, and **escalate to
   break-glass / Node condition** when the restart budget is exceeded. Restart storms MUST NOT
   become unbounded PID1 work (constant-work / anti-fragility).
4. **Exclusive supervisor lease / anti-split-brain (Round-5 MAJOR):** single-writer lease via
   **pidfd and/or lockfile + generation**. A second supervisor **MUST refuse** or take over
   **only with a fenced generation**. This lease covers **supervisor ownership of the durable
   store**; adopt-or-kill remains the shim survival law.
5. **CAS / durable-schema crash semantics (Round-5 cheap):** durable writes have an explicit
   **commit point**; readers MUST tolerate **n−1** schema and detect/refuse a **torn write**.
   Crash between prepare and commit leaves the prior committed generation authoritative;
   recovery MUST NOT treat a partial record as live ownership without the lease generation
   check above.
6. **Required tests (encode evidence; mandatory before claiming Accept (a) DoD met):**
   - **kill -9 supervisor continuity** — shims/workloads survive; durable-record reconciliation
     restores or kills correctly; **kubelet-level reconvergence** is asserted (not merely
     "process still running"); NodeReady flap within a **falsifiable numeric budget** (or
     TBD-with-owner at EV0 — not vague "within budget").
   - **Upgrade reconnect** — supervisor restart+reconnect completes with **kubelet-level
     reconvergence** asserted under the same flap budget rule.
   - **Dual-supervisor race** — Done-when for the exclusive lease: two supervisors racing MUST
     yield exactly one writer (refuse or fenced takeover); durable-store ownership stays single.
   - **Stub respawn budget escalate** — exceeding max restarts/window fires the break-glass /
     Node condition path (not silent unbounded restart).
   - **Bounded recovery objective** — continuity/upgrade tests MUST bind a **measurable**
     maximum or percentile recovery budget (wall-clock to kubelet-level reconvergence) recorded
     in the recovery DoD; exceeding the budget fails the test. Exact numeric SLO is an Accept
     (a) encode parameter, not founder-silent unbounded "eventually".

#### D-A4 — State-machine / recovery DoD (gate for Accept (a))

Accept (a) of "no manager daemon" is **forbidden** until a recorded **state-machine / recovery
Definition of Done** exists that names:

1. Boot marker → supervisor up → runtime ready → kubelet register → CNI validate → taint remove
   (checked edges).
2. Crash / kill-9 / upgrade restart transitions and adopt-or-kill outcomes.
3. Stub respawn backoff/jitter/window/escalate transitions (D-A3 item 3).
4. Exclusive supervisor lease acquire / refuse / fenced-takeover transitions (D-A3 item 4).
5. Escalation to NotReady on unrecoverable loops.
6. Mapping of the required tests in D-A3 to those transitions.
7. **Stub LOC / privilege ratchet (Round-5 cheap):** harvest into the stub MUST NOT fatten
   PID1 beyond named residuals (~hundreds of lines). First ratchet fixture = canonical
   stub-split architecture (passes); fat PID1 / kubelet-in-PID1 fails the fixture.

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

#### D-A7 — Node telemetry-first DoD (Round-5 MAJOR)

**Day-1 before any K-stage "promotion past dev" claim** (Proposed-founder encode gate for Accept
(a) follow-on; not an Accept of this ADR by itself). Mandatory metric + trace set:

| Signal | Requirement |
|---|---|
| Supervisor restart count / reason | Counter + reason label (lease loss, OOM, upgrade, budget escalate, …) |
| Shim adopt / kill | Counters for adopt-or-kill outcomes |
| CRI REFUSE | Counter for out-of-profile / unauthorized peers |
| Pull QPS / bytes | Registry request rate + bytes fetched (ties to pull-storm differential) |
| Attestation UNKNOWN / TTL | Gauge/counter for stale/UNKNOWN and result TTL expiry |
| NodeReady flap SLO | **Falsifiable numeric budget** (or explicit TBD-with-owner at EV0) |

**OpenSLO** (or an **explicit EV0 deferral with named owner**) is required for the NodeReady flap
objective and any other day-1 SLO claimed as gateable. No K-stage "promotion past dev" claim
without this set published.

#### D-A8 — Zero-trust checklist beyond attestation (Round-5 MAJOR)

On Accept (a) encode, Node Substrate MUST clear this checklist (attestation alone is insufficient):

1. Break-glass **authn/authz + audit**
2. A/B supervisor **image signature verify**
3. Durable-record **tamper posture**
4. Shim↔supervisor channel **peer-cred**
5. Verifier **mTLS / identity**
6. Pull-worker **egress allowlist**

#### D-A9 — FinOps unit-cost exits + process-law K exits (Round-5)

**FinOps / unit-cost (Proposed-founder):** Done-when rows at **K1-reference / K2 / private-kernel**
exits MUST include (measure at EV0/K — **do not invent tip numbers**):

- Density floor **or** `$/pod-class` unit cost
- **Supervisor RSS** budget
- **Guest-pull extra registry-bytes** vs host-CAS (attested confidentiality path)

**Process-law exits (recorded here while this ADR stays Proposed; not silent Open items):**

1. **Owned-executor security-response = K1-owned exit precondition.** Embargo handling + patch
   SLA process MUST exist before promoting the forever path from **K1-reference** to
   **K1-owned**. Until it does, stay on K1-reference (youki/runc/crun remain differential
   oracles only — never shipped to green a gate).
2. **Conformance flake/rerun policy before first promotion-gate claim.** A minimal flake
   taxonomy + rerun budget MUST be published before the first **promotion/release** claim that
   runs full CNCF + Sonobuoy. PR-gate smoke subsets are unaffected; this is a promotion-gate
   precondition, not an Accept of this ADR.

**Harvested doctrine — the owned-executor law and its adversarial corpus (2026-08-19).**
This clause records, in the ADR that owns the decision, design law that until now existed only
inside `os/harness/oci-executor-oracle` — a self-declared scaffold slated for deletion under
Accept (b). It is recorded here BEFORE that deletion, so retiring the scaffold costs no doctrine.

- **The forever executor is an OWNED library** of the per-sandbox shim, built from the OCI
  runtime-spec. `youki`, `runc` and `crun` are **pinned differential oracles and CVE regression
  fixtures only — never shipped product**.
- **Shipping an oracle to green a gate the owned executor did not pass is _conformance
  laundering_.** Naming it is the point: it is the specific failure this clause exists to forbid,
  and it is indistinguishable from success in any report that counts gate colour alone.
- **Bootstrap lock.** K1-reference is the declared bootstrap (youki / Go-containerd) with a
  calendar fail-closed expiry; promotion to **K1-owned** is gated on the security-response
  process named in the process-law exits above. The two clauses are the same lock seen from
  either end.
- **Adversarial corpus, closed set.** Three regression classes are mandatory, each bound to its
  identity so a rename cannot silently drop one:

  | CVE | Regression class |
  |---|---|
  | `CVE-2019-5736` | `proc_self_exe_reexec` |
  | `CVE-2024-21626` | `fd_leak` |
  | `CVE-MOUNT-SYMLINK-RACE` | `mount_symlink_race` |

  CVE fixtures are the adversarial corpus, not a coverage statistic. An executor that passes the
  differential oracles but fails one of these has not passed.

This is recorded doctrine, not an Accept. Accept (a) remains gated on its state-machine /
recovery Definition of Done exactly as written above.

### Accept (b) — `os/`-layer retirement encode

#### D-B1 — Apex noun amend proposal (text only)

On Accept (b), a **follow-on** amend to ADR-0701 (separate PR) replaces the owned substrate
stack noun. The **destination** noun (full forever shape) is:

```text
k8s (projected) → node supervisor (owned PID1 stub + restartable supervisor child) → guest kernel
```

**Severability rule for Accept (b)-only (without Accept (a)):**
- D-B2 harvest + D-B3 precondition work MAY proceed.
- Apex noun amend under Accept (b)-only MUST name an **interim** bridge:
  `PID1 stub + external/runtime bridge (CONSUME path)` — **not** claim the full restartable
  NON-PID1 kubelet/runtime-controller child as already landed.
- **Retire/delete** of dual-truth `os/` halves that the destination topology assumes (D-B2 step 2)
  is **forbidden** until Accept (a) lands the child **or** a dated founder waiver names the
  interim bridge and residual risk.
- The full destination noun above requires **Accept (a)+(b)** (or Accept (b) after Accept (a)).

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
| PID1 stub + supervisor child crates (destination capability TBD) | create | Accept (a): full owned-runtime shape. Accept (b)-only: PID1 stub + harvest surfaces required by D-B2/D-B3 may land; retire/delete of destination-assumed `os/` halves blocked until Accept (a) or dated waiver; full NON-PID1 kubelet/runtime-controller child remains Accept (a)-gated |
| CRI compatibility profile `v1` contract + tests | create | RPCs, streaming, PLEG, errors, peer cred, rate limits, read-only set — Accept (a) |
| kill-9 continuity + upgrade reconnect + dual-supervisor race + stub respawn escalate tests | create | Mandatory encode evidence for Accept (a) DoD (Round-4 + Round-5) |
| Node telemetry / OpenSLO (or EV0 deferral-with-owner) + zero-trust checklist evidence | create | Accept (a) follow-on before K-stage promotion claims |
| Stub LOC ratchet fixture + CAS torn-write / n−1 schema tests | create | Round-5 cheap encode evidence for Accept (a) |
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
- Recovery DoD (Round-4 continuity + Round-5 respawn/lease/telemetry/zero-trust/FinOps) is on
  the critical path before Accept (a).
- K1-owned and promotion-gate claims stay blocked on process-law exits even while this ADR is
  Proposed.

### Operational

- Boot receipts become checked edges; crash-loops escalate to NotReady; stub restart storms
  escalate via budgeted Node condition / break-glass.
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
| **Accept (b) only** | Apex interim noun + `os/` harvest authorized after D-B3; retire/delete of destination-assumed halves blocked until Accept (a) or dated waiver; no full supervisor-child encode from (a) |
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
- Round-2/4/5 Discovery local artifact `e6ec1a68` — provenance only
- PR #1929 Round-4 amend + Round-5 DoD absorb; merges former owned-runtime + os/-retirement draft topics; vacates the colliding draft number reserved for PR #1644
