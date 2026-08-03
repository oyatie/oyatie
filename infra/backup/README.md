# Off-node backup — design, cost, and the approval gate

ADR-0197 is Accepted and unimplemented. This directory is a **design plus unapplied
manifests**. Nothing here has touched a cluster. The lab cell is shared with the
console project and no go has been given, so `velero-offnode.k8s.yaml` is fail-closed
three ways (paused schedules, a credential Secret that does not exist, and Velero not
being installed at all).

## 1. What is actually at risk

The backlog names `nativelink-cas`, `seaweedfs`, and `registry` as the urgent set.
They are the wrong set. All three hold derived state:

| PVC | Holds | Node loss costs |
|---|---|---|
| `oya-ci/nativelink-cas-data` (50Gi) | content-addressed build cache (ADR-0560) | one cold build |
| `oya-storage/seaweedfs-data` (50Gi) | BuildKit cache, sccache, artifact overlay | one cold build |
| `oya-registry/registry-data` (20Gi) | images BuildKit rebuilds from this repo | one rebuild |

The set that is genuinely irreplaceable is not in that list, and is easy to miss
because neither PVC declares a `storageClassName` at all — both silently inherit the
cell default, which here is the same node-local `local-path`:

| PVC | Holds | Node loss costs |
|---|---|---|
| `oya-kms/openbao-data` (2Gi) | every secret the platform holds | the platform |
| `oya-forge/github-data` (5Gi) | forge users, webhooks, PR records | forge history |

Worse, `infra/kms/openbao.k8s.yaml` L7–8 puts the OpenBao unseal keys in a k8s Secret
in etcd — on the same node as the barrier-encrypted data they unlock. Node loss takes
both halves simultaneously. **That is the actual cliff**, and no amount of backing up
build caches moves it.

## 2. The design defect in ADR-0197 D-2

D-2 routes all three prongs to SeaweedFS. On this cell SeaweedFS is
`oya-storage/seaweedfs-data`, `storageClassName: local-path` — the same node as every
source. Implementing ADR-0197 exactly as written would produce green backups that
vanish with the thing they protect.

The amendment is one clause: **the backup storage location must be off the cell.**
Everything else in ADR-0197 (Velero for K8s state, the D-5 retention ladder, the D-6
drill cadence) stands unchanged. pgBackRest and Restic are not in scope for this cell:
there is no in-cluster Postgres and no non-K8s host state.

## 3. Mechanism fit

Velero, reused as-is. It is already the Accepted choice, the chart already exists at
`cloud/cloud-iac/iac/helm/velero/`, and the only change needed is the storage target.
No new crate, no new image, no new gate. Writing an owned-Rust backup agent for 7Gi on
one lab cell would be new code where a values change does the job — the owned-Rust
posture is satisfied here by *not* adding a bespoke component, and by keeping the
`oya-shared-backup-kernel::BackupExecutor` seam (ADR-0197's inviolate seam) untouched.

Target: OCI Object Storage over its S3-compatibility API. The tenancy is provisioned,
Always Free covers it, and `velero-plugin-for-aws` speaks to it unchanged. Cloudflare
R2 is a one-line swap (`s3Url` + `region`) if residency argues for it;
`infra/cloudflare/` already carries the account tofu.

Two things that are not optional and are easy to get wrong:

- `checksumAlgorithm: ""` on the BackupStorageLocation. Non-AWS S3 implementations
  reject the AWS SDK's default CRC32 trailer; without it Velero reports success and
  writes unreadable objects. Note this applies to the existing SeaweedFS BSL in
  `cloud/cloud-iac/iac/helm/velero/values.yaml` too, which does not set it.
- The bucket credential **cannot** be projected from OpenBao by External Secrets.
  OpenBao is the thing being backed up; sourcing the backup credential from it means
  losing OpenBao loses the ability to reach the backup. It must be a hand-created
  Secret with the material custodied off-machine.

ADR-0197 D-3 mandates age encryption before write. There is no age sidecar in the
chart — that part of D-3 is unimplemented. For the OpenBao prong this design removes
the need for it: the `file` backend is already barrier-encrypted, so excluding Secrets
from the backup leaves only ciphertext in the bucket and the barrier *is* the at-rest
encryption. For the forge prong there is no such property, so bucket-level SSE is a
hard precondition.

### Known limit, stated plainly

fs-backup of a live `file`-backend OpenBao is a best-effort copy, not a consistent
snapshot — the backend has no snapshot API and a mid-write backup can tear across
entries. ADR-0197 rejects this exact shape for Postgres in Alternative (a); the same
objection applies. The consistent answer is the raft migration already mandated by
F-SEC-OPENBAO-FILE-BACKEND (the `file` backend is removed at OpenBao 2.7.0), after
which `bao operator raft snapshot save` is atomic. The schedule here closes the
total-loss cliff in the interim; it is not the destination.

## 4. Cost

| Line | Cost |
|---|---|
| OCI Object Storage, ~7Gi claimed / far less used | $0 — Always Free covers 20 GB |
| Egress for restore | $0 — 10 TB/mo free outbound |
| OCI Vault key for bucket SSE | ~$1/mo — already an accepted line item per `infra/TOPOLOGY-2026-05-16.md` |
| Velero controller | ~256Mi RAM on the shared cell |
| Velero node-agent DaemonSet | ~512Mi RAM request per node on the shared cell |

The last two are the real cost and they are paid by a **shared** cluster. API request
volume is the one number to watch, not assert: Always Free allows 50,000 requests/mo
and an hourly kopia incremental should sit far under that, but it has not been
measured and the first month should be checked against the OCI usage report.

## 5. What a human must approve before anything is applied

1. **Distribute the OpenBao Shamir shares off this machine.** Restore under this design
   requires them and they currently live only in etcd on the at-risk node. Shamir
   exists to be distributed; this is the single highest-value action here and it needs
   no cluster change at all.
2. **Off-node target and residency.** Data leaves the laptop. Confirm OCI (vs R2) and
   confirm the region against the sovereign-pack posture in ADR-0240.
3. **Bucket SSE with an OCI Vault key** before `forge-daily` is unpaused — its sqlite
   payload holds webhook secrets and is not encrypted by the app.
4. **Credential custody** for `velero-offnode-s3`, given it cannot come from OpenBao.
5. **Co-tenancy sign-off** from the console project for the Velero controller +
   DaemonSet footprint on the shared cell.
6. **ADR-0197 amendment** for the D-2 target clause.
7. **Restore drill** before this is called done. ADR-0197 D-6 requires it and an
   unrestored backup is not a backup. Drill target: restore `oya-kms` into a scratch
   namespace and confirm OpenBao unseals with the custodied shares.

## 6. This lands INERT — `infra/**` is not reconciled

Flagging this explicitly, because a manifest that lands here changes nothing today:

- **Neither Velero nor this directory is declared anywhere.** The app-of-apps list is
  `infra/gitops/values.yaml`; `velero`, `pgbackrest`, and `infra/backup` do not appear
  in it. `infra/nativelink/` does not appear either — the CAS was applied out of band,
  which is itself evidence that `infra/**` is not the reconciled source of truth.
- **The declared list is already broken.** Two of its eighteen entries point at paths
  that no longer exist (`microservices/cloud-intelligence/k8s`,
  `microservices/observability/iac/k8s/helm`), and `infra/gitops/bootstrap-sync.yaml`
  `include:`s `local-path-storage.yaml`, which is not in this repo. There is no
  `kind: StorageClass` anywhere under `infra/` — the `local-path` class every PVC names
  is assumed to exist, not declared.

Wiring is deliberately **not** done here. The insertion point is one entry in
`infra/gitops/values.yaml`, and adding it arms `automated: {prune: true, selfHeal: true}`
against a shared cluster. That is a founder decision, not a subagent's. The exact diff,
to be applied only after §5 clears:

```yaml
  # Off-node backup of non-reproducible substrate state (ADR-0197 as amended).
  # Requires the velero chart entry above it; both stay out until §5 clears.
  - name: backup
    type: path
    path: infra/backup
    namespace: velero
    syncWave: "2"
```

Landing that entry alone is not sufficient — Velero's CRDs and controller must be
declared first (a `type: chart` entry for `cloud/cloud-iac/iac/helm/velero/`, retargeted
per §2), or the Application fails to sync on unknown kinds.
