---
id: ADR-0382
status: Superseded
planning_impact: true
deciders: founder, ops-platform, council-architecture
date: 2026-05-28
owner: ops-platform
supersedes: []
superseded_by: [ADR-709]
related: [ADR-0375, ADR-0378, ADR-0381, ADR-0148, ADR-0083]
related_specs: [/specs/deployment-ops-contract.json]
milestone: M-LOCAL-CI-SUBSTRATE
depends_on: [ADR-0375]
door: two-way
affected_surfaces:
  crates: []
  microservices: []
  specs: []
deliverables:
  - id: ADR-0382-D1
    description: "Sidero Metal management cluster bootstrap. A small 'meta' Kubernetes cluster (initially a single Talos VM via the same vfkit substrate as ADR-0378, or any other Kubernetes — kind/k3s acceptable) hosts: sidero-controller-manager (the Sidero CRD + reconciler set: Server / ServerClass / Environment / MetalMachine / MetalCluster), cluster-api-provider-sidero (the CAPI InfraProvider that maps CAPI Cluster + Machine resources to Sidero-provisioned bare-metal nodes), and the static DHCP/TFTP/iPXE services Sidero ships. Bootstrap automation: a `clusterctl init --infrastructure sidero` plus the Sidero installer chart, applied via ArgoCD once the meta cluster exists. Hyperscaler-lens: Sidero Metal is Apache 2, actively maintained (Sidero Labs ships quarterly), self-hostable, and the OSS analogue of what hyperscalers run for internal bare-metal orchestration (Equinix Metal stack, OpenStack Ironic, etc.) — passes (a)-(d)."
    exit_criteria: "`kubectl -n sidero-system get pods` shows sidero-controller-manager + cluster-api-provider-sidero Running; `kubectl get serverclass any` returns a default ServerClass; the meta cluster reaches the bare-metal LAN's DHCP/TFTP endpoints."
    verified_by: "kubectl get pods + ServerClass present + a sample iPXE boot from the bare-metal LAN reaches the Sidero TFTP server."
  - id: ADR-0382-D2
    description: "Zero-touch node enrollment via PXE + iPXE chain. On power-on, an empty bare-metal machine PXE-boots → DHCP option 67 hands it Sidero's iPXE script → iPXE pulls the Talos kernel + initramfs (kernel cmdline includes the Sidero MetalMachine selector). On first boot the node registers as a Sidero `Server` resource (discovery mode); admin labels it (e.g., `oya.cell/foundation=true`, `oya.machine/role=cp`) which routes it to the matching ServerClass. The Talos installer then runs from the iPXE-served kernel + initramfs and lays down Talos on the local disk. Subsequent boots come up from local disk. Required infra: DHCP + TFTP on the bare-metal management VLAN — Sidero ships both as part of the management cluster install (no external dnsmasq required for a single-VLAN dev/lab setup; production-multi-VLAN deployments would use a dedicated DHCP relay)."
    exit_criteria: "A factory-fresh machine plugged into the bare-metal LAN PXE-boots, registers as a Server, and reaches 'discovered' state without operator action beyond cabling + power-on."
    verified_by: "`kubectl get servers` shows the new machine in 'discovered' state within 5 minutes of power-on."
  - id: ADR-0382-D3
    description: "CAPI resources for the first control-plane node — a `Cluster` (with `infrastructureRef: MetalCluster`), a `TalosControlPlane` (CP machine spec: image=Talos vX.Y.Z, certSANs, talos config patches per ADR-0381 D2 cell-foundation.yaml), and a `MetalMachine` matched against the labelled Server from D2. The CAPI controller drives Sidero to provision Talos onto the matched Server, bootstrap etcd, and produce a kubeconfig. Subsequent CPs (etcd quorum) + workers + specialty pool nodes (ci/storage per ADR-0381 D2) follow the same pattern: label a Server, declare a MetalMachine, watch Sidero provision."
    exit_criteria: "`kubectl get cluster <name>` reports `Ready: true`; `kubectl get kubeconfig <name> -o jsonpath=...` returns a valid kubeconfig; `kubectl --kubeconfig=that get nodes` shows the first CP node Ready (after the in-cluster CNI is installed — Cilium per ADR-0148)."
    verified_by: "End-to-end smoke: a labelled Server transitions through Discovered -> Allocated -> Provisioned -> the CAPI Cluster reports Ready; kubectl exec into a system pod from the resulting kubeconfig succeeds."
  - id: ADR-0382-D4
    description: "One-command bring-up script — `infra/talos/bare-metal/up.sh` — that takes cold hardware to a green `kubectl get nodes` without operator hand-holding beyond power-on. Sub-commands: `up.sh check` (preflight: LAN reachability, DHCP relay sanity, Sidero CRDs in management cluster); `up.sh bootstrap-mgmt` (idempotent management-cluster install per D1); `up.sh enroll <serial> --role cp` (Server label + MetalMachine apply); `up.sh up --role cp --count 3` (declare TalosControlPlane with replicas=3 + MetalMachine matchers; wait for Cluster Ready); `up.sh up --role worker --count N --cell ci|storage|tenant` (same for worker pools per ADR-0381 D2); `up.sh status` (kubectl get cluster + get servers in one screen). Mirror of the talos-local.sh idiom (subcommands, idempotent, exit-code-clean) so the operator's mental model is the same across local + bare-metal."
    exit_criteria: "From a fresh machine with PXE enabled on the NIC, running `up.sh up --role cp --count 1` returns success within N minutes and `kubectl get nodes` shows the CP Ready."
    verified_by: "Live test on a bare-metal lab machine post-D1-D3 landing. Smoke-only until then."
purpose: >
  Capture the decision to use Sidero Metal as Oyatie's bare-metal Talos
  provisioning substrate, distinct from the vfkit local-VM substrate
  (ADR-0378) used on the macOS dev box. Zero-day automated bring-up — cold
  hardware to green CP + CAPI #1 — without manual ISO authoring or
  hand-running talosctl per machine. REJECTED 2026-07-30 without ever being
  accepted: it proposed a second bare-metal provider one day after ADR-0375
  (Accepted) had already ratified and wired Metal3/CAPM3, and never evaluated
  Metal3 in its own Alternatives Considered. Retained as the record of the
  decision-space and hyperscaler-lens validation; no implementation IP follows.
---

# ADR-0382: Bare-metal Talos zero-day bring-up via Sidero Metal

## Status

**Rejected (2026-07-30).** Never accepted; sat at Proposed for two months. The
design content below is retained deliberately as the record of the decision-space
and the hyperscaler-lens validation — it is history, not authority.

### Why rejected — the adjudication this ADR never made

This ADR proposed **Sidero Metal** as the bare-metal Talos provisioning layer one
day after ADR-0375 (**Accepted**, 2026-05-27) had already ratified a bare-metal CAPI
infra provider and wired it: `infra/capi/init.sh` pins
`INFRA="oci:v0.24.0,aws:v2.11.1,metal3:v1.13.0"`, and `infra/capi/clusters` renders
per-cell clusters with `substrate` one of `oci | aws | metal3`.

**Metal3/CAPM3 does not appear anywhere in this ADR's Alternatives Considered**,
which weighs only Tinkerbell, MAAS, manual `talosctl`, Matchbox, and
hyperscaler-managed offerings. The two ADRs never met. Note also that ADR-0375's
own rejection names **Sidero Omni** — the proprietary fleet manager — which is a
*different product* from the Apache-2 Sidero Metal proposed here, so no prior
adjudication covered this choice either.

Rejected rather than superseded because there is nothing to supersede: the ratified
provider (Metal3/CAPM3 v1.13.0, ADR-0375 D2) stands unchanged, and this ADR
proposed a parallel second provider without retiring the first.

Three further grounds, each verified at rejection time:

1. **Every precondition is absent.** D1 requires a management cluster running
   Sidero + `cluster-api-provider-sidero`; D2 requires factory-fresh hardware
   PXE-booting through iPXE; the LAN requires DHCP-relay control plus TFTP; the
   Helm values require BMC access; and the stated lifecycle hands the manifests to
   Argo CD, which is not installed on any cluster. None of these exist.
2. **The provisioning destination is already port-backed and owned.**
   `k8s/ports/cluster-lifecycle-api` exposes `provision_cluster`, and
   `k8s/adapters/control-plane-host-adapter-capi` is the honest-deferred CAPI
   adapter behind it. `infra/sidero-metal/**` was raw Helm values and raw CRs
   referencing no owned port — the unmediated second copy that
   transient-substrate doctrine forbids.
3. **The SideroLink protocol is already reimplemented in owned Rust.**
   `os/core/siderolink-domain` is `no_std`, models the provision API, and exposes a
   `ProvisionService` trait plus a `Disabled → Configured → Provisioned → Up` state
   machine. Deleting the vendor YAML destroys no engineering.

ADR-0536 D-3 independently classifies Cluster API as an adapter to be replaced,
and ADR-0482 (Accepted) already ordered the amendment class this ADR sits inside.

### What this rejection does and does not remove

Removed: `infra/sidero-metal/**` (5 files, referenced by no ADR by path), and the
`sidero_zero_day_matrix` row plus the ADR-0382 authority block from the TALOS-001
substrate slice.

**Retained:** `infra/capi/**` — it is the named deliverable of ADR-0375 D2/D3 and is
untouched by this rejection.

Also retained, and left broken on purpose: `infra/talos/bare-metal/up.sh`, which
ADR-0523 (Accepted) ledger item 4 admits as irreducible glue independently of this
ADR, so deleting it is a declared one-way door requiring separate re-justification.
Be precise about the damage rather than understating it: `up.sh` defaults
`SIDERO_DIR` to `$ROOT/infra/sidero-metal`, so **five of its six subcommands are now
non-functional** — `check` hard-exits on the first missing manifest, and
`enroll`/`up`/`down` apply files that no longer exist; only `status` is unaffected.
It also remains a live entry in the shell-exception budget
(`rust-first-automation-policy.json` `exceptions[5]`, `temporary_legacy_bridge`)
whose replacement plan promises a Rust/GitOps controller for an operation the script
can no longer perform. Tracked as `F-INFRA-SIDERO-BRINGUP-SCRIPT-DEAD`; the
disposition is either deletion or repointing it at the ratified Metal3/CAPM3
provider.

Left as-is, deliberately out of scope: `cloud/cloud-kernel` anchors ADR-0382 in its
`manifest.json` and in three crate `adr_anchors` arrays. That anchor is
self-disclaimed as context — *"relevant to future Talos-compatible boot/provisioning
verification but not to pure kernel seams"* — and is gate-inert, so re-curating
another capability's anchor list belongs to its owner, not to this rejection.

Reversal, if bare-metal fleet provisioning is later wanted: re-propose against the
owned `cluster-lifecycle-api` port, adjudicate Metal3 vs Sidero Metal explicitly,
and restore the deleted files from this branch's parent commit.

## Context

The substrate stack so far:

- ADR-0375 — Talos + Cluster API + Argo CD fleet substrate (production fleet
  pattern).
- ADR-0378 — vfkit + Talos canonical **local** substrate (macOS dev box;
  single-VM bring-up via `talos-local.sh`).
- ADR-0381 D2 — multi-node Talos cell topology (CP / Worker / Specialty
  pools; lands on the vfkit substrate first, then bare metal).

What's missing: a zero-day automated path from **cold bare-metal hardware**
(brand-new server, NIC plugged into a LAN) to a green `kubectl get nodes`
showing the first CP + CAPI registered, with no per-machine ISO authoring,
no manual `talosctl gen config`, no per-machine kernel-cmdline editing.

The vfkit local substrate (`talos-local.sh`) works at the VM level: it
allocates a disk, writes a fixed MAC, boots, applies the machineconfig over
the Talos API once the VM has a DHCP lease. That works for VMs (we can
fabricate the disk + serial console + EFI store) but does NOT generalize to
bare metal — a physical machine doesn't let us pre-write its disk, and the
"Talos in maintenance mode" state is reached via PXE/installer media rather
than a pre-built disk image.

The standard bare-metal-K8s pattern: PXE + iPXE chain + a management cluster
that owns discovery + provisioning + lifecycle. Sidero Labs (the Talos
maintainers themselves) ship this as **Sidero Metal** — Talos-native, CAPI-
integrated, Apache 2, and is what they run for managed bare-metal Talos.

## Decision

### Substrate: Sidero Metal

**Choice**: Sidero Metal (github.com/siderolabs/sidero) as the bare-metal
Talos provisioning layer; cluster-api-provider-sidero as the CAPI
InfraProvider that maps `Cluster` / `Machine` to provisioned bare-metal nodes.

**Hyperscaler-lens validation** (per memory `hyperscaler-lens-architectural-filter`):

- **(a) Active upstream**: Sidero Labs releases quarterly; v0.7.x as of 2025
  with active CAPI v1beta integration. Passes.
- **(b) License-clean**: Apache 2 across both `siderolabs/sidero` and
  `siderolabs/cluster-api-provider-sidero`. Passes.
- **(c) Fully self-hostable**: runs as Kubernetes CRDs + controllers + DHCP
  + TFTP + iPXE in our own management cluster. No managed-service equivalent
  is consumed. Passes.
- **(d) Hyperscaler-internal-equivalent**: Sidero Metal is the OSS analogue
  of what hyperscalers run for internal bare-metal orchestration (Equinix
  Metal's Tinkerbell-derived stack, OpenStack Ironic, AWS's internal Nitro
  provisioning system). It is the open-source canonical for self-hosted
  bare-metal Kubernetes. Passes.

### Topology + integration

- **Management cluster** (D1): runs Sidero + CAPI Sidero provider. Bootstrap:
  initially a single Talos VM via the same vfkit substrate (`talos-local.sh
  up --role single` → install Sidero on top), or any pre-existing
  Kubernetes (kind/k3s acceptable for the initial bootstrap). Once the
  management cluster is up, ArgoCD takes ownership of Sidero's manifests
  going forward (same lifecycle pattern as Cilium + Kubewarden + Istio per
  ADR-0148 / ADR-0379).
- **Bare-metal LAN**: DHCP relay forwards to Sidero's DHCP service; TFTP
  serves the Sidero iPXE chain; iPXE pulls Talos kernel + initramfs.
- **Node enrollment** (D2): factory-fresh hardware PXE-boots → registers as
  Sidero `Server` resource (discovery mode); admin labels it (cell-aware,
  matching ADR-0381 D2 cell patches `oya.cell/foundation|tenant|ci|storage=true`).
- **CAPI provisioning** (D3): `Cluster` + `TalosControlPlane` + `MetalMachine`
  resources declare the desired topology; Sidero provisions Talos onto
  matched Servers.
- **One-command bring-up** (D4): `infra/talos/bare-metal/up.sh` mirrors the
  subcommand shape of `talos-local.sh` (check / bootstrap / enroll / up /
  down / status) so the operator mental model is consistent.

## Consequences

**Positive**:
- Zero-day cold-hardware → green-CP path; no manual per-machine work.
- Cell-aware: bare-metal Servers carry the same `oya.cell/*` labels as
  vfkit VMs (ADR-0381 D2 patches) so workloads schedule identically across
  substrates.
- CAPI-integrated: same `Cluster` / `Machine` API as the production fleet
  per ADR-0375 — local + bare-metal + production are one mental model.
- Sidero Labs are the Talos maintainers; the integration tracks Talos
  releases with no separate compatibility-matrix to maintain.

**Negative**:
- Requires a small management cluster (the meta-K8s that runs Sidero) — a
  few extra resources up front. Mitigated by reusing the existing vfkit
  Talos VM as the bootstrap management cluster (no new substrate).
- DHCP / TFTP / iPXE configuration is sensitive to the bare-metal LAN
  topology — multi-VLAN deployments need a DHCP relay. Single-VLAN dev/lab
  is straightforward; documented in D4's runbook.
- Adds Sidero CRDs + 2 controllers + a CAPI provider — non-trivial install
  footprint on the management cluster (~200 MiB).

## Alternatives Considered

- **Tinkerbell** (CNCF; Equinix Metal-derived). Strong + active. Lost on
  Talos-native integration: it works with any OS, but doesn't ship a
  Talos-aware installer flow out of the box. Sidero is the natural Talos
  fit. Tinkerbell remains the fallback if Sidero proves to be off-track.
- **MAAS** (Canonical). Strong + active, but Ubuntu-centric. Talos
  integration is community-driven, not first-party. Adds a different
  paradigm (MAAS as a separate management stack) that doesn't match our
  CAPI-everywhere convention (ADR-0375).
- **Plain Talos installer + manual `talosctl` per machine** (no
  management plane). Works for 1-2 machines; fails at scale and doesn't
  meet the "zero-day automated" bar. Currently this is what we do in
  `talos-local.sh` for vfkit VMs — but the assumptions there (we can
  pre-build the disk image) don't carry to physical hardware.
- **Plain Talos + Matchbox** (CoreOS-era). Mostly superseded by Sidero in
  the Talos ecosystem; Matchbox is broader (other OSes) but less Talos-
  integrated.
- **Bypass to a hyperscaler-managed bare-metal** (Equinix Metal, AWS
  Outposts, etc.). Fails the hyperscaler-lens (c) — we'd be consuming a
  managed service. Oyatie is itself a cloud provider; we provide bare-metal
  orchestration, we do not consume it.

## Related

- ADR-0375 — Talos + CAPI + Argo CD fleet substrate (this ADR layers Sidero
  underneath CAPI for the bare-metal path).
- ADR-0378 — vfkit + Talos canonical local substrate (the VM-only sibling;
  Sidero is the bare-metal sibling).
- ADR-0381 — multi-node Talos cell topology + the 4 cell-pattern config
  patches (`cell-foundation/tenant/ci/storage.yaml`) — Sidero provisions
  Talos with the SAME patches so workloads schedule identically.
- ADR-0148 — Service mesh: Cilium L3/L4 + Istio Ambient L7 (CNI installed
  on the bare-metal CP just as on vfkit nodes).
- ADR-0083 — Pod runtime tier panic policy (the cell labels map directly to
  the tier scheduler).

## Memory references

- `hyperscaler-lens-architectural-filter` — the standing meta-rule used as
  the choice filter for Sidero Metal (every choice validated against active
  upstream + clean license + self-hostable + hyperscaler-internal-equivalent).
- `vfkit-talos-canonical-local-substrate` — the substrate fact this ADR
  layers a bare-metal sibling onto (vfkit is for the macOS dev box; Sidero
  is for physical hardware).
- `talos-local-stack-state` — the resumable single-node baseline; the
  bare-metal flow uses the same Cilium / Kubewarden / OpenBao / ESO /
  registry / Istio / GitHub / Jenkins layering on top.
