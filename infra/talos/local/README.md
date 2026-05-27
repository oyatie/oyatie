# Local Talos substrate (Apple Silicon, vfkit + Apple `vz`)

One-shot, production-fidelity local Kubernetes on this Mac — the laptop mirror of
the bare-metal/cloud fleet in `infra/capi` (ADR-0375). Real Talos VMs (real disks,
NICs, nested virt for Kata), not docker-in-a-trenchcoat.

```bash
infra/talos/local/talos-local.sh check     # read-only preflight (safe anytime)
infra/talos/local/talos-local.sh setup      # brew install vfkit + talosctl + kubectl
infra/talos/local/talos-local.sh up          # bring up a 1-node dev cluster (--role single)
infra/talos/local/talos-local.sh status      # vfkit pids + node state
infra/talos/local/talos-local.sh down --all  # stop + delete VMs + clear secrets
```

## Roles

| `--role` | What it stands up |
|---|---|
| `single` (default) | one schedulable control-plane node — the dev box |
| `control-plane` | a control-plane node (generates the cluster secrets in `~/.oya/talos-local`) |
| `worker` | a worker that joins the existing control plane (reuses those secrets) |

A 3-CP HA + worker layout = run `up --role control-plane` ×3 (same `CLUSTER`) then
`up --role worker` ×N. Each is its own vfkit VM (use a distinct `--name` per node).
128 GB / M-series handles it; small hosts should use `single`.

## Why vfkit + Apple `vz` (not UTM, not QEMU-HVF, not docker)

Apple's Virtualization.framework exposes **nested virtualization** on M3+/macOS 15+,
which the Kata runtime tier (ADR-0147/0338) requires. Raw QEMU-HVF does not; docker
provisioners give you containers, not VM-fidelity.

[vfkit](https://github.com/crc-org/vfkit) is the single-command, **headless** CLI
front-end to that same `vz` backend — the same Talos image + `cni:none` + Cilium +
Kata posture the fleet runs, but scriptable. We use vfkit (not UTM) because UTM's
`utmctl` can only drive VMs created through the UTM GUI: a hand-authored `.utm`
bundle is never imported headless, so UTM cannot do GUI-less bring-up. vfkit boots
the arm64 **metal raw** image directly with one command and no GUI.

## VM bring-up + IP discovery (how `up` works)

`up` is mechanism + Talos legs:

1. **Disk** — copy the cached `~/.oya/talos-local/talos-<ver>-arm64.raw` to a
   per-VM `<name>.img` and grow it to `--disk-gb` (BSD `dd` seek; Talos resizes
   its partition on first boot). The cached raw is reused — never re-downloaded.
2. **MAC** — assign a deterministic, locally-administered unicast MAC derived from
   the node name (`52:54:00:<hash>`), so the same name always gets the same MAC.
3. **Boot** — launch vfkit headless in the background with EFI bootloader (creates
   the variable store), the boot disk, a NAT NIC pinned to that MAC, an RNG source,
   and the serial console piped to `~/.oya/talos-local/<name>.log`. The pid is
   written to `<name>.pid`.
4. **IP** — vfkit NAT hands the guest a DHCP lease via macOS `bootpd`. We poll
   `/var/db/dhcpd_leases` for the node's MAC until the lease appears (timeout
   ~5 min). bootpd writes that file log-structured, hardware-type-prefixed (`1,`),
   and with per-octet **leading zeros stripped** (`0c`→`c`); the lookup normalizes
   both sides and takes the **last** lease block for the MAC.
5. **Talos** — the usual `talosctl apply-config` → (cp/single) `bootstrap` →
   `kubeconfig`/`talosconfig` legs against the discovered IP. Backend-agnostic.

`down` kills the vfkit pid(s) and removes the per-VM disk, EFI store, pid, and log.
`status` lists tracked vfkit pids (running/stopped + IP) and `kubectl get nodes`.

## CNI

Nodes come up `cni:none` (identical to the fleet's spoke posture), so they stay
`NotReady` until Cilium lands. `up` prints the one-liner:
```bash
helm install cilium cilium/cilium --version 1.19.4 -n kube-system -f infra/talos/cilium-values.yaml
```

## Honest limits

- **vfkit NAT has no API to report the guest IP.** We pin a fixed MAC and read the
  macOS DHCP lease (`/var/db/dhcpd_leases`). That file is created by `bootpd` only
  once it hands out the first NAT lease, is log-structured (last block wins),
  type-prefixed (`1,`), and strips per-octet leading zeros — the lookup accounts
  for all three. If the host's lease file is root-only on your macOS build, `up`
  may need `sudo` to read the VM IP; `check` flags that case.
- **If NAT lease discovery is unreliable on a given host**, install `socket_vmnet`
  (`brew install socket_vmnet`) for a bridged interface and hand its socket to
  vfkit (`--device virtio-net,unixSocketPath=…`), then drive the same Talos legs.
  Plain NAT + lease lookup is preferred and is what `up` uses by default.
- **Verified:** `check` is exercised on the dev Mac (M-series, macOS ≥ 15) and
  reports readiness correctly; the MAC-normalization + lease-parse IP discovery is
  unit-tested against the real bootpd lease format. `setup`/`up`/`down` are
  syntax-checked (`bash -n`) + logic-reviewed; the full VM boot is hardware/vfkit-
  gated and is run by the operator (it `brew install`s vfkit + boots real VMs).
  Treat the first `up` as the live smoke test.
- State lives in `~/.oya/talos-local/` (`talosconfig`, `kubeconfig`, cluster
  secrets, per-VM disks/EFI stores/pids/logs) — **gitignored by location**
  (outside the repo); treat it like the fleet's installation-media `secrets/`
  (see ADR-0375 PKI custody note).

## Relationship to the fleet

This is the **local cell** of the same managed-Kubernetes substrate Oyatie ships as
its own OKE/GKE/EKS (ADR-0375 Product framing). `infra/capi` provisions remote
cells (OCI/AWS/bare-metal) via Cluster API; this script provisions a local cell via
vfkit. Same Talos version, same Kata schematic, same Cilium values — so what works
locally is what runs in the fleet.
