# Talos/OpenBao pre-wipe preservation-method verification

Date: 2026-08-04

## Verdict

**FAIL / HARD STOP for wiping the machine now.** A secure archive method is feasible, but the current host does not yet have a proven durable decryption recipient, OpenBao recovery material has not been matched and independently restored, CNPG has no backup objects, and iCloud capacity/upload/decryptability from a second device is unproved.

No cluster state, credentials, or iCloud contents were mutated during this verification. No secret values or secret-bearing history lines were read or recorded.

## Direct evidence

| Claim | Evidence | Result |
|---|---|---|
| Source volume is large and sparse | `du -sk` vs `du -skA` | `~/.talos`: 99.29 GiB allocated / 476.83 GiB apparent; `oya-talos`: 97.90 / 450.38 GiB |
| Current macOS archive path can preserve holes | 1 GiB sparse probe through `/usr/bin/bsdtar`, `zstd`, and extraction with `bsdtar -xS` | byte comparison PASS; source and restore both 64 x 512-byte blocks; compressed archive 470 bytes |
| `ditto` is unsuitable for the sparse VM tree | Same probe through `/usr/bin/ditto` | byte comparison PASS but output expanded to 2,097,152 x 512-byte blocks (1 GiB) |
| Recommended authenticated recipient encryption is unavailable | `command -v age rage gpg restic borg` | all absent |
| OpenSSL `enc` is not an acceptable substitute | OpenSSL 3.6.3 `enc -list` | `aes-256-gcm` absent; unauthenticated CBC-style archives are rejected for this backup |
| Existing recipient durability is unproved | SSH/key/hardware/password-manager metadata only | two local Ed25519 private-key candidates; SSH agent has no identities; no recognized hardware token; no 1Password/Bitwarden/KeePassXC app or CLI; no GPG home |
| Time Machine is not a durable second copy | `tmutil destinationinfo`, `tmutil latestbackup` | zero configured destinations; a local latest snapshot is visible, which does not survive wiping the disk |
| iCloud target is only a local staging directory so far | `stat`, `du`, `df` | target exists, mode 0700, empty, same APFS device; `df` proves 3.4 TiB local free space but does not expose iCloud quota or remote-upload completion |
| Current cluster is live and should not be frozen opportunistically | Kubernetes `/readyz` and Pod metadata | ready; 27 Running Pods; one runner-related Pod; OpenBao Running |
| CNPG is not backed up | Backup CR metadata | 0 `Backup` and 0 `ScheduledBackup` objects |
| OpenBao durability needs an offline copy | Pod/PVC metadata plus prior cluster audit | OpenBao uses bound `local-path` PVC; prior audit identified filesystem backend. OpenBao documents offline backup for non-atomic backends |
| Local recovery candidates are exposed and unproved | metadata/count-only history scan | `.bash_history` mode 0644; 12 `unseal`-keyword lines and 2 `root-token`-keyword lines; no values inspected |
| Sensitive Talos configs remain raw | metadata only | `controlplane.yaml` and `worker.yaml` are untracked, mode 0644; `~/.talos/config`, `~/.kube/config`, and `~/.config/talos-mnt/secrets.yaml` exist mode 0600 |
| Required local lifecycle tooling exists | help/version only | `talosctl` v1.13.7 supports `cluster stop/start` and `etcd snapshot`; `kubectl` v1.36.3; `zstd` 1.5.7; `bsdtar` 3.5.3/libarchive 3.7.4 |

Probe artifacts are non-secret and isolated under `.omx/tmp/talos-archive-method-sparse-probe/`.

## Hard blockers

1. **No durable decryption domain is proven.** Encrypting to either local SSH key would make the archive unrecoverable after the same disk is wiped unless that private key is independently recoverable. A public key alone is not proof.
2. **OpenBao recovery is not proven.** The only suspected unseal/root material is in local history, is effectively compromised, and has not been matched to an isolated restore. The current 1-of-1 design is a single point of permanent loss.
3. **Talos disaster recovery is not proven.** A successful etcd snapshot with the matching Talos/Kubernetes secret material and an isolated restore receipt is required. The earlier audit could not authenticate to the Talos API with the discovered local material.
4. **CNPG restoreability is absent.** There are no backup objects or schedules. A filesystem/VM copy is not a substitute for a database recovery test.
5. **OpenBao's filesystem backend cannot be safely copied while live.** OpenBao explicitly recommends offline backup when the backend lacks atomic snapshots.
6. **Remote durability is unproved.** The iCloud path is empty, iCloud quota is unknown, and no fresh-device download plus decrypt test exists.
7. **Rotation cannot precede the only rollback copy.** Rotation first can destroy the only working recovery path; copying raw secrets first violates the stated security boundary. The safe reconciliation is an authenticated, multi-recipient **ciphertext-only quarantine** backup, independently decrypted off-host, followed by rotation and a new canonical archive.

## Required recovery-recipient setup

Use `age` with at least two recipients whose private identities live in independent recovery domains. Multi-recipient age encryption is availability-oriented: either identity can decrypt. It is not threshold cryptography.

Run key generation on the recovery devices, not on the machine being wiped:

```sh
umask 077
age-keygen -o <OFFLINE_IDENTITY_A>
age-keygen -y <OFFLINE_IDENTITY_A> > <PUBLIC_RECIPIENT_A>

umask 077
age-keygen -o <OFFLINE_IDENTITY_B>
age-keygen -y <OFFLINE_IDENTITY_B> > <PUBLIC_RECIPIENT_B>
```

Acceptable independent domains include a hardware-backed/offline key and a separately authenticated password-manager recovery key. iCloud Keychain on this same Apple account/device is not sufficient independence by itself. Copy only the public recipient files onto the source host. Record public-recipient fingerprints and custody locations in the receipt; never record private identities in Git or iCloud as plaintext.

Install and pin the encryption tool before capture, then receipt its binary:

```sh
brew install age
age --version
shasum -a 256 "$(command -v age)"
cat <PUBLIC_RECIPIENT_A> <PUBLIC_RECIPIENT_B> > <PUBLIC_RECIPIENTS_FILE>
chmod 0644 <PUBLIC_RECIPIENTS_FILE>
```

Stop if each recovery identity has not independently decrypted a non-secret canary generated on this host.

## Minimum safe sequence

### 0. Freeze and inventory

- Stop new CI admission and all application writes at a declared cut time.
- Preserve a metadata-only manifest: source paths, byte counts, modes, mtimes, tool versions, cluster versions, Git commit, and public recipient fingerprints.
- Do not print secret-bearing history lines. Recover candidates only through a no-history TTY workflow and treat every candidate as compromised.
- Budget at least 130 GiB for the encrypted sparse VM artifact and additional local scratch for a restore test. Confirm actual iCloud quota separately; local `df` is not cloud quota.

### 1. Capture logical recovery assets before rotation

Talos etcd snapshot, using a Talos config that first proves server authentication:

```sh
umask 077
talosctl --talosconfig <MATCHED_TALOSCONFIG> -n <CONTROL_PLANE_IP> version
talosctl --talosconfig <MATCHED_TALOSCONFIG> -n <CONTROL_PLANE_IP> etcd snapshot <QUARANTINE_ROOT>/talos/etcd.snapshot
cp -p <MATCHED_TALOSCONFIG> <QUARANTINE_ROOT>/talos/talosconfig
cp -p <MATCHED_TALOS_SECRETS_BUNDLE> <QUARANTINE_ROOT>/talos/secrets.yaml
cp -p <CONTROLPLANE_CONFIG> <WORKER_CONFIG> <QUARANTINE_ROOT>/talos/
```

CNPG: configure a supported object-store or volume-snapshot backup and wait for completion. If time forces a last-resort logical export, freeze writes, dump every database/role, and require an isolated restore test; do not call the dump alone a production backup.

```sh
kubectl -n <DB_NAMESPACE> exec <CNPG_PRIMARY_POD> -- \
  pg_dumpall --clean --if-exists --quote-all-identifiers -U <DB_ADMIN_ROLE> \
  > <QUARANTINE_ROOT>/postgres/<CLUSTER_NAME>.sql
```

OpenBao filesystem storage: scale the workload to zero, mount `openbao-data` read-only in a single-purpose offline copy Pod, stream the entire PVC into `<QUARANTINE_ROOT>/openbao/`, delete the copy Pod, then restart. The copy Pod manifest must contain no credentials and must use a digest-pinned image, read-only root filesystem, no service-account token, no network egress, and a read-only PVC mount.

```sh
kubectl -n <OPENBAO_NAMESPACE> scale deployment/<OPENBAO_DEPLOYMENT> --replicas=0
kubectl -n <OPENBAO_NAMESPACE> rollout status deployment/<OPENBAO_DEPLOYMENT> --timeout=<TIMEOUT>
kubectl apply -f <OFFLINE_READONLY_COPY_POD_MANIFEST>
kubectl -n <OPENBAO_NAMESPACE> exec <OFFLINE_COPY_POD> -- \
  bsdtar -cf - -C <READONLY_OPENBAO_MOUNT> . \
  > <QUARANTINE_ROOT>/openbao/storage.tar
kubectl -n <OPENBAO_NAMESPACE> delete pod/<OFFLINE_COPY_POD> --wait=true
kubectl -n <OPENBAO_NAMESPACE> scale deployment/<OPENBAO_DEPLOYMENT> --replicas=1
kubectl -n <OPENBAO_NAMESPACE> rollout status deployment/<OPENBAO_DEPLOYMENT> --timeout=<TIMEOUT>
```

Also capture digest-pinned registry images that cannot be rebuilt from Git, Peach state disposition, live-only manifests, and the exact rebuild manifest. Store all of these only under the mode-0700 quarantine root.

### 2. Create ciphertext-only quarantine

The current `bsdtar`/`zstd` pair passed a sparse round-trip probe. Use `pipefail`, a partial name, multiple public recipients, and an external ciphertext digest:

```sh
set -euo pipefail
umask 077
OUT=<QUARANTINE_ARCHIVE>.tar.zst.age
PARTIAL="${OUT}.partial"
COPYFILE_DISABLE=1 bsdtar --format=pax --numeric-owner -cf - -C <QUARANTINE_ROOT> . \
  | zstd -T0 -19 \
  | age -R <PUBLIC_RECIPIENTS_FILE> -o "$PARTIAL"
mv "$PARTIAL" "$OUT"
shasum -a 256 "$OUT" > "${OUT}.sha256"
```

The public receipt stored beside the ciphertext must contain only: ciphertext hash/size, archive schema version, capture time, public recipient fingerprints, tool versions, source Git commit, backup-object IDs, and test verdicts. Put sensitive paths and manifests inside the encrypted archive.

### 3. Prove decryptability off-host without exposing plaintext

On recovery device A and independently on recovery device B, download the ciphertext and read the stream to EOF. An archive listing alone is insufficient if it terminates early; the full pipeline must return zero:

```sh
shasum -a 256 -c <QUARANTINE_ARCHIVE>.tar.zst.age.sha256
set -o pipefail
age -d -i <OFFLINE_IDENTITY_A_OR_B> <QUARANTINE_ARCHIVE>.tar.zst.age \
  | zstd -d -c \
  | bsdtar -tf - >/dev/null
```

Then restore onto encrypted scratch storage on an isolated host/network and verify:

- Talos etcd snapshot can bootstrap the matching version with the matching secret bundle.
- OpenBao starts from the offline data copy, unseals with the recovered share, and a scoped read-only smoke check succeeds.
- Every CNPG dump/backup restores and passes schema plus row-count/application smoke checks.
- Registry images match recorded digests.
- File-content hashes, logical sizes, modes, and sparse allocation constraints match the encrypted manifest.

Destroy the plaintext scratch volume after the signed restore receipt is preserved. Do not boot a copied VM cluster on the same network as the live cluster.

### 4. Rotate/rebuild after the quarantine restore passes

- **OpenBao:** install the matching `bao` CLI; rekey away from 1-of-1 to at least 2-of-3, encrypt each share directly to a distinct custodian public key, verify the rekey nonce, establish scoped admin authentication, revoke the exposed root token, and rotate every downstream credential issued by or stored in OpenBao.

```sh
bao operator rekey -init -key-shares=3 -key-threshold=2 -verify \
  -pgp-keys=<PGP_PUBLIC_KEY_A>,<PGP_PUBLIC_KEY_B>,<PGP_PUBLIC_KEY_C>
bao operator rekey -nonce=<REKEY_NONCE>
bao operator rekey -verify -nonce=<VERIFICATION_NONCE>
```

Commands that accept unseal shares should prompt on the TTY; never place a share/token in argv, a shell pipeline, a transcript, or an environment retained by an agent. Use generated root tokens only for bootstrap/recovery and revoke them after scoped administration works.

- **Talos:** because the cluster is greenfield and the matching current recovery chain is unproved, prefer a clean replacement cluster generated from fresh Talos secrets and declarative Git state over risky in-place CA/etcd key surgery. Restore only explicitly retained application state. If exact-state recovery is required instead, plan and test Talos/Kubernetes credential rotation separately before claiming completion.
- Rotate GitHub App/ARC, registry, CNPG, Kubernetes, kubeconfig, Talos API, and any history-exposed credentials. Verify old credentials fail and new credentials work.
- Lock or securely eliminate raw `controlplane.yaml`, `worker.yaml`, shell-history candidates, and superseded configs only after the new restore proof exists.

### 5. Create and prove the canonical post-rotation archive

Repeat steps 1-3 using only rotated material. This post-rotation artifact is the canonical archive.

For an optional belt-and-suspenders full VM image, first stop the local cluster cleanly, confirm the VM processes are gone, archive the sparse tree, and start it only after capture:

```sh
talosctl cluster stop --name oya-talos --state <TALOS_CLUSTER_STATE_ROOT>
COPYFILE_DISABLE=1 bsdtar --format=pax --numeric-owner -cf - -C <TALOS_CLUSTER_STATE_ROOT> oya-talos \
  | zstd -T0 -19 \
  | age -R <PUBLIC_RECIPIENTS_FILE> -o <OFFLINE_VM_ARCHIVE>.tar.zst.age
talosctl cluster start --name oya-talos --state <TALOS_CLUSTER_STATE_ROOT>
```

The VM image is not a substitute for logical Talos/OpenBao/CNPG restore evidence. A live copy of the disk files is inconsistent and must not be treated as a backup.

### 6. Final iCloud handoff and fresh-session proof

Copy only the canonical ciphertext, its SHA-256 file, and the sanitized public receipt to:

```text
/Users/jasonlee/Library/Mobile Documents/com~apple~CloudDocs/talos/
```

Do not copy raw identities, tokens, unseal shares, Talos secrets, kubeconfigs, database dumps, shell history, or plaintext manifests there. Confirm upload completion and quota through the iCloud provider UI, then perform a fresh-device/fresh-session download, hash verification, full-stream decryption test, and isolated restore. The machine is wipe-safe only after that off-host receipt is signed and independently stored.

Retain the pre-rotation quarantine artifact only for a defined rollback window; delete it from every replica after the post-rotation restore proof and rollback window complete.

## Acceptance gate for disk wipe

All boxes must be true:

- [ ] Two independent recovery identities decrypt a canary and the canonical archive off-host.
- [ ] Talos etcd snapshot plus matching secrets restore successfully in isolation.
- [ ] OpenBao offline data plus rotated 2-of-3 recovery material restore and unseal successfully.
- [ ] All CNPG clusters restore and pass application smoke checks.
- [ ] Required registry/live-only artifacts have digest/rebuild proof.
- [ ] All exposed credentials are rotated; old credentials are proven invalid.
- [ ] Canonical post-rotation ciphertext is fully uploaded to iCloud and downloaded on a different device.
- [ ] Ciphertext hash, full-stream authenticated decryption, archive listing, and isolated restore all pass.
- [ ] Public receipt and successor-session handoff are remotely preserved outside this disk.
- [ ] Pre-rotation quarantine retention/deletion decision is recorded.

Until every required box is satisfied, **do not wipe**.

## Primary references

- Talos CLI (`etcd snapshot`, secrets generation): https://www.talos.dev/latest/reference/cli/
- Talos control-plane backup recommendation: https://www.talos.dev/v1.7/learn-more/control-plane/
- OpenBao backup model and offline requirement for non-atomic storage: https://openbao.org/docs/concepts/storage/
- OpenBao filesystem-backend limitations: https://openbao.org/docs/2.5.x/configuration/storage/filesystem/
- OpenBao rekey and PGP-encrypted shares: https://openbao.org/docs/commands/operator/rekey/
- CloudNativePG backup/recovery: https://cloudnative-pg.io/documentation/1.26/appendixes/backup_barmanobjectstore/
- age multiple recipients: https://github.com/FiloSottile/age#multiple-recipients
- GNU sparse archive semantics: https://www.gnu.org/software/tar/manual/html_node/sparse.html
