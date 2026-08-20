# Talos USB installation-media generator

Builds bootable Talos installation media for the fleet's bare-metal nodes:

- **`gen-media.sh control-plane`** — config BAKED into the ISO (offline). Boot a
  control-plane machine → auto-install → the CAPI management cluster forms itself.
  Requires `HUB_ENDPOINT`.
- **`gen-media.sh node`** — GENERIC image (Kata-baked, no secrets). Boots with
  `talos.config=$CONFIG_URL` and fetches its role/cluster config from the hub. One
  image for ALL bare-metal nodes.

Cloud spokes (OCI/AWS) are NOT built here — CAPI provisions them with platform images.

## Inputs / outputs

- Output ISOs → `_out/` (gitignored).
- Cluster secrets → `secrets/` (gitignored; **never commit; never copy off the
  build host without an encryption envelope**).

## Cluster PKI custody (LOAD-BEARING — read before generating)

The `control-plane` preset BAKES the following into the ISO:

- cluster root CA + key
- etcd CA + key
- kubelet bootstrap token
- apiserver client certs
- service-account signing key

This is the cluster's **root of trust**. The trust boundary is the ISO file +
`secrets/secrets.yaml`. Loss or theft of either = full cluster compromise.
Discipline:

1. **Generate on a trusted host** (offline if possible). Treat the build host
   like an HSM operator workstation.
2. **`secrets/` is gitignored** but the responsibility for its safety is yours.
   If you lose `secrets/secrets.yaml`, the script will regenerate from scratch
   on next run — and the PKI will be NEW, which means the cluster you previously
   built is now orphaned. Keep a sealed backup (e.g. OpenBao Transit, age-encrypted
   off-host) before treating any cluster as authoritative.
3. **Shred the USB after the control plane is up.** The script does not (yet)
   wipe `_out/` automatically; once the cluster is reachable via `kubectl` +
   `talosctl`, the bootstrap ISO must be physically destroyed or cryptographically
   shredded. Tracked as a follow-up in
   `registry/placeholder-debt/adr-follow-ups.yaml#adr-0375-installation-media-shred-action`.
4. **Rotate after a known compromise.** There is no in-script rotation today —
   rotation = regenerate secrets, rebuild + reflash the ISO, re-bootstrap the
   cluster (the old PKI cannot be reused). Tracked as follow-up in the same
   placeholder-debt entry.
5. **Post-bootstrap PKI custody belongs to OpenBao** (per the app-of-apps Argo CD
   chart). The bake-in is bootstrap-only; long-term certificate issuance for
   workloads flows through OpenBao PKI + cert-manager once the cell is up.

## Distribution

USB media must reach the control-plane machines through a hand-carry or
physically-trusted channel (USB-via-mail is acceptable for low-trust cells; for
high-trust cells use sealed tamper-evident envelopes). Do NOT email or
network-transfer the ISO — every minute it lives on a multi-tenant filesystem is
a key-custody risk.

## See also

- ADR-0375 — Talos + Cluster API + Argo CD fleet substrate
- ADR-0241 — DR-pair design (custody of the second cluster's PKI)
- ADR-0306 — disaster-mode (per-cell Argo CD enables cell-local survival)
- `infra/talos/cilium-values.yaml` — the L4 dataplane values the CRS ships
