---
doc_status: published
id: ADR-0711
title: "Node kernel + pool matrix: Asterinas shared-kernel pools with co-selected stripped-Linux KVM SKU"
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
related: [ADR-0611, ADR-0704, ADR-0713]
milestone: F1
deliverables:
  - id: ADR-0711-D1
    description: "Select a permanent two-SKU node-kernel pool matrix: Asterinas hosts shared-kernel workloads (plus a TDX-guest trajectory); stripped-Linux KVM-capable pools are co-selected as a permanent SKU for private-kernel and private-kernel-attested workloads."
    exit_criteria: "A1 4-surface ABI matrix evidence is published (syscalls + proc/sys/cgroupfs files + netlink families + mount semantics) comparing pinned ISO profiles of runc/youki/containerd/kubelet against Asterinas v0.17.2's implemented set; founder Accept selects the co-SKU matrix OR Accept selects named fallback G5 (Linux/Talos-class kernel remains the node kernel; Asterinas remains soak/boot evidence only) when Critical gaps fire (cgroup v2 delegation, netlink, or overlayfs)."
    verified_by: "oya-ci-required"
  - id: ADR-0711-D2
    description: "Name fallback G5 as the only authorized retreat from D-1 without inventing a third kernel product."
    exit_criteria: "G5 trigger criteria are enumerated in this ADR; Accept of D-1 without A1 evidence is forbidden; Accept of G5 requires a recorded Critical-surface gap from the A1 matrix."
    verified_by: "oya-ci-required"
---
# ADR-0711: Node kernel + pool matrix: Asterinas shared-kernel pools with co-selected stripped-Linux KVM SKU

## Status

**Proposed.** Deliberately not Accepted: clause D-1 depends on **A1 4-surface ABI matrix
evidence** that is not yet published. Accepting the pool matrix before that measurement would
assert a kernel posture we have not verified. This ADR carries **no implement authority** while
Proposed.

Discovery input (not law): Round-2 synthesis in the local planning artifact
the Round-2 node forever-shape Discovery plan (local artifact id e6ec1a68) (founder F1(a) clause). Plan text cannot overrule
Accepted apex ADRs.

## Context

ADR-0611 deferred an Asterinas-as-canonical-node-kernel pivot; ADR-0704 ratified the k8s
port-engine path without selecting the forever node-kernel SKU matrix. Current boot evidence
for Asterinas proves only that an upstream ISO reaches a NixOS login prompt under QEMU TCG —
not that the Linux ABI surfaces required by kubelet, CNI, cAdvisor, eviction, or overlayfs are
present and enforcing.

Pool physics constrain the decision independently of brand preference:

- Asterinas exposes **no `/dev/kvm`**. It is a TDX *guest* trajectory target, not a KVM host.
- Therefore private-kernel isolation (VM / confidential VM) **cannot** pin to Asterinas host
  pools.
- Shared-kernel isolation **can** pin to Asterinas pools once A1 evidence clears Critical
  surfaces (netlink/netns/netfilter, cgroup-v2 enforcement fidelity, mount/overlay semantics).

Live law until Accept/Reject: ADR-0701 / ADR-0704 substrate gist; no kernel SKU flip may be
encoded from this Proposed ADR alone.

## Decision (proposed)

### D-1 — Permanent two-SKU pool matrix (gated on A1)

On Accept after A1 evidence:

1. **Asterinas pools** serve **`shared-kernel`** workloads, with a recorded TDX-guest trajectory
   (not a day-one claim).
2. **Stripped-Linux KVM-capable pools** are **co-selected as a permanent SKU** (not a temporary
   fallback) for **`private-kernel`** and **`private-kernel-attested`** workloads.
3. ValidatingAdmissionPolicy (or the live admission substrate until ADR-0710 Accept) MUST refuse
   `private-kernel*` RuntimeClasses on non-KVM pools.

A1 evidence is a **4-surface ABI matrix**, not a syscall count:

| Surface | Why load-bearing |
|---|---|
| Syscalls | Executor / sandbox allowlists |
| procfs / sysfs / cgroupfs files | cAdvisor (`/proc/stat`, `memory.current`, `cpu.stat`); eviction (`statfs`) |
| Netlink families | ROUTE/LINK/ADDR; kube-proxy has no netfilter-free mode — **Critical** |
| Mount semantics | `MS_SHARED`/`MS_SLAVE`, overlayfs whiteouts/xattrs, `pivot_root` |

Native snapshotter is preferred first on Asterinas; overlayfs is required on Linux shared-kernel
pools from day one of that SKU.

### D-2 — Named fallback G5

If A1 shows Critical gaps in cgroup v2 delegation, netlink, or overlayfs (or an equivalent
blocker that prevents shared-kernel node duties), founder Accept of **G5** instead:

- Linux / Talos-class kernel remains the **node kernel** SKU.
- Asterinas remains **soak / boot evidence** only — not a production pool SKU.
- No silent third product kernel may be introduced to paper over the gap.

G5 is a recorded Accept outcome of this ADR, not an agent-side escape hatch.

## Consequences

- Positive: isolation tiers map to pool physics; private-kernel does not pretend KVM exists on
  Asterinas; measurement precedes SKU law.
- Negative: two permanent kernel SKUs until/unless G5 collapses to one; A1 work is on the
  critical path before any encode of Asterinas as production shared-kernel.
- Operational: cell density SLOs for `shared-kernel` remain first-class (historical ADR-0338
  density caution, carried via ADR-0701 gist); VM-first is per-cell-class, not global.

## Rejected alternatives (proposed framing)

| Option | Why not default |
|---|---|
| Asterinas-only forever host | No `/dev/kvm`; breaks private-kernel* |
| Linux-only forever, Asterinas never | Premature; A1 may clear shared-kernel |
| Global microVM-everything | Density loss; collapses shared-kernel |
| Accept matrix without A1 | Asserts unverified ABI coverage |

Comparative prior art that used a Go-heavy node userspace on a transitional kernel stack is
**not adopted** as the forever product shape; this ADR decides kernel/pool SKUs only.

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept D-1** | Co-SKU matrix becomes amend authority for ADR-0701/0704 kernel placement prose; encode may proceed after A1 publish |
| **Accept G5 (D-2)** | Linux/Talos-class node kernel retained; Asterinas soak-only; D-1 closed as Rejected-for-now |
| **Reject ADR** | No SKU flip; live ADR-0701/0704 gist unchanged; plan Discovery remains non-law |

## Citation contract

Proposed — **not implement authority**. Authority surfaces (`CLAUDE.md`, `AGENTS.md`,
`docs/AGENTS.md`) MUST NOT cite this ADR as binding law while `status: Proposed`.
