# Kata-containers VM-isolation for oya-ci gate runners — enablement runbook

**Status:** research + author only (no cluster mutation). All sources cited inline with dates.
**Scope:** make `runtimeClassName: kata-cloud-hypervisor` real on the Talos cluster so oya-ci
gate-runner pods (untrusted PR code) run inside per-pod microVMs instead of bare runc.

**Target cluster (per task context):** Talos Linux v1.13.3, single control-plane node, containerd
2.2.4, **arm64**, k8s 1.36.1.

---

## 0. TL;DR / verdict

- **Handler to use:** `kata` (the containerd runtime handler the Talos extension registers).
  Our RuntimeClass is *named* `kata-cloud-hypervisor` (ADR-0147 platform pin) but its `handler:`
  field is `kata`. See `runtimeclass-kata.yaml`.
- **Talos supports Kata via an official system extension** — `siderolabs/kata-containers` — and
  **arm64 is built** (the extension's `pkg.yaml` pulls `kata-static-<ver>-arm64.tar.zst`).
- **The make-or-break is nested virtualization, not the extension.** The Talos Kata extension bundles
  **Cloud Hypervisor (CLH) only**, and CLH needs `/dev/kvm` in the Talos guest. On Apple-Silicon-hosted
  Talos that means **macOS 15+ + M3-or-newer chip + the macOS hypervisor wrapper explicitly enabling
  nested virt**. On M1/M2 or macOS <15, Kata **cannot** run — use the gVisor fallback.
- **arm64 CLH caveat:** recent Cloud Hypervisor requires **GICv3**. Real arm64 servers and
  Apple-Silicon guests provide GICv3, so this is fine; only old GICv2 boards (e.g. Raspberry Pi 4)
  fail. Not a blocker for this cluster.

---

## 1. How Kata runs on Talos (authoritative)

Talos is immutable/API-managed — you cannot `apt install` a runtime. Runtimes ship as **system
extensions** baked into the boot/installer image via the Image Factory. The Kata extension:

- Installs the Kata `containerd-shim-kata-v2`, the **cloud-hypervisor** binary, `virtiofsd`, the guest
  kernel (`vmlinux.container`) and rootfs image (`kata-containers.img`) into the Talos rootfs.
- Drops a containerd CRI config fragment at `/etc/cri/conf.d/10-kata-containers.part` that registers
  exactly one runtime handler, **`kata`**:

  ```toml
  [plugins."io.containerd.cri.v1.runtime".containerd.runtimes.kata]
    runtime_type = "io.containerd.kata.v2"
    privileged_without_host_devices = true
    pod_annotations = ["io.katacontainers.*"]
  [plugins."io.containerd.cri.v1.runtime".containerd.runtimes.kata.options]
    ConfigPath = "/usr/local/share/kata-containers/configuration.toml"
  ```

  Sources (fetched 2026-05-31):
  - 10-kata-containers.part: <https://github.com/siderolabs/extensions/blob/main/container-runtime/kata-containers/10-kata-containers.part>
  - pkg.yaml (arm64 build + copies `cloud-hypervisor`, NOT qemu): <https://github.com/siderolabs/extensions/blob/main/container-runtime/kata-containers/pkg.yaml>
  - extension README (RuntimeClass `handler: kata`): <https://github.com/siderolabs/extensions/blob/main/container-runtime/kata-containers/README.md>

> **Note on a common wrong claim:** some 2026 third-party blogs (e.g. oneuptime, 2026-03-03) state the
> Talos Kata extension "ships QEMU and registers handler `kata`." The handler name is right, but the
> bundled VMM is **Cloud Hypervisor**, verified directly in `pkg.yaml` (it copies
> `./opt/kata/bin/cloud-hypervisor`). Trust the extension source over the blog.

Version pairing: the extension is published per Talos minor as `ghcr.io/siderolabs/extensions:<talos-ver>`
and via Image Factory. Recent Kata versions in the extension: 3.22.0 (extensions v1.11.2), 3.26.0
(extensions v1.13.0-alpha.2). Pin the extension set that matches the running Talos minor (v1.13.x here).
Source: <https://github.com/siderolabs/extensions/releases> (2026-05-31).

---

## 2. Talos machine-config change (install the extension)

Two equivalent paths. **Image Factory is the canonical Talos path** (the standalone
`ghcr.io/siderolabs/installer` is no longer published per release; you must go through Factory —
Talos v1.13 docs, <https://docs.siderolabs.com/talos/v1.13/learn-more/image-factory>).

### Path A — Image Factory schematic (recommended)

1. Define a schematic (already present in this repo at `infra/talos/schematic.yaml`):

   ```yaml
   customization:
     systemExtensions:
       officialExtensions:
         - siderolabs/kata-containers
   ```

2. POST it to get a content-addressed schematic ID:

   ```bash
   curl -sX POST --data-binary @infra/talos/schematic.yaml https://factory.talos.dev/schematics
   # -> {"id":"<schematic-id>"}   (this repo already has one in infra/talos/.schematic-id)
   ```

3. Reference the Factory installer (carrying the extension) as `machine.install.image`, matching the
   running Talos version (v1.13.3 here):

   ```yaml
   machine:
     install:
       image: factory.talos.dev/installer/<schematic-id>:v1.13.3
     kernel:
       modules:
         - name: vhost_net      # Kata/CLH guest networking
         - name: vhost_vsock    # Kata agent vsock transport
   ```

   Apply with `talosctl apply-config` (or `talosctl upgrade --image factory.talos.dev/installer/<id>:v1.13.3`
   so the on-disk install carries the extension, not just a live ISO). This repo's
   `infra/talos/controlplane.patch.yaml` / `worker.patch.yaml` already encode exactly this (currently
   pinned `:v1.13.2`; bump to `:v1.13.3` to match the running node).

### Path B — `.machine.install.extensions` (boot-asset extensions, legacy form)

```yaml
machine:
  install:
    extensions:
      - image: ghcr.io/siderolabs/kata-containers:<kata-ver>-<talos-ver>
```

Path A is preferred on current Talos; B is shown because the task asked for the
`.machine.install.extensions` form. Either way the result is the same containerd `kata` handler.

### Node label (so the RuntimeClass nodeSelector matches)

`runtimeclass-kata.yaml` schedules only onto nodes labelled `katacontainers.io/kata-runtime: "true"`.
On a worker pool, set it in machine config (`machine.nodeLabels`, as in `infra/talos/worker.patch.yaml`).
**On the current single CP-only node**, either label that control-plane node:

```bash
kubectl label node <cp-node> katacontainers.io/kata-runtime=true
```

or drop the `scheduling.nodeSelector` block from `runtimeclass-kata.yaml` for the single-node case.

---

## 3. Verify nested virt / KVM (the gate that decides viability)

Kata-CLH is viable only if `/dev/kvm` exists inside the Talos guest:

```bash
talosctl -n <node-ip> ls /dev/kvm        # present  => nested virt reached the guest (proceed)
                                          # absent   => Kata cannot run; use gVisor fallback
```

(This repo already has `infra/talos/smoke-kata.sh` which does exactly this check + boots a
`kata-cloud-hypervisor` test pod.)

**Apple-Silicon-host reality (sources, 2026-05-31):**
- Apple added nested virtualization to Virtualization.framework only for **M3+ chips on macOS 15+**;
  exposed via `VZGenericPlatformConfiguration.isNestedVirtualizationSupported` /
  `isNestedVirtualizationEnabled`. <https://developer.apple.com/documentation/virtualization/vzgenericplatformconfiguration/isnestedvirtualizationsupported>
  Background: <https://news.ycombinator.com/item?id=40642328>, <https://eclecticlight.co/2024/06/17/how-sequoia-changes-virtualisation-on-apple-silicon/>
- The macOS hypervisor wrapper must opt in: **vfkit** added `--nested-virtualization` / `-n`
  (PR #327, merged 2025-06-25, requires macOS 15 + Apple Silicon, fails fast via
  `vz.IsNestedVirtualizationSupported()`): <https://github.com/crc-org/vfkit/pull/327>.
  **UTM**'s Apple `vz` backend enables it by default on supported hardware (>=4.6).
- If the host is M1/M2 or macOS <15, **`/dev/kvm` will not appear in the guest and Kata is impossible**
  on this substrate regardless of the extension — go to §5.

---

## 4. Wire the controller to use it

The chart already pins the class; only two things are needed.

1. **Helm values** — `oya/ci-controller/iac/k8s/helm/values.yaml` already has:
   ```yaml
   podRuntime:
     runtimeClassName: kata-cloud-hypervisor
   ```
   and the controller Deployment template injects it (`helm/templates/deployment.yaml`:
   `runtimeClassName: {{ .Values.podRuntime.runtimeClassName | quote }}`). No change required for the
   controller pod itself — but see the gate-Job gap below.

2. **Gate-runner Job pods (the untrusted ones) — GAP FOUND.**
   The k8s adapter that builds the gate Job
   (`oya/ci-controller/crates/oya-ci-controller-k8s-adapter/src/lib.rs`, `build_gate_job`, the
   `PodSpec` at ~line 233) sets `restart_policy`, `service_account_name`,
   `automount_service_account_token: false`, security context, and volumes, but **does NOT set
   `runtime_class_name`**. So gate-runner pods currently run on the node default runtime (runc),
   NOT inside a Kata microVM. The chart's `runtimeClassName: kata-cloud-hypervisor` applies only to
   the *controller* Deployment pod, which runs trusted code — exactly backwards from the intent.

   To close the gap (left as a follow-up, since this lane must not modify the crate):
   `PodSpec { runtime_class_name: Some("kata-cloud-hypervisor".to_owned()), .. }` in `build_gate_job`,
   sourced from a config value (mirror `GateRunSpec`/values plumbing) so it can be flipped to
   `gvisor` or `None` per substrate. Track as its own PR against `oya/ci-controller/crates/**`.

---

## 5. Honest viability verdict + fallback

**Is kata-on-Talos-arm64 viable?**
- **On a real arm64 server / bare-metal node with KVM (GICv3): YES.** Official extension, arm64 build,
  `kata` handler, CLH VMM. This is the production answer and the manifest is correct as-is.
- **On the current local Apple-Silicon substrate: CONDITIONAL.** Viable *only* if host is **M3+** on
  **macOS 15+** AND the VM wrapper (vfkit `-n` or UTM-vz) enabled nested virt AND `/dev/kvm` shows up
  in the guest (verify per §3). On M1/M2 or macOS <15 it is **not possible**.
- The task context says the current node is single-CP and the extension "was dropped for the interim
  deploy" — so today the class does not exist and gate pods are unsandboxed.

**Closest viable isolation if KVM is unavailable (recommended fallback ladder):**
1. **gVisor (runsc)** — userspace application kernel, no KVM needed, runs on any arm64 node. Strongest
   no-KVM option. See `runtimeclass-gvisor-fallback.yaml`. Caveat: NOT an official siderolabs
   extension as of 2026-05-31, so it still requires a custom `runsc` system extension or custom image —
   it is not zero-config.
2. **Hardened runc** — user-namespace remap + `RuntimeDefault` seccomp + drop-all-caps + read-only
   rootfs + no-SA-token + restrictive NetworkPolicy. The gate Job already does most of this
   (`automount_service_account_token: false`, `run_as_non_root`, `RuntimeDefault` seccomp, low-priv SA,
   egress NetworkPolicy). This is the weakest of the three (shared host kernel) but needs zero extra
   substrate and is the realistic interim posture for the single-node local cluster until KVM or a
   runsc extension lands.

**Recommendation:** keep `runtimeclass-kata.yaml` as the production target and the value the chart
pins; on the local Apple-Silicon node, first run §3's `/dev/kvm` check — if green, install the
extension (§2) and you are done; if red, do NOT silently fall back to runc — either build/install a
runsc extension and switch `podRuntime.runtimeClassName: gvisor`, or explicitly accept hardened-runc
for local-only and document that gate isolation is degraded outside production.

---

## Sources (all fetched 2026-05-31)

- siderolabs/extensions kata `10-kata-containers.part` (handler + runtime_type): <https://github.com/siderolabs/extensions/blob/main/container-runtime/kata-containers/10-kata-containers.part>
- siderolabs/extensions kata `pkg.yaml` (arm64 build, cloud-hypervisor VMM): <https://github.com/siderolabs/extensions/blob/main/container-runtime/kata-containers/pkg.yaml>
- siderolabs/extensions kata `README.md` (RuntimeClass example, `handler: kata`): <https://github.com/siderolabs/extensions/blob/main/container-runtime/kata-containers/README.md>
- siderolabs/extensions DeepWiki — Container Runtimes (CLH VMM, arm64/amd64): <https://deepwiki.com/siderolabs/extensions/3.2-container-runtimes>
- siderolabs/extensions releases (Kata versions per Talos minor): <https://github.com/siderolabs/extensions/releases>
- Talos v1.13 Image Factory docs: <https://docs.siderolabs.com/talos/v1.13/learn-more/image-factory>
- Talos discussion #11273 — GICv2/GICv3 CLH arm64 error (Pi 4): <https://github.com/siderolabs/talos/discussions/11273>
- Apple `isNestedVirtualizationSupported` (M3+/macOS 15): <https://developer.apple.com/documentation/virtualization/vzgenericplatformconfiguration/isnestedvirtualizationsupported>
- vfkit PR #327 — `--nested-virtualization` flag (merged 2025-06-25): <https://github.com/crc-org/vfkit/pull/327>
- oneuptime "Install Kata on Talos" (2026-03-03) — useful but WRONG on the VMM (says QEMU; it is CLH): <https://oneuptime.com/blog/post/2026-03-03-install-kata-containers-runtime-on-talos-linux/view>
