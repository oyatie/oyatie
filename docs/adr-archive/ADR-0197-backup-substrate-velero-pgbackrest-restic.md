---
id: ADR-0197
status: Superseded
deciders: council-architecture, axis-cloud-iac, ops-sre-reliability, council-privacy
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-701]
related: [ADR-0064, ADR-0131, ADR-0152, ADR-0162, ADR-0165, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership, ADR-0240-sovereign-cloud-per-regional-pack, ADR-0241-dr-business-continuity-portfolio-policy, ADR-0184, ADR-0186, ADR-0196]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0197 — Backup substrate: 3-pronged (Velero + pgBackRest + Restic) on SeaweedFS, age-encrypted

## Status

Accepted (2026-05-18). Mandates a three-pronged backup substrate where each
prong owns exactly one concern (K8s state vs Postgres PITR vs filesystem
state outside K8s). All targets land in SeaweedFS object storage (per ADR-
0196) and are encrypted with age before write.

## Context

ADR-0180 (DR + business continuity portfolio policy) establishes RPO/RTO
tiers but does not name the canonical backup tooling. ADR-0152 maps RPO/
RTO to workload classes (app / batch / regulatory) but defers tool
selection. ADR-0165 (chaos engineering substrate) runs disruption drills
but the restore-drill cadence has no concrete owner. ADR-0162 (per-tenant
audit-log slicing) requires audit-chain archival with regulatory
retention but does not name the executor.

The hyperscaler reference shape for backup at a multi-µservice K8s fleet:

- **Stripe** — Postgres physical backups (pg_basebackup + WAL archive) +
  K8s manifest archival + filesystem snapshots; recovery drills quarterly
  per their public incident retrospectives.
- **Pinterest** — Velero for K8s state; pgBackRest for Postgres PITR; Restic
  for non-K8s host state. Restore drills via chaos engineering.
- **Heroku** — pgBackRest is the canonical Postgres backup tool (Crunchy
  Data, now Crunchy Bridge, productionizes pgBackRest at multi-tenant SaaS
  scale).
- **GitLab** — Velero plus chaos-engineering quarterly restore drills as
  the operational pattern.

Anti-patterns this ADR forecloses:

1. Cloud-native snapshot only (EBS snapshots, GCE persistent disk
   snapshots) — no cross-region replication, no PITR semantics, vendor
   lock-in.
2. Manual `pg_dump` cron jobs — no PITR, no incremental, fragile.
3. Single backup tool covering all surfaces — each surface has a different
   ideal shape; one tool always compromises.

## Decision

### D-1. Three prongs, one per concern

- **Velero 1.18.0** — Kubernetes state (manifests, CRDs, ConfigMaps,
  Secrets, PVCs) + persistent volume content via the integrated
  filesystem-backup uploader (kopia, the modern replacement for restic in
  Velero). Velero is the only prong that understands Kubernetes objects.
- **pgBackRest 2.58.0** — Postgres point-in-time recovery (PITR) via WAL
  archive + full/differential/incremental backup cadence. pgBackRest is
  the only prong that understands Postgres WAL semantics. (Maintainer
  transition note: the original maintainer transitioned the project in
  2026; PGX provides continuity as `pgxbackup`. The
  `oya-shared-backup-kernel` adapter trait abstracts the executor so the
  migration is a single adapter swap if/when oyatie promotes pgxbackup.)
- **Restic 0.18.1** — Filesystem-level backup for non-K8s host state (e.g.
  bare-metal substrate host configs, node-local app data, ops bastions).
  Restic is the only prong covering state outside the K8s envelope.

### D-2. SeaweedFS as the single backup target (per ADR-0196)

- All three prongs land in SeaweedFS buckets:
  - Velero → `oya-velero-backup-shared-<env>`.
  - pgBackRest → `oya-backup-postgres-shared-<env>`.
  - Restic → `oya-backup-filesystem-shared-<env>`.
- Tenant-scoped backups use the same bucket purpose with tenant suffix
  (`oya-velero-backup-<tenant-id>-<env>`) when per-tenant residency or
  custom retention is required by a regulatory pack.
- One bucket purpose per prong keeps lifecycle policies, encryption keys,
  and retention rules separable.

### D-3. age encryption before write

- All backup payloads are encrypted with **age** (Apache 2.0; modern
  alternative to GPG; Rust impl `rage` 0.11.2 used in oyatie's Rust
  control-plane tooling).
- age was chosen over GPG because: small explicit keys (no web-of-trust
  complexity), no algorithm-negotiation rabbit holes, UNIX-style
  composability, an actively maintained Rust impl (`rage`).
- Keys are issued per-tenant + per-prong. Public keys are written to the
  Helm chart values; private keys live in OpenBao with quarterly rotation
  per ADR-0043.
- The bucket-level encryption layer (per ADR-0196 D-6) is additive — age
  ciphertext is what lands in the bucket.

### D-4. RPO / RTO per workload class (per ADR-0152)

| Class       | RPO       | RTO   | Prongs                                  |
|-------------|-----------|-------|-----------------------------------------|
| app         | ≤ 15 min  | ≤ 1 h | Velero hourly + daily; pgBackRest WAL   |
| batch       | ≤ 1 h     | ≤ 4 h | Velero daily; pgBackRest WAL            |
| gpu         | ≤ 1 h     | ≤ 4 h | Velero daily (model weights ephemeral)  |
| regulatory  | ≤ 5 min   | ≤ 30 m| Velero hourly + daily; pgBackRest WAL   |

### D-5. Retention policy (overlay-overridable per regulatory pack)

Default cadence:

- 7 days hourly.
- 30 days daily.
- 12 months weekly.
- 7 years annual.

Regulatory-pack overlays raise the floor:

- KR pack: 5 years (Personal Information Protection Act).
- EU pack: 7 years (CSRD + financial-sector retention).
- US healthcare (HIPAA): 6 years.
- US financial (SOX): 7 years.

Per-pack overlay lives in
`microservices/cloud-iac/iac/helm/velero/values-<pack>.yaml` and the
equivalent for pgBackRest and Restic.

### D-6. Restore drill cadence (per ADR-0165)

- **Quarterly** restore drills run via the chaos-engineering substrate.
- Drill targets one µservice per quarter on a rotation; the target's
  primary Postgres + PVs are restored to the `dr-staging` environment;
  validation runs the µservice's smoke-test set against the restored
  state.
- Drill evidence (success + duration + RPO/RTO observed) is sealed to the
  audit chain with `class: BackupRestoreDrill` and forwarded to ops-
  finops + the regulator-evidence quarterly emit (per ADR-0174).
- A drill failure is a SEV-2 incident — the µservice owner has 7 days to
  remediate or the µservice is downgraded one promotion tier.

### D-7. Disaster-recovery posture

- Cross-region replication of backup buckets is opt-in per regulatory
  pack. KR pack pins backups to KR (CSAP requirement). EU pack pins to
  EU + DR replica in EU-secondary. US packs replicate across two US
  regions.
- Recovery testing exercises both same-region and cross-region restore
  per quarter.

## Alternatives considered

### (a) Velero alone covering Postgres via filesystem backup — REJECTED

- **Pros:** one tool to operate.
- **Cons:** filesystem backup of an open Postgres cluster produces
  inconsistent snapshots without quiescing the database. PITR requires
  WAL archive, which is a Postgres-native concept. Recovery RPO drifts
  to "last successful filesystem snapshot" — often hours, sometimes a
  full day.
- **Rejected:** semantics gap.

### (b) Stash — REJECTED

- **Pros:** Kubernetes-native; CRD-driven.
- **Cons:** abandoned (last release 2022; transitioned to AppsCode
  commercial). Per ADR-0173 vendor lock-in avoidance, abandoned OSS is a
  forbidden default.
- **Rejected:** abandoned.

### (c) Commercial backup (Kasten K10, Cohesity DataProtect, Veeam) — REJECTED

- **Pros:** rich UI, vendor support, enterprise compliance reports.
- **Cons:** vendor lock-in (per ADR-0173); commercial licensing scales
  punitively with cluster size; sovereign packs (KR CSAP) require
  on-prem, license-portable tooling.
- **Rejected:** vendor lock-in + on-prem licensing.

### (d) Native cloud snapshots only (EBS, GCE PD) — REJECTED

- **Pros:** simplest; zero ops.
- **Cons:** vendor lock-in (per ADR-0173); no cross-region for restore
  without manual orchestration; no Postgres-aware PITR semantics; no
  on-prem story.
- **Rejected:** cannot satisfy hyperscaler invariants + sovereign packs.

### (e) Single tool: Restic for all — REJECTED

- **Pros:** simplest; one tool to learn.
- **Cons:** no K8s-object awareness (Restic backs up files, not Kubernetes
  CRDs); no Postgres-WAL awareness.
- **Rejected:** semantics gap for K8s + Postgres.

### (f) **CHOSEN:** Velero + pgBackRest + Restic, one per concern, all on
SeaweedFS with age encryption.

## Consequences

### Positive

- Each prong is optimal for its concern; no shoehorning.
- One target (SeaweedFS) for ops simplicity (single quota, single key
  rotation, single retention engine).
- age encryption is small, auditable, modern; no GPG sprawl.
- Restore drills are CI-automated per ADR-0165; evidence is sealed.

### Negative

- Three tools to operate. Mitigation: each tool wraps behind
  `oya-shared-backup-kernel` `BackupExecutor` trait; ops dashboards are
  unified per prong via Prometheus metrics + Grafana.
- pgBackRest maintainer transition adds operational risk. Mitigation:
  kernel trait + pgxbackup adapter ready to swap in.

### Neutral

- The 3-prong shape mirrors Pinterest + GitLab + Stripe public practice;
  hiring + ops familiarity is easier.

## In-house roadmap

Per the user directive "wherever possible, support in-house tech stack —
like AWS, Google, Microsoft, Oracle" (2026-05-18), the three prongs
ladder differently because each occupies a different category:

### pgBackRest — KEEP (community standard, in-house adapter not needed)

- pgBackRest is THE community canonical for Postgres PITR. AWS RDS,
  Google Cloud SQL, Azure DB for Postgres, Oracle Postgres Cloud all
  use it (or pg_basebackup + WAL archive, which pgBackRest is the
  productionized form of).
- **No Phase 2 in-house rebuild.** The maintainer-transition risk is
  hedged via the `oya-shared-backup-kernel` `BackupExecutor` trait;
  pgxbackup (continuity fork) is a 1-line adapter swap if/when needed.
- **Boundary** at which oyatie would reconsider: if pgxbackup itself
  becomes abandoned AND no other community fork picks it up. Today,
  that boundary is far away.

### Restic — KEEP (community standard, in-house adapter not needed)

- Restic is THE community canonical for filesystem backup. Used by
  ops/SRE communities, Backblaze, rsync.net, dozens of large
  enterprises.
- **No Phase 2 in-house rebuild.** Trait-wrapped via
  `oya-shared-backup-kernel`; alternates (Borg, Duplicacy) are
  adapter-swap candidates if Restic's pace slows.

### Velero — IN-HOUSE TARGETED

- Velero is a CNCF Sandbox project (per the 2026 KubeCon announcement
  moving toward Sandbox governance) maintained primarily by Broadcom-
  acquired-VMware. Per ADR-0173 vendor-lock-in posture, dependence on
  a single commercial steward is the exact failure mode oyatie hedges
  against.
- **Phase 0 (TODAY):** Velero 1.18 wrapped behind
  `oya-shared-backup-kernel`'s `BackupExecutor` trait. The K8s-state +
  PV-snapshot semantics are exposed via that trait; no application
  code calls Velero directly.
- **Phase 1 (M02-M03 horizon; ~Q4 2026):** harden the trait surface;
  add a Stash-deprecated-style fallback adapter only if Velero shows
  signs of stewardship risk in the wild.
- **Phase 2 (~Q2 2027 target):** build `oya-backup-orchestrator` — an
  in-house Rust orchestrator that drives pgBackRest + Restic +
  CSI-snapshot primitives directly, replacing Velero's
  Kubernetes-object-aware role with native Kubernetes-controller
  semantics tuned to oyatie's shape. The orchestrator becomes the third
  `BackupExecutor` adapter.
- **Build trigger** (one of):
  - Velero stewardship risk materializes (CVE flow slows, releases
    stall, or commercial-only feature gating begins).
  - Multi-tenant scale exposes a Velero-architectural limit (e.g.
    single-CRD-controller bottleneck at 10⁴+ namespaces).
  - AWS Backup / Google Backup / Azure Backup parity is needed at
    multi-tenant scale and Velero cannot supply it.
- **Parallel to:** AWS Backup (in-house), Google Backup and DR Service
  (in-house since the Actifio acquisition), Microsoft Azure Backup
  (in-house), Oracle ZFS Storage Appliance backup (in-house).
- **Migration shape:** the kernel trait is unchanged; cutover is
  per-µservice. Existing Velero schedules continue running until the
  per-µservice migration ack is recorded in `evidence/backup-cutover-
  ledger.jsonl`.

### Cross-prong direction

- Encryption (age / rage) — KEEP. age is the modern open-standard
  primitive; rage is the Rust impl oyatie already owns operationally.
- Object-store target (SeaweedFS today; in-house in Phase 2 per
  ADR-0196) — the backup substrate moves to in-house object store at
  the same horizon ADR-0196 lands its Phase 2.

This roadmap means the kernel trait `oya-shared-backup-kernel::
BackupExecutor` is the inviolate seam; vendor swaps are kernel-
adapter swaps, not application rewrites.

## Industry sources

- **Stripe public retrospectives** (PostgreSQL PITR + restore drill
  cadence): Stripe Engineering blog 2024-2025 series on availability.
- **Pinterest engineering** — *Velero + pgBackRest + Restic, restore
  drills via chaos engineering* (Pinterest Eng blog, K8s practice notes).
- **GitLab** — public DR runbook references Velero + quarterly restore.
- **Crunchy Data / Crunchy Bridge** — pgBackRest production posture at
  multi-tenant SaaS scale.
- **CNCF Velero project** — <https://velero.io/>; *Backing up Kubernetes
  applications* user guide.
- **pgBackRest user guide** — <https://pgbackrest.org/user-guide.html>.
- **age encryption** — Filippo Valsorda et al., *age: simple, modern, and
  secure file encryption*, <https://github.com/FiloSottile/age>.
- **rage (Rust impl)** — <https://github.com/str4d/rage> v0.11.2.

## Verification

- Helm charts at `microservices/cloud-iac/iac/helm/velero/` and
  `microservices/cloud-iac/iac/helm/pgbackrest/` render.
- `crates/oya-shared-backup-kernel/` `BackupExecutor` trait + the three
  prong adapters compile and test green.
- Restore-drill evidence emit is wired to the audit chain via
  `class: BackupRestoreDrill`.
- `crates/oya-check-backup-retention-discipline/` validates retention
  declarations against regulatory pack floors.

## Footnotes (versions verified 2026-05-18)

- Velero 1.18.0: <https://github.com/velero-io/velero/releases>.
- pgBackRest 2.58.0: <https://github.com/pgbackrest/pgbackrest/releases>.
- Restic 0.18.1: <https://restic.net/>.
- rage 0.11.2: <https://crates.io/crates/rage>.
- pgxbackup continuity note: <https://thebuild.com/blog/2026/05/01/pgxbackup-continuity-support-for-pgbackrest/>.
