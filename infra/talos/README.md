> **SUPERSEDED (2026-05-27) by ADR-0375.** Cluster bring-up is now **Talos
> installation-media (USB zero-touch) + Cluster API + per-cell Argo CD**:
> `infra/talos/installation-media/` (the `gen-media.sh` ISO generator + README),
> `infra/capi/` (clusterctl pins + init + ClusterResourceSet), and
> `infra/capi/clusters/` (the parameterized spoke-cell Helm chart). There is NO
> OpenTofu/libvirt Talos module — the earlier `infra/talos/tofu/` /
> `microservices/cloud-iac/tofu/modules/talos-cluster/` direction was dropped before it
> landed; OpenTofu owns only the Cloudflare edge per ADR-0375. The `*.patch.yaml`,
> `cilium-values.yaml`, `kata-runtimeclass.yaml`, `schematic.yaml`, and `smoke-kata.sh`
> here remain the canonical Kata-worker-pool + Cilium-values references the new path
> consumes. The Apple-Silicon UTM prose below is historical.

# Talos local production-fidelity substrate (Apple Silicon)

The Oyatie local cluster runs **Talos Linux** (immutable, API-managed) multi-node on this Mac via
**UTM** (Apple `vz` backend). This is the production-fidelity substrate that replaces single-node
colima+k3s — chosen because our own platform invariants *require* real multi-node + nested virt:

- **ADR-0147** pins `runtimeClassName: kata-cloud-hypervisor` across 10+ microservices. Kata
  cloud-hypervisor needs nested virtualization → impossible on container-node clusters (kind/k3d) and
  on single-node. Apple exposes nested virt on M3+/macOS 15+ via Virtualization.framework; UTM's `vz`
  backend enables it by default (≥4.6). Raw QEMU (HVF) does **not** expose it — hence UTM-vz, not QEMU.
- **ADR-0165** nightly node-failure/partition chaos drills + 3-replica anti-affinity → need ≥3 nodes.

Full research + rationale: [`.omx/plans/talos-on-apple-silicon-procedure.md`](../../.omx/plans/talos-on-apple-silicon-procedure.md).

## Topology

| Node | Role | vCPU | RAM | Disk | IP (default, confirm subnet) |
|---|---|---|---|---|---|
| cp-1/2/3 | control-plane (HA etcd) | 4 | 8 GB | 60 GB | 192.168.64.11/.12/.13 |
| w-1/2 | worker (Kata-capable) | 6 | 24 GB | 120 GB | 192.168.64.21/.22 |
| **VIP** | kube-apiserver HA endpoint | — | — | — | 192.168.64.10 |

72 GB RAM total → **`colima stop` first** (it reserves 96 GB; 96+72+macOS > 128). See procedure §6.

## Files (the IaC)

| File | Purpose |
|---|---|
| `schematic.yaml` | Image Factory schematic — bakes the Kata/CLH system extension. Already POSTed; ID in `.schematic-id`. |
| `controlplane.patch.yaml` / `worker.patch.yaml` | Machine-config patches (installer ref, Kata kernel modules, VIP, cni=none). |
| `cilium-values.yaml` | Cilium CNI Helm values (Talos kube-proxy replacement via KubePrism). |
| `kata-runtimeclass.yaml` | Aliases `kata-cloud-hypervisor` → the extension's CLH-backed `kata` handler. |
| `bootstrap.sh` | Full bring-up: gen-config → apply → bootstrap → kubeconfig → Cilium → Kata. |
| `smoke-kata.sh` | Validates `/dev/kvm` + a `kata-cloud-hypervisor` pod (the make-or-break check). |

Prereqs (already installed): UTM (`/Applications/UTM.app`, `utmctl`), `talosctl` v1.13.2.
ISO (Kata-baked): `~/talos-mac/talos-kata-arm64.iso`.

---

## Phase A — DE-RISK FIRST (one VM, prove nested virt)

Before building 5 VMs, prove the make-or-break assumption empirically on **one** VM.

**You (UTM GUI, ~2 min):**
1. Open UTM → **Create a New Virtual Machine → Virtualize** (NOT Emulate) → **Linux**.
   Ensure **"Use Apple Virtualization"** is **checked** (this is the `vz` backend with default nested virt).
2. **Boot ISO image** → browse to `~/talos-mac/talos-kata-arm64.iso`.
3. RAM **24 GB**, CPU **6**; storage **120 GB**.
4. Network: open **Settings → Network → Network Mode = Shared Network** (simplest; gives 192.168.64.x).
   *(Bridged also works but uses your LAN subnet — then update the IPs in the patches/bootstrap.)*
5. Save, **Start** the VM. Talos boots to maintenance mode. Note the IP it prints / shows in UTM.

**Then tell me the IP** — I run a minimal `apply-config` + `talosctl ls /dev/kvm`. If `/dev/kvm` is
present, nested virt is confirmed and we proceed to Phase B. (This single check de-risks the entire
substrate decision before any further VM work.)

---

## Phase B — Full cluster (3 CP + 2 worker)

Once Phase A is green:
1. **Clone the golden VM** via CLI: `utmctl clone <golden> --name talos-cp-1` … (×3 CP-spec, ×2 worker
   at 6/24). Or create from the ISO. Start all 5; note each node's IP.
2. Confirm the subnet, then (adjusting IPs if not the defaults):
   ```bash
   export CP_IPS="<cp1> <cp2> <cp3>" WORKER_IPS="<w1> <w2>" VIP="<vip>"
   bash infra/talos/bootstrap.sh        # gen-config → apply → bootstrap → kubeconfig → Cilium → Kata
   bash infra/talos/smoke-kata.sh       # confirm /dev/kvm + a kata-cloud-hypervisor pod Runs
   ```
3. Acceptance gates (production-fidelity proof): etcd survives 1 CP kill; a `kata-cloud-hypervisor`
   pod boots; a chaos node-drain runs; a 3-replica Deployment spreads across nodes.

---

## Follow-ups (tracked separately)
- Record the substrate decision as an ADR (supersedes the colima+k3s toolchain note).
- Migrate the canonical substrate (GitHub/OpenBao/ArgoCD/Rollouts/observability/Valkey) onto
  Talos via an ArgoCD **app-of-apps** (Cilium = sync-wave 0, then platform). GitOps, no hand-rolling.
- Move Talos secrets (`~/talos-mac/talosconfig`, machine secrets) into **OpenBao** / sops — not git.
- Optional 3rd worker for strict 3-replica anti-affinity (vs `maxSkew:1`/`ScheduleAnyway` on 2).
- For zero-touch VM creation: Parallels 26 (`prlctl create/set`) instead of UTM's GUI golden+clone.
