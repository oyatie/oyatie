---
doc_class: Program-Discovery-Decision-Record
doc_status: drafted
discovery_id: DDR-E0-20260810-node-stack-forever-shape
judgment_class: node-stack-forever-shape
recorded_at: 2026-08-10
owner: council-architecture
authority_tier: 3
---
# DDR-E0-20260810-node-stack-forever-shape

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-10) |
|---|---|---|
| Repository baseline | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` | E0 discovery encode lane base. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Pinned program input; not consumed here. |
| Engine | `build/port-engine/*`, v0 | Not in force as a producer for this record. |
| Neutral rule pack | `specs/port-rules/**`, v0 | Not in force. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Not in force. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This record emits no receipt. |
| Program authority | ADR-0704 (live apex). ADR-0637 / ADR-0638 archived provenance | Discovery record only. |

## Record identity

- **Stable ID:** `DDR-E0-20260810-node-stack-forever-shape`.
- **Judgment class:** node-stack forever shape (supervisor-as-OS, owned runtime libraries, isolation tiers, attestation, surface ratchets).
- **Status:** `discovery` — record, **not** doctrine, **not** Accepted apex.
- **Recorded:** 2026-08-10.
- **Owner role:** `council-architecture`.
- **Honest-ladder phase:** E0 (encodable subset; blocked items remain F1/W0).

## Authority fence

This record **MUST NOT** be read as:

- an amendment to ADR-0701 or any other Accepted apex;
- a flip of `specs/k8s-port/scope.json` disposition rows;
- a divergence-ledger row or test-id enumeration;
- a capability-registry amendment;
- a fold into the live programme SSOT while that SSOT is mid-reorg;
- acceptance of ADR-0710 (remains Proposed; see plane-split companion record).

E0 authorizes **encoding the judgment as a discovery record only**. Downstream encode into apex,
scope, ledger, or registry requires the matching F1 founder ADR and/or W0 artifact exit.

## Judgment

### J1 — Mechanical Kubernetes PORT stays; PID1 stays a minimal stub

Mechanical port-engine PORT of first-party Kubernetes A-prime (including kubelet) remains the
product path. **PID1** is a **minimal reaper/launcher stub** only (mount/reap/signals, bootstrap
networking preconditions, machine-config ingestion shaped as Kubernetes config, node identity
bootstrap hand-off). The projected kubelet and runtime-controller run in a **restartable
non-PID1 supervisor child**. Per-sandbox shims survive via **subreaper + durable-record
reconciliation**; upgrades are restart+reconnect; a **kill-9 continuity** test is required so
one panic cannot lose all pods (Round-4 adversarial fix).

**Ban:** hand-maintained / kube-rs paraphrase kubelet. **Ban:** Go kubelet as forever product
userspace (bootstrap CONSUME of upstream binaries for canary/validation is a validation tactic, not
the forever shape). **Ban:** placing projected kubelet + runtime-controller in PID1.
### J2 — No container-manager daemon product; owned runtime libraries + CRI faces

There is **no** long-lived container-manager daemon as forever product, and **no** containerd
product PORT. Forever shape:

- **Owned runtime libraries** inside the node supervisor: CAS image store, short-lived sandboxed
  pull workers, per-sandbox shims, owned executor library.
- **CRI semantics stay canonical** (Round-3/4 lock):
  - **In-process CRI-shaped transport** — owned trait generated from the pinned CRI proto; the
    projected kubelet drives the owned runtime through this seam (mechanical PORT keeps its
    upstream runtime path; this is **not** “rewrite kubelet off CRI”).
  - **External Unix socket** — versioned compatibility profile **v1** (RPCs, streaming server,
    evented PLEG, gRPC error-code semantics, peer creds, rate limits, read-only RPC set) for a
    closed, contract-tested external consumer list (for example crictl, node-problem-detector).
    Unlisted external consumers = REFUSE. Profile is **not** a binary-name allowlist.
  - Device plugins remain on the **Device Plugin API**, not the external CRI socket.
- **Bootstrap only:** K1-reference declared bootstrap (pinned youki/Go-containerd) as **CONSUME**
  with a **calendar-dated fail-closed expiry** toward K1-owned; youki/runc/crun remain differential
  oracles and are **never shipped** to green a gate. Future ledger row for the CONSUME is
  **out of E0** — do not invent rows here.

**Ban:** hand mini-CRI in any role. **Ban:** shipping a manager daemon as the forever product
without a founder ADR that explicitly overrules this derivation. **Ban:** collapsing Device Plugin
API into the external CRI face.
Comparative reject (not adopted): prior-art stacks that keep a forever Go container-manager + Go
kubelet userspace as the product shape are rejected for the forever path; they may inform bootstrap
or soak evidence only when explicitly time-boxed.

### J3 — Owned executor is a shim library; oracles are not the product

The OCI executor is an **owned library of the per-sandbox shim**, not a separate forever binary.
youki / runc / crun are pinned **differential-test oracles** and CVE regression fixtures behind the
same trait — **never shipped** to green a gate the owned executor did not pass (**conformance
laundering ban**). Escape hatch only as a future **ledgered**, dated-expiry CONSUME adapter on the
compound trigger (parity slip ≥ 2 waves AND spike shows no allowlist savings AND oracle actively
maintained) — that ledger row is **blocked on W0/F1 sequencing**, not authorized by this record.

### J4 — Isolation-property tier names on a pool matrix

Isolation mechanism names (what the workload is isolated *from*):

| RuntimeClass name (unbranded) | Isolation property |
|---|---|
| `shared-kernel` | shares the host/guest kernel with other pods on the pool |
| `private-kernel` | private kernel via KVM-backed VM |
| `private-kernel-attested` | private kernel plus relying-party attestation |

The **placement axis** (general vs edge-tuned hardware: SR-IOV, hugepages, CPU pinning) is
**orthogonal** and restores the Tier-3 nodepool contract without inventing a fourth isolation tier.
Trust classification 0..3 (who wrote the code) remains a separate axis — do not collapse trust into
isolation mechanism.

**Pool physics:** Asterinas exposes no `/dev/kvm` (TDX *guest* trajectory). Therefore
`private-kernel`* classes **cannot** schedule on Asterinas pools; they pin to **KVM-capable
stripped-Linux host pools** (co-selected permanent SKU). Linux pools are the **primary production
path**; Asterinas shared-kernel is **soak** until the A1 ABI/kernel-service matrix (including
io_uring, seccomp, and device/driver rows) goes green. Asterinas pools serve `shared-kernel`
(+ future TDX-guest role).

**Attested placement (split from ordinary private-kernel):** KVM capability alone admits
`private-kernel` but **not** `private-kernel-attested`. The attested class additionally requires
an **attestation-capability** pool label / scheduling constraint (SEV-SNP, TDX, CCA, or another
quote source the J6 collector can produce). Day-1 attested claim is **attested-identity** (host
in TCB, explicitly labeled); operator-excluded confidentiality (guest-pull) is the F1 Isolation
target — not a day-1 claim.

VAP MUST forbid `private-kernel`* on non-KVM pools and forbid `private-kernel-attested` on pools
lacking attestation capability when enforcement lands (enforcement remap is F1(c), not E0). VAP/CEL
itself remains target-pending-F1(d) (which may resolve as explicit Reject per ADR-0710 D-8); live
admission law today remains ADR-0701/Kubewarden.
**Ban:** branded RuntimeClass names (including vendor brands such as `kata` as a class name).
Kata-as-component dissolves: the shim compiled for the guest role is the same code; Cloud
Hypervisor remains the VMM.

### J5 — Supervisor IS the OS; Talos-shaped OS layer harvest-then-retire

The Talos-style **OS layer noun is retired** as forever product. The node supervisor **is** the OS
substrate. Disposition for the existing `os/` tree: **harvest-then-retire** — keep network / install
/ disk / time domains and `init-app` PID1 primitives (mount/reaper/switch_root as the **minimal
stub**); delete dual-truth simulation halves (COSI, apid, trustd, `config-v1alpha1`, controller
runtime) only once F1(e) preconditions land (fleet-basis replacement, boot-marker contract,
charter amendment, **and** a named SVID issuance/verifier replacement with a migration edge for
live IAM adapters that currently depend on `os/core/trustd-domain` /
`TrustdEcdsaIssuanceBackend`). Do **not** schedule trustd deletion as “simulation half” removal
while J6 still consumes the existing SVID issuer path.

Genuine residuals as **siblings, not an OS noun:** upgrade actor (A/B install + rollback),
out-of-band break-glass node API, NTP/disk-crypto clients.

**Target apex noun (proposal only — F1(e), not encoded here as Accepted):**
`k8s (projected) → node supervisor (minimal PID1 stub + restartable non-PID1 supervisor child) →
guest kernel`, with upgrade actor and break-glass as named siblings. **This discovery record does
not amend ADR-0701.**
### J6 — Relying-party attestation

Attestation follows the relying-party pattern (RATS / KBS):

1. Guest collector (configfs-tsm, nonce-bound report data).
2. **Off-node owned verifier** with pinned collateral; stale collateral ⇒ verdict **UNKNOWN, never PASS**.
3. Short-TTL signed **attestation result** (not raw evidence).
4. Consumed into the existing SVID issuer path + Cedar context keys for authorization.

Extend the existing `ConfidentialPlatform` trait (SNP / TDX / ARM CCA). Label-only “confidential”
without attestation→Cedar is banned. Activation hooks and ledger rows are **blocked on W0/F1**.

### J7 — Surface law = three shrink-only ratchets

Surface budget is not a slogan. Declare and ratchet down, never up without founder override:

1. Per-tier declared syscall-allowlist size.
2. Count of long-lived privileged processes.
3. Count of hostile-input parsers running outside a sandbox.

A1 (W0 entry) publishes the **4-surface ABI matrix** (syscalls + proc/sys/cgroupfs files + netlink
families + mount semantics) that feeds F1(a). This record does not claim that matrix exists yet.

## Round-2 basis

Encodes Round-2 FINAL SHAPE consensus as amended by Round-3/4 locks: owned runtime libraries +
canonical CRI (in-process trait + external profile v1); Go containerd / youki = dated K1-reference
bootstrap CONSUME only with owned destination; owned executor as shim library with oracle law;
isolation-property tier names on pool matrix (Linux primary; Asterinas soak; attested placement
split); OS-layer retirement / supervisor-as-OS with **minimal PID1 stub** + restartable non-PID1
child; relying-party attestation (day-1 = attested-identity); surface ratchets. Round-1 locks that
conflict (containerd product PORT; G6 vendored-youki default; process/kata/confidential naming as
forever; “internal MUST NOT route through CRI” without an in-process CRI-shaped seam) are
**superseded** and MUST NOT be re-encoded as fact.

## Alternatives

| Alternative | Disposition |
|---|---|
| Mechanical containerd PORT as forever product | OVERRULED round 2 — manager fails existence test; conformance is at the Kubernetes API |
| Hand mini-CRI | BAN — semantic forever liability |
| Forever Go container-manager + Go kubelet userspace (prior art; **not adopted**) | Rejected as forever product; dated bootstrap CONSUME only |
| Vendored youki `libcontainer` as shipped default (round-1 G6) | REVISED — oracle / ledgered escape only |
| Global microVM-first | Rejected — density is per-cell-class; `shared-kernel` is first-class |
| NixOS/systemd/Nix store as forever product userspace | BAN — ISO = soak/evidence only |

## Downstream blockers

| Item | Blocked on |
|---|---|
| Apex noun amend (supervisor-as-OS) | F1(e) founder ADR; fleet-basis + boot markers + `os/` charter + SVID issuance/verifier replacement before trustd harvest |
| OWN disposition token + Go-containerd bootstrap scope row | F1(b); `scope.json` vocabulary today is PORT/CONSUME/EXCLUDE |
| Tier rename + VAP enforcement remap + attestation-capability pool labels | F1(c); Kyverno→VAP enforcement before rename (VAP/CEL live-law flip remains F1(d)) |
| Attestation adapter activation + Cedar context keys | W0 ledger test-ids + ConfidentialPlatform extension |
| Ledger growth (`DVG-OWNED-NODE-RUNTIME`, oracle escape, …) | Baseline five rows must gain ratified `test_ids` first; grandfathered existing OS rows; then weighted budget + 2/wave cap |

`F1(a)`–`F1(e)` and sibling Round-3/4 labels (A1, K1/K2) are **program sequencing tags** for
founder-ADR / soak work tracked on the node-stack program (F1 drafts on #1929; founder Accept
remains external). They are **not** a parallel masterplan namespace and do not invent `MPV2-*`
IDs here; mapping into `specs/masterplan.json` work items is F1 encode / dispatcher work, not E0.
## Naming law

Forever nouns used in this record: **node supervisor**, **guest kernel**, **owned runtime**,
`shared-kernel` / `private-kernel` / `private-kernel-attested`, Asterinas-or-Linux **pool SKUs**.

**Ban adopting** the product/public names `asterkube` or `kuberos` (or branded RuntimeClass /
capability / crate / apex nouns of that class). Comparative mention of prior art is allowed only
when framed as **rejected / not adopted**, never as the forever product name.
