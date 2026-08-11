---
doc_status: published
id: ADR-0712
title: "Owned node runtime: supervisor libraries, no manager daemon; CRI external face; Go containerd bootstrap CONSUME"
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
related: [ADR-0637, ADR-0638, ADR-0704, ADR-0715]
milestone: F1
deliverables:
  - id: ADR-0712-D1
    description: "Forever node runtime shape is owned supervisor libraries (CAS store, sandboxed pull workers, per-sandbox shims, owned executor) with NO long-lived container-manager daemon. CRI is an external compatibility face only with a closed consumer list (unlisted REFUSE). Internal components never route through CRI."
    exit_criteria: "Founder Accept records D-1; Reject restores CONSUME-external-runtimes posture without owned runtime encode. No PR may flip specs/k8s-port/scope.json to PORT containerd as product while this ADR is Proposed or if Rejected."
    verified_by: "oya-ci-required"
  - id: ADR-0712-D2
    description: "Propose (do not apply) an OWN disposition token in specs/k8s-port/scope.json vocabulary so first-party owned node runtime is expressible without colliding SCP-CONSUME-EXTERNAL-RUNTIMES (rank 95). Bootstrap = pinned Go containerd CONSUME with dated expiry + ledger row. CNI plugins remain CONSUME. D5 third-corpus neutrality proof uses ttrpc and/or go-cni — NOT containerd product PORT."
    exit_criteria: "ADR text names the required scope vocabulary edit, bootstrap CONSUME row shape, DVG-OWNED-NODE-RUNTIME ledger row intent, and runtime/ capability home OR destination ruling candidates; actual scope.json / divergence-ledger / capability-registry edits are out of band and blocked until Accept."
    verified_by: "oya-ci-required"
  - id: ADR-0712-D3
    description: "OVERRULE containerd as forever product PORT. Round-2 discovery rejects CONSUME→PORT of containerd as product; mechanical port-engine may still PORT first-party k8s A-prime including kubelet."
    exit_criteria: "Accept forbids sibling specs/containerd-port/ product registry and forbids treating containerd PORT as D5 third corpus; D5 proof corpus named as ttrpc/go-cni satellite."
    verified_by: "oya-ci-required"
---
# ADR-0712: Owned node runtime — supervisor libraries, no manager daemon

## Status

**Proposed.** Deliberately not Accepted: clauses D-1–D-3 are founder pause-and-pair shape
decisions. They wait on explicit founder **Accept or Reject**. While Proposed this ADR carries
**no implement authority** and MUST NOT be used to flip `scope.json`, the divergence ledger, or
the closed capability registry.

Discovery input (not law): Round-2 synthesis in the local planning artifact
the Round-2 node forever-shape Discovery plan (local artifact id e6ec1a68) (founder F1(b); todos `founder-f1-set`,
`apex-overrule-containerd` proposal surface, `k8s-port-scope-cri-port` proposal surface).

## Context

Live `specs/k8s-port/scope.json` disposition vocabulary is exactly
`[PORT, CONSUME, EXCLUDE]`. Rule `SCP-CONSUME-EXTERNAL-RUNTIMES` (specificity_rank 95) currently
classes `containerd_or_cri_runtime` (among others) as CONSUME. An owned first-party node runtime
is **inexpressible** in that vocabulary today: encoding it as PORT of containerd would assert a
product PORT Round-2 discovery overrules; encoding it as CONSUME would keep an external binary
as forever law.

Round-2 discovery (non-binding until Accept) overrules a container-manager daemon as forever
product:

- Restart-survival belongs to **durable on-disk sandbox records + per-sandbox shims reparented
  to PID1**, not to a long-lived manager process.
- CNCF / Sonobuoy conformance is measured at the **Kubernetes API**, not at the CRI socket.
- containerd is a poor second/third corpus for port-engine neutrality (ttrpc, `init()`-time
  reflection plugin registration, cgo paths); ADR-0637 already designates Talos-domain code as
  W0-D second corpus. D5 third-corpus neutrality should use a **small satellite** (ttrpc /
  go-cni), not a containerd product commitment.

Comparative prior art that kept a Go container-manager daemon as forever node userspace is
**not adopted**.

## Decision (proposed)

### D-1 — Owned runtime shape (no manager daemon)

On Accept, the forever node runtime is:

```text
node supervisor (owned; PID1 + runtime libraries)
  ├─ CAS image store (library)
  ├─ short-lived sandboxed pull workers
  ├─ per-sandbox shims (genuine process boundary; reparent to PID1)
  │    └─ owned executor LIBRARY (from OCI runtime-spec; youki/runc/crun = differential oracles only)
  └─ CRI server = EXTERNAL compatibility face only
       (closed consumer list: e.g. crictl, node-problem-detector, …; unlisted = REFUSE)
```

Internal components that we own (projected kubelet inside the supervisor wrapper) MUST NOT
route through CRI. CRI exists historically because kubelet and runtime were different projects;
ownership of both collapses that requirement for the internal path.

**BAN while/after Accept of D-1:** shipping a long-lived container-manager daemon as the
product forever shape. **BAN:** hand mini-CRI as forever semantics (bootstrap only if ever
used — Round-2 prefers dated Go containerd CONSUME instead).

### D-2 — Scope OWN token + Go containerd bootstrap CONSUME (proposal text only)

This clause **authors the required follow-on edits**; it does **not** apply them.

On Accept, a subsequent scoped PR (not this ADR alone) MUST:

1. Extend `disposition_vocabulary` in `specs/k8s-port/scope.json` with **`OWN`** — meaning
   first-party owned implementation that is neither a mechanical upstream PORT nor an external
   binary CONSUME.
2. Add a higher-rank rule for owned node runtime libraries (CAS / pull workers / shims /
   executor / CRI face) disposition **OWN**, so resolution does not fail-closed into
   `SCP-CONSUME-EXTERNAL-RUNTIMES`.
3. Add a **bootstrap** rule: pinned **Go containerd CONSUME** with **dated expiry**, digest pin,
   and a divergence-ledger row (intent name `DVG-OWNED-NODE-RUNTIME` / bootstrap companion as
   sequenced under the 2-rows/wave budget). Expiry MUST be a calendar date, not a vague wave
   slogan.
4. Keep **CNI plugin binaries CONSUME** (unchanged category).
5. Decide **capability home**: register a top-level `runtime/` capability **or** record an
   explicit destination ruling under an existing capability (`k8s` seam vs harvested supervisor
   substrate). The capability registry is `closed: true` today — amendment is required either
   way and is **blocked until Accept**.

### D-3 — Overrule containerd product PORT

On Accept:

- **Forbidden:** CONSUME→PORT of containerd as product forever shape.
- **Forbidden:** creating `specs/containerd-port/` as a product sibling registry for that purpose.
- **Required:** D5 third-corpus neutrality proof uses **ttrpc and/or go-cni** (satellite), not
  containerd product PORT.
- Mechanical **k8s** PORT (including kubelet) remains under ADR-0704; this overrule is scoped to
  the container-manager product, not to the k8s projection.

Oracle / licensing rows for youki, runc, crun, and OCI runtime-spec suites remain test artifacts,
not product PORT commitments.

## Consequences

- Positive: smaller privileged surface (three shrink-only ratchets become enforceable); no false
  "conformance requires containerd daemon" premise; scope vocabulary can express OWN.
- Negative: bootstrap CONSUME window must be dated and ledgered; OWN token is a schema change
  with fail-closed resolver implications; capability destination must be ruled before encode.
- Operational: closed CRI consumer list becomes a contract-tested allowlist; unlisted callers
  fail closed.

## Rejected alternatives (proposed framing)

| Option | Why not |
|---|---|
| Mechanical containerd PORT as forever product | Daemon fails existence test; worst corpus; false conformance premise |
| Hand mini-CRI forever | Semantic debt vs KEPs; Round-2 prefers dated Go CONSUME bootstrap |
| Keep Go containerd forever as product | Leaves Go/C TCB on the node critical path without OWN destination |
| Skip CRI entirely | Breaks closed external tools that legitimately speak CRI |

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept** | D-1..D-3 become amend authority for ADR-0701/0704 runtime prose; follow-on PRs may add OWN token, bootstrap CONSUME row, ledger row, and capability home — still via separate scoped changes |
| **Reject** | Live `SCP-CONSUME-EXTERNAL-RUNTIMES` posture remains; no OWN token; no owned-runtime encode from this Discovery |

## Citation contract

Proposed — **not implement authority**. Do not cite from authority surfaces as binding law while
`status: Proposed`.
