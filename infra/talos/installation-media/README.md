# Talos USB installation-media generator

Builds bootable Talos installation media for the fleet's bare-metal nodes:

- **`gen-media.sh control-plane`** — config BAKED into the ISO (offline). Boot a
  control-plane machine → auto-install → the CAPI management cluster forms itself.
  Requires `CONTROLPLANE_ENDPOINT`.
- **`gen-media.sh node`** — GENERIC image (Kata-baked, no secrets). Boots with
  `talos.config=$CONFIG_URL`, but production worker zero-touch is **not** claimed
  until a hub config-serving plane maps attested MAC/UUID identity to a generated
  Talos machineconfig. Without that plane, node media is experimental for more
  than hand-assigned nodes. One image can still serve ALL bare-metal nodes once
  the assignment registry and serving contract are implemented.
- **`gen-media.sh verify-backup` / `shred` / `rotate-pki`** — local custody
  guardrails for the control-plane bootstrap PKI and ISO artifacts. They do not
  contact a fleet, mutate hardware, or print secret values.

Cloud spokes (OCI/AWS) are NOT built here — CAPI provisions them with platform images.

## Inputs / outputs

- Output ISOs → `_out/` (gitignored).
- Cluster secrets → `secrets/` (gitignored; **never commit; never copy off the
  build host without an encryption envelope**).
- Sealed-backup receipts → keep under `secrets/` or another ignored/off-host
  operator evidence store. Receipts must contain exactly the metadata allowlist
  (`created_at`, `sealed_backup_uri`, and `secrets_sha256`) and must not contain
  secret values.

## Node config-serving plane contract (specified, not implemented)

`gen-media.sh node` only proves the generic media shape: Kata is baked into the
image and the Talos kernel argument points at `CONFIG_URL`. The image does **not**
by itself prove fleet production zero-touch. Before that claim is allowed,
`CONFIG_URL` must terminate on either an adopted Talos/Sidero metadata server or
a small Oyatie-owned config service with this minimum contract:

1. **Assignment identity** — each request must bind to an approved machine row by
   attested network-interface MAC and/or SMBIOS system UUID. Trusted bindings can
   come from BMC/DCIM inventory, switch-port/DHCP lease inventory, or an adopted
   Talos/Sidero metadata identity. A self-asserted `mac=` or `uuid=` query value
   is not sufficient by itself for production config release.
2. **Assignment registry** — registry rows must include `assignment_id`,
   `cell_id`, `cluster_id`, `node_role`, MAC/UUID identity, machineconfig version,
   `machineconfig_sha256`, custody secret ref, status, validity window,
   `revoked_at`, approver, and audit-chain event id.
3. **Machineconfig custody** — generated Talos machineconfigs live only in an
   approved secret/custody system such as OpenBao or a sealed off-host envelope.
   Git stores refs and digests only; config reads are audited without logging
   payloads.
4. **Authn/authz** — the serving endpoint must be private/protected, assignment
   writes require ops-platform approval and policy/audit evidence, and reads must
   match an active assignment before any secret-bearing body is returned.
5. **Rotation/revocation** — revoked identities stop serving immediately; config
   rotation creates a new version/digest; lost or reassigned hardware triggers
   assignment revocation plus Talos credential/certificate rotation as applicable.
6. **Bootstrap failure behavior** — unknown, duplicate, expired, or revoked
   identities fail closed (`403`/`404`) with no machineconfig payload. Remediation
   is an assignment-registry or generated-config fix followed by normal
   Talos/CAPI retry, not manual SSH troubleshooting.

The machine-readable version of this contract lives in
`specs/deployment-ops-contract.json#node_config_serving_plane`. This repository
does not currently ship a live config-serving endpoint or real assignment
registry.

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

1. **Generate on a trusted host** (offline if possible, and preferably on an
   encrypted scratch volume). Treat the build host like an HSM operator
   workstation.
2. **`secrets/` is gitignored** but the responsibility for its safety is yours.
   If you lose `secrets/secrets.yaml`, the script will regenerate from scratch
   on next run — and the PKI will be NEW, which means the cluster you previously
   built is now orphaned. Keep a sealed backup (e.g. OpenBao Transit,
   age-encrypted off-host) before treating any cluster as authoritative.
3. **Verify a sealed backup before re-generating control-plane media.** When
   `secrets/secrets.yaml` already exists, `gen-media.sh control-plane` refuses to
   continue unless `gen-media.sh verify-backup` can verify an operator receipt.
   Create the sealed backup outside this repo, then write a receipt with exactly
   these metadata keys:

   ```text
   created_at=2026-07-01T00:00:00Z
   sealed_backup_uri=<OpenBao transit path, age envelope id, or offline escrow id>
   secrets_sha256=<sha256 of secrets/secrets.yaml>
   ```

   Verification command:

   ```bash
   ./gen-media.sh verify-backup --receipt secrets/secrets.yaml.backup.receipt
   CONTROLPLANE_ENDPOINT=https://<control-plane-ip-or-vip>:6443 \
     ./gen-media.sh control-plane --backup-receipt secrets/secrets.yaml.backup.receipt
   ```

   The script prints receipt paths and guard status only; it does not print the
   digest or any secret value.
4. **Shred bootstrap ISO artifacts after the control plane is up.** Once the
   cluster is reachable through the normal `kubectl` + `talosctl` paths, remove
   `_out/*.iso` with an explicit operator acknowledgement:

   ```bash
   ./gen-media.sh shred --confirm-control-plane-up --all
   # or dry-run a specific artifact first:
   ./gen-media.sh shred --dry-run --confirm-control-plane-up control-plane-metal-amd64.iso
   ```

   The command only accepts regular `*.iso` files under `_out/` and refuses
   symlinks or arbitrary paths. On APFS/SSD media, overwrite/unlink primitives
   are not a cryptographic recovery guarantee by themselves; for high-trust
   cells, build on encrypted scratch media and crypto-erase the volume key or
   physically destroy removable media.
5. **Rotate after a known compromise.** Rotation is destructive re-bootstrap:
   the old PKI cannot be reused. Use the bounded dry-run runbook to record the
   exact operator intent without mutating files or printing secrets:

   ```bash
   ./gen-media.sh rotate-pki --dry-run \
     --ack-compromise \
     --confirm-destructive-rebootstrap
   ```

   Then regenerate secrets/media, seal and verify a fresh backup, re-bootstrap
   the management cluster, and let CAPI + Argo CD reconcile nodes/apps through
   the zero-touch path. Do not add manual SSH troubleshooting.
6. **Post-bootstrap PKI custody belongs to OpenBao** (per the app-of-apps Argo CD
   chart). The bake-in is bootstrap-only; long-term certificate issuance for
   workloads flows through OpenBao PKI + cert-manager once the cell is up.

## Safe local verification

The custody guardrails can be exercised without generating production media,
contacting Image Factory, or reading real secrets:

```bash
./gen-media.sh self-test
bash -n ./gen-media.sh
CONFIG_URL=https://join.oyatie.dev/config ./gen-media.sh node --dry-run
python3 -m json.tool ../../../specs/deployment-ops-contract.json >/dev/null
```

The self-test creates a temporary fixture secret and fake ISO under `/tmp`, then
checks backup-receipt verification, control-plane dry-run gating, shred path
guardrails, and rotate-pki acknowledgement parsing. The node dry-run proves the
generic media command shape without contacting Image Factory or a live config
endpoint; the JSON check validates the machine-readable config-serving contract.

## Non-claim boundaries

- These commands do **not** prove a real fleet bootstrap happened.
- These commands do **not** move PKI off-host; the operator must create the
  sealed backup in an approved custody system first.
- `gen-media.sh node` does **not** prove production worker zero-touch until the
  node config-serving plane and MAC/UUID assignment registry have evidence.
- These commands do **not** mutate Cloudflare, CAPI clusters, Argo CD, hardware,
  or Kubernetes resources.
- These commands do **not** authorize manual SSH troubleshooting.

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
