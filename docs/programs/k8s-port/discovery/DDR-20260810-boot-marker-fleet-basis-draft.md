---
doc_class: Program-Discovery-Decision-Record
doc_status: drafted
discovery_id: DDR-20260810-boot-marker-fleet-basis-draft
judgment_class: f1e-os-retirement-preconditions
recorded_at: 2026-08-10
owner: council-architecture
authority_tier: 3
---
# DDR-20260810-boot-marker-fleet-basis-draft

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-10) |
|---|---|---|
| Repository baseline | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` | Discovery proposal lane base. |
| Boot markers (live) | `kernel/core/asterinas-boundary` `BOOT_READY_MARKERS` | Includes `Welcome to NixOS` and `systemd[1]` strings. |
| Fleet/pin basis (live) | `specs/k8s-port/upstream-pin.json` + `infra/talos/installation-media/presets.yaml` | k8s minor derived from Talos presets. |
| F1(e) Proposed | ADR-0715 | OS-layer retirement; preconditions listed, not assumed done. |

## Record identity

- **Stable ID:** `DDR-20260810-boot-marker-fleet-basis-draft`.
- **Judgment class:** F1(e) preconditions — boot-marker contract + fleet-basis replacement **drafts**.
- **Status:** `drafted` discovery — **not** an apex amend; **not** a pin/marker live flip.
- **Recorded:** 2026-08-10.

## Authority fence

This record **MUST NOT**:

- amend ADR-0701 apex noun;
- edit `BOOT_READY_MARKERS` in `asterinas-boundary`;
- edit `upstream-pin.json` or Talos presets;
- charter-amend `os/` meta-directory.

Those flips wait on founder Accept of ADR-0715 plus same-wave implementation PRs.

## Judgment

### J1 — Why current boot markers are invalidated (draft)

Live `BOOT_READY_MARKERS` (5 regexes) include:

- `Welcome to NixOS`
- `systemd[1]: Startup finished`

Round-2 userspace ban + supervisor-as-OS path means NixOS login / systemd success strings are **soak evidence for the pinned ISO**, not forever product boot readiness for the owned supervisor. Replacement markers must describe **owned PID1 receipts** (mount → ABI stamp → net → runtime ready → kubelet register), not distro branding.

**Draft replacement classes (not encoded):**

| Marker class | Intent |
|---|---|
| `supervisor.pid1.mounted` | Pseudo-fs mount receipt |
| `supervisor.abi_matrix.stamped` | ABI-matrix stamp check (mismatch = refuse boot) |
| `supervisor.bootstrap.net_ready` | Minimal net precondition |
| `supervisor.runtime.ready` | Owned runtime libraries ready |
| `supervisor.kubelet.registered` | Projected kubelet registered (bootstrap taint) |

Exact regex/strings are deferred to an implementation PR after Accept.

### J2 — Fleet-basis replacement (draft)

`upstream-pin.json` currently derives the Kubernetes minor from Talos installation-media presets. Retiring the Talos-shaped OS noun requires a **replacement fleet basis** that does not die with `os/` harvest-then-retire.

**Draft options (not selected):**

| Option | Notes |
|---|---|
| A. Pin k8s minor directly in `upstream-pin.json` with digest | Drops Talos preset indirection |
| B. Named fleet SKU registry (Asterinas shared-kernel / stripped-Linux KVM) co-pinning k8s minor | Aligns with F1(a) pool matrix |
| C. Keep presets during bootstrap window with dated expiry | Parallel to Go containerd CONSUME bootstrap |

Selection is founder/F1(e) Accept territory.

### J3 — Charter amendment (pointer only)

`os/` is chartered Talos-class today. Harvest-then-retire needs a charter amendment naming harvest domains (network/install/disk/time + init-app PID1 primitives) vs delete domains (COSI/apid/trustd/config-v1alpha1). **Not drafted as charter text here.**

## Round-2 basis

OS-layer verdict confirmed; apex noun amend blocked on fleet-basis + boot-marker + charter preconditions (ADR-0715).

## Alternatives

| Alternative | Why rejected now |
|---|---|
| Flip markers/pins in this PR | Apex-adjacent; blocked on F1 Accept |
| Keep NixOS/systemd markers as forever law | Contradicts userspace ban / supervisor-as-OS |
| Silent dual-truth (old markers + new supervisor) | Dual-truth ban |

## Downstream blockers

1. Founder Accept/Reject of ADR-0715.
2. Same-wave PRs for marker set, fleet basis, and `os/` charter.
3. A1 ABI matrix evidence feeding F1(a) pool physics before private-kernel claims.

## Naming law

Forever nouns: **node supervisor**, **guest kernel**, **fleet SKU**, **boot receipt**. Ban `asterkube` / `kuberos`. Comparative prior art only when framed as **not adopted**.
