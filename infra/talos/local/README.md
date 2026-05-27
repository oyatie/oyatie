# Local Talos substrate (Apple Silicon, UTM + Apple `vz`)

One-shot, production-fidelity local Kubernetes on this Mac — the laptop mirror of
the bare-metal/cloud fleet in `infra/capi` (ADR-0375). Real Talos VMs (real disks,
NICs, nested virt for Kata), not docker-in-a-trenchcoat.

```bash
infra/talos/local/talos-local.sh check     # read-only preflight (safe anytime)
infra/talos/local/talos-local.sh setup      # brew install UTM + talosctl + kubectl
infra/talos/local/talos-local.sh up          # bring up a 1-node dev cluster (--role single)
infra/talos/local/talos-local.sh status      # VM + node state
infra/talos/local/talos-local.sh down --all  # stop + delete VMs + clear secrets
```

## Roles

| `--role` | What it stands up |
|---|---|
| `single` (default) | one schedulable control-plane node — the dev box |
| `control-plane` | a control-plane node (generates the cluster secrets in `~/.oya/talos-local`) |
| `worker` | a worker that joins the existing control plane (reuses those secrets) |

A 3-CP HA + worker layout = run `up --role control-plane` ×3 (same `CLUSTER`) then
`up --role worker` ×N. Each is its own UTM VM. 128 GB / M-series handles it; small
hosts should use `single`.

## Why UTM + Apple `vz` (not QEMU-HVF, not docker)

Apple's Virtualization.framework exposes **nested virtualization** on M3+/macOS 15+,
which the Kata runtime tier (ADR-0147/0338) requires. Raw QEMU-HVF does not; docker
provisioners give you containers, not VM-fidelity. UTM's `vz` backend is the
production-fidelity local substrate — the same Talos image + `cni:none` + Cilium +
Kata posture the fleet runs.

## CNI

Nodes come up `cni:none` (identical to the fleet's spoke posture), so they stay
`NotReady` until Cilium lands. `up` prints the one-liner:
```bash
helm install cilium cilium/cilium --version 1.19.4 -n kube-system -f infra/talos/cilium-values.yaml
```

## Honest limits

- **UTM has no clean "create VM from ISO" CLI.** This script generates a `.utm`
  bundle (`config.plist`, apple-vz backend) programmatically so bring-up is
  GUI-less. The plist schema is **UTM-version-sensitive**. If `up` fails at the
  VM-create/start step on a future UTM, use the **golden-VM fallback**:
  1. In UTM (GUI, once): Create → **Virtualize** → Linux → attach the Talos arm64
     image → save as `oya-local-golden`.
  2. `utmctl clone oya-local-golden --name oya-local-single` then re-run the
     talosctl legs (`apply-config` / `bootstrap` / `kubeconfig`) — the second half
     of `up` is backend-agnostic.
- **Verified:** `check` is exercised on the dev Mac (M-series, macOS ≥ 15) and
  reports readiness correctly. `setup`/`up`/`down` are syntax-checked + logic-
  reviewed; full VM boot is hardware/UTM-gated and is run by the operator (it
  installs UTM + boots real VMs). Treat the first `up` as the live smoke test.
- State lives in `~/.oya/talos-local/` (`talosconfig`, `kubeconfig`, cluster
  secrets) — **gitignored by location** (outside the repo); treat it like the
  fleet's installation-media `secrets/` (see ADR-0375 PKI custody note).

## Relationship to the fleet

This is the **local cell** of the same managed-Kubernetes substrate Oyatie ships as
its own OKE/GKE/EKS (ADR-0375 Product framing). `infra/capi` provisions remote
cells (OCI/AWS/bare-metal) via Cluster API; this script provisions a local cell via
UTM. Same Talos version, same Kata schematic, same Cilium values — so what works
locally is what runs in the fleet.
