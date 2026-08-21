---
doc_status: published
id: ADR-0712
title: "Node kernel + pool matrix: Linux-pools primary; Asterinas soak until A1; attestation-capable private-kernel-attested pools"
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
related: [ADR-0611, ADR-0704, ADR-0714]
milestone: F1
masterplan_work_item: MPV2-0053
deliverables:
  - id: ADR-0712-D1
    description: "Select a permanent two-SKU node-kernel pool matrix after A1 evidence: Asterinas hosts shared-kernel workloads (plus a TDX-guest trajectory); stripped-Linux KVM-capable pools are co-selected as a permanent SKU for private-kernel and attestation-capable pools for private-kernel-attested. Until A1 is green, Linux pools are primary production and Asterinas remains soak/boot evidence only."
    exit_criteria: "A1 4-surface ABI matrix evidence is published (syscalls + proc/sys/cgroupfs files + netlink families + mount semantics) comparing pinned ISO profiles of runc/youki/containerd/kubelet against Asterinas v0.17.2's implemented set; founder Accept selects the co-SKU matrix OR Accept selects named fallback G5 (Linux/Talos-class kernel remains the node kernel; Asterinas remains soak/boot evidence only) when Critical gaps fire (cgroup v2 delegation, netlink, or overlayfs)."
    verified_by: "oya-ci-required"
  - id: ADR-0712-D2
    description: "Name fallback G5 as the only authorized retreat from D-1 without inventing a third kernel product."
    exit_criteria: "G5 trigger criteria are enumerated in this ADR; Accept of D-1 without A1 evidence is forbidden; Accept of G5 requires a recorded Critical-surface gap from the A1 matrix."
    verified_by: "oya-ci-required"
  - id: ADR-0712-D3
    description: "Attestation-capable pool constraint for private-kernel-attested: pool MUST advertise TEE hardware profile (selected TDX/SEV-SNP) AND relying-party reachability as pool properties; admission MUST deny private-kernel-attested unless both are present with evidence."
    exit_criteria: "Accept records the attestation-capable pool capability schema and deny rule; day-1 attested tier is labeled attested-identity (host in TCB); pool TEE+RP is necessary but not sufficient — attested identity/authz requires fresh nonce-bound quote validated by relying party before leaving quarantine; operator-excluded confidentiality (guest-pull) remains the F1 Isolation target, not a day-1 claim."
    verified_by: "oya-ci-required"
---
# ADR-0712: Node kernel + pool matrix — Linux primary; Asterinas soak until A1

## Status

**Proposed.** Deliberately not Accepted: clause D-1 depends on **A1 4-surface ABI matrix
evidence** that is not yet published. Accepting the pool matrix before that measurement would
assert a kernel posture we have not verified. This ADR carries **no implement authority** while
Proposed.

Live masterplan anchor (planning only, not implement authority):
[`/specs/masterplan.json#masterplan_v2.work_items[MPV2-0053]`](../../specs/masterplan.json)
(F1(a) pool-matrix package). Local Discovery artifact `e6ec1a68` is provenance input only and
MUST NOT be cited as live plan authority.

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
- Ordinary KVM supplies a private guest kernel but **does not** supply a confidential-computing
  attestation root. `private-kernel-attested` therefore requires a distinct
  **attestation-capable** pool capability.

Live law until Accept/Reject: ADR-0701 / ADR-0704 substrate gist; no kernel SKU flip may be
encoded from this Proposed ADR alone.

Bominal inheritance: no Bominal equivalent — oyatie override for owned node-kernel SKU matrix.

## Decision

### D-1 — Permanent two-SKU pool matrix (gated on A1) with Linux-primary interim posture

**Interim posture (nonbinding planning guidance while Proposed):** this paragraph is **not**
live placement law and **must not** be followed as if Accepted. Live law remains ADR-0701 /
ADR-0704 until founder Accept of D-1 or G5. The intended post-Accept interim (recorded here so
Accept does not invent it) is:

1. **Linux pools are primary** for production shared-kernel and private-kernel placement.
2. **Asterinas remains soak / boot evidence only** until A1 is green **and** founder Accepts
   D-1 (or Accepts G5, which keeps Asterinas soak-only permanently).

On Accept of **D-1** after A1 evidence (not on Accept of G5, and not on a generic ADR-level
Accept alone):

1. **Asterinas pools** serve **`shared-kernel`** workloads, with a recorded TDX-guest trajectory
   (not a day-one claim).
2. **Stripped-Linux KVM-capable pools** are **co-selected as a permanent SKU** (not a temporary
   fallback) for **`private-kernel`** workloads.
3. **Attestation-capable pools** (subset of, or sibling to, stripped-Linux KVM pools) are
   required for **`private-kernel-attested`** (see D-3).
4. ValidatingAdmissionPolicy (or the live admission substrate until ADR-0710 Accept) MUST refuse
   `private-kernel*` RuntimeClasses on non-KVM pools and MUST refuse
   `private-kernel-attested` on pools lacking attestation capability evidence.

Accept of **G5** does **not** authorize Asterinas production `shared-kernel` placement.

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

### D-3 — Attestation-capable pool constraint for `private-kernel-attested`

On Accept:

1. A pool that may schedule `private-kernel-attested` MUST advertise **both** as pool
   properties (machine-readable; admission-readable):
   - **TEE hardware profile** — the selected TDX or SEV-SNP profile (closed enum in the pool
     capability schema; not "any KVM").
   - **Relying-party reachability** — evidence that the attestation relying party endpoint(s)
     required by the platform are reachable from that pool's attestation path.
2. Admission MUST **deny** `private-kernel-attested` unless both properties are present with
   current evidence. KVM-only is insufficient. Deny rules alone are not enough for placement:
   RuntimeClass / isolation-property objects MUST also carry **scheduling.nodeSelector and
   tolerations** (ADR-0338 pattern) that bind `private-kernel*` to compatible pools so a pod
   cannot schedule onto an incompatible node after admission.
3. **Day-1 attested tier** is **attested-identity**: the host remains in the TCB and that fact
   MUST be explicitly labeled on the RuntimeClass / isolation property documentation and on
   any customer-facing attestation claim. Day-1 MUST NOT claim operator-excluded confidentiality.
4. **Pool properties are necessary, not sufficient, for attested authorization.** Scheduling
   admission may place a pod onto an attestation-capable pool only when (1)–(2) hold. Granting
   the `private-kernel-attested` **identity / authz context** (and any customer-facing attested
   claim) additionally REQUIRES a **fresh nonce-bound attestation quote** tied to the pod, the
   image measurement, and the node, validated by the relying party. Until that validation
   succeeds, the workload MUST remain **quarantined** from attested authorization (ordinary
   deny / non-attested identity only). A stale or misconfigured pool, or an unapproved guest
   measurement, MUST NOT receive attested identity solely from pool TEE profile + RP
   reachability.
5. **Operator-excluded confidentiality** (guest-pull / host-out-of-TCB) is the **F1 Isolation
   target** tracked under the isolation-names package ([ADR-0714](ADR-0714-isolation-property-runtime-tier-names.md))
   — not a day-1 encode claim of this ADR. That target inherits the same quote-before-authz
   rule with a stronger measurement profile (host out of TCB).

### D-3a — Harvested attestation chain (recorded 2026-08-19)

This clause records, in the ADR that owns the decision, the relying-party shape that until now
existed only inside `os/harness/attestation-relying-party` — a scaffold slated for deletion with
the rest of the hand-written `os/` tree. Recording it here means that deletion costs no doctrine.
It is recorded design law, not an Accept; D-1 and D-3 remain gated exactly as written.

**The chain, end to end:**

```
guest collector          ->  off-node OWNED verifier      ->  short-TTL signed    ->  existing SVID issuer
(configfs-tsm,               (pinned AMD KDS / Intel PCS)     attestation RESULT      + Cedar context keys
 nonce-bound report_data)
```

**Invariants, in the order they bite:**

1. **UNKNOWN is never PASS.** Stale or unreachable collateral yields verdict **UNKNOWN**, and
   UNKNOWN must be treated as a failure to attest — never as a pass, and never as an absent
   check. This is what a naive implementation gets wrong under provider outage, which is exactly
   when it matters most.
2. **The verifier is owned and off-node.** Verification does not run on the node being attested,
   and the KDS/PCS endpoints are pinned rather than resolved at use time.
3. **The result is short-TTL and signed**, and it feeds the existing SVID issuer plus Cedar
   context keys rather than establishing a parallel identity path.
4. **Hardware-agnostic evidence schema first**, with SEV-SNP / TDX / ARM-CCA adapters behind it —
   not a schema shaped around whichever TEE lands first.
5. **Day-1 `private-kernel-attested` is attested-identity**, host in TCB, explicitly labeled.
   Operator-excluded confidentiality (guest-pull) is the F1 Isolation target and MUST NOT be
   claimed on day one. This restates D-3(3) from the evidence side, deliberately: the two clauses
   are the same promise made once to admission and once to the customer.


## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | Notes |
|---|---|---|
| `specs/` pool / node capability schema (follow-on) | create/update | Attestation-capable pool properties; blocked until Accept |
| Admission VAP / live substrate policy | update | Deny `private-kernel-attested` without TEE+RP evidence; blocked until Accept |
| `docs/decisions/ADR-0701-*.md` | amend (follow-on) | Kernel/pool placement prose after Accept |
| A1 evidence packet under `docs/` or `evidence/` | create | Gate for Accept of D-1 |

No crate/file encode may land from this ADR while `status: Proposed`.

### Integration via Workflow + Ontology

Not applicable — this ADR decides pool/kernel SKU law and admission deny predicates. Integration
points live in the k8s admission and node-pool µservices' PRDs after Accept.

### Positive

- Isolation tiers map to pool physics; private-kernel does not pretend KVM exists on Asterinas.
- Attested workloads cannot be admitted on ordinary KVM without TEE+RP evidence.
- Measurement precedes Asterinas production SKU law; Linux-primary interim avoids premature flip.

### Negative

- Two permanent kernel SKUs until/unless G5 collapses to one; A1 work is on the critical path.
- Attestation-capable pool inventory is an additional operational surface.

### Operational

- Cell density SLOs for `shared-kernel` remain first-class (historical ADR-0338 density caution,
  carried via ADR-0701 gist); VM-first is per-cell-class, not global.
- CI: Accept evidence rides `oya-ci-required`; no authority-surface citation while Proposed.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none while Proposed |
| `cross-product-refusal` (LEAN-A2) | Not affected | none |
| `port-location` | Not affected | none |
| `layer-correctness` | Not affected | none |
| `composition-root-only` | Not affected | none |
| `sdk-kernel-only` | Not affected | none |

No new port traits in this Proposed record. Follow-on Accept encode introduces pool-capability
ports in the owning node/admission capability, not in this ADR body.

## Alternatives considered

**Alternative 1 — Asterinas-only forever host**
- Pros: single kernel SKU branding.
- Cons: no `/dev/kvm`; breaks `private-kernel*`.
- Reason rejected: pool physics forbid it.

**Alternative 2 — Linux-only forever, Asterinas never**
- Pros: simpler ops.
- Cons: premature; A1 may clear shared-kernel.
- Reason rejected as default: G5 remains the recorded retreat after measurement.

**Alternative 3 — Admit `private-kernel-attested` on any KVM pool**
- Pros: simpler scheduling.
- Cons: ordinary KVM lacks attestation root; false confidentiality/identity claims.
- Reason rejected: D-3 attestation-capable constraint.

**Alternative 4 — Accept matrix without A1**
- Pros: faster encode.
- Cons: asserts unverified ABI coverage.
- Reason rejected: gated Proposed policy.

## What Accept / Reject means

| Outcome | Effect |
|---|---|
| **Accept D-1** | Co-SKU matrix becomes amend authority for ADR-0701/0704 kernel placement prose; encode may proceed after A1 publish |
| **Accept G5 (D-2)** | Linux/Talos-class node kernel retained; Asterinas soak-only; D-1 closed as Rejected-for-now |
| **Reject ADR** | No SKU flip; live ADR-0701/0704 gist unchanged; MPV2-0053 remains planning-only |

## Citation contract

Proposed — **not implement authority**. Authority surfaces (`CLAUDE.md`, `AGENTS.md`,
`docs/AGENTS.md`) MUST NOT cite this ADR as binding law while `status: Proposed`.

## References

- Live masterplan: `MPV2-0053` in `/specs/masterplan.json#masterplan_v2.work_items`
- ADR-0611 (Asterinas deferral), ADR-0701 / ADR-0704 (live substrate gist)
- ADR-0714 (isolation-property names; attested-identity vs operator-excluded target)
- Round-2 Discovery local artifact `e6ec1a68` — provenance only, not live plan authority
- PR #1929 F1 founder Proposed apex set; vacated the prior draft number (now reserved for PR #1644 Swarm Delivery Law integ-branch topology) so this file is ADR-0712
