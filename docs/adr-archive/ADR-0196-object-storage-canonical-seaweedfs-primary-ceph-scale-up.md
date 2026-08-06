---
id: ADR-0196
status: Superseded
deciders: council-architecture, axis-cloud-iac, ops-sre-reliability, axis-observability
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
amended_by: [ADR-0520]
related: [ADR-0064, ADR-0131, ADR-0161, ADR-0173-vendor-lock-in-avoidance-and-stack-ownership, ADR-0184, ADR-0186, ADR-0197, ADR-0199, ADR-0520]
related_specs:
  - /specs/hyperscaler-architecture-invariants.json
  - /specs/microservices/manifest-schema.json
---

# ADR-0196 — Object storage canonical: SeaweedFS primary, Ceph RGW scale-up path

## Status

Accepted (2026-05-18). Mandates SeaweedFS as the canonical S3-compatible
object store for oyatie's blob, artifact, evidence, audit-archive, and
warehouse-cold-tier workloads. Ceph RGW is the named scale-up path when
SeaweedFS hits its cluster ceiling (~1 PB / 10⁹ objects per cluster).

## Context

Oyatie's blob/artifact/evidence/cold-tier storage shape requires:

- **S3-compatible API** so any µservice — including ClickHouse cold tier,
  Milvus object backend, Velero backup target, audit-chain archival, and
  Workflow Studio attachments — speaks one wire protocol.
- **Horizontal scalability** to billions of objects without re-architecting.
- **Erasure coding** for capacity efficiency at the warehouse-cold tier.
- **Permissive OSS licensing** per ADR-0173 vendor-lock-in policy.
- **On-prem deployable** for sovereign packs (KR CSAP, EU GAIA-X) per
  ADR-0240-sovereign-cloud-per-regional-pack.
- **Active maintainer** + commercial-support availability for ops escalation.

The hyperscaler reference shape for blob storage:

- **AWS** — S3 as the canonical blob primitive; underpins EMR, Redshift cold
  tier, Glue catalog, Athena.
- **Google Cloud** — Cloud Storage (GCS) as the canonical blob primitive;
  underpins BigQuery long-term storage, Dataflow shuffle, Vertex AI dataset
  staging.
- **Stripe** — S3 + GCS dual-backend for ledger artifact archival; abstracted
  via a single Repository trait.
- **CERN** — Ceph object + block at exabyte scale (the ATLAS + ALICE
  detectors store on Ceph clusters totalling ~1.5 EB).
- **DigitalOcean Spaces, Western Digital ActiveScale** — Ceph RGW at multi-PB
  scale for their commercial object-storage offerings.

The ADR's job is to choose oyatie's primary object store and name the
scale-up path so we never get stuck mid-migration.

## Decision

### D-1. SeaweedFS 4.22 is the primary object store

- **License:** Apache 2.0.
- **Why primary:** simple operational model (master + volume + filer + S3
  gateway), strong erasure-coding support, S3-compatible API, filer-backed
  POSIX/WebDAV side surfaces, active maintainer, broad adoption (Sina Weibo
  with billions of objects, multiple sovereign-cloud deployments in Asia).
- **Scale envelope** validated for oyatie's first 3 years: ~1 PB and ~10⁹
  objects per cluster. Above that, the cluster topology becomes harder to
  reason about (master raft quorum cost rises sub-linearly; filer
  metadata-PG load rises).
- **Deployment:** Helm chart at `microservices/cloud-iac/iac/helm/seaweedfs/`
  with the canonical `_oya-helpers` library base per ADR-0064.
- **Topology:** 3-master raft quorum + 6-volume-node M-tier baseline +
  3-filer quorum + 4-S3-gateway. Cross-AZ replication on (`defaultReplication
  = "020"` — 2 racks). Cross-region replication opt-in per regulatory pack
  via volume-server pairing.

### D-2. Ceph RGW is the named scale-up path

- When a single SeaweedFS cluster approaches its ceiling, the canonical
  promotion path is **Ceph RGW** (the S3 gateway over Ceph's RADOS object
  store).
- **Why Ceph for scale-up:** proven at multi-PB / exabyte scale (CERN,
  DigitalOcean Spaces, Western Digital ActiveScale); deep maturity; broader
  ecosystem (CSI, CephFS, RBD) if oyatie later wants to unify object + block +
  file under one substrate.
- **Trigger:** cluster cap of 800 TB or 8·10⁸ objects (20 % below the
  empirical ceiling, leaves migration headroom).
- **Migration shape (already designed):** the `storage-object-store-kernel`
  trait abstracts the backend; the Ceph RGW adapter is a parallel
  implementation behind the same trait. Migration is a per-bucket
  dual-write + cutover; no application code change required.
- **Why NOT Ceph as today's primary:** higher operational complexity (rook +
  monitor + OSD + mgr + RGW + MDS stack); oyatie does not need that
  complexity at the 2026 scale floor; deferring keeps ops simple while
  preserving the migration path.

### D-3. Bucket naming + preallocation

- Canonical bucket name convention: `oya-<purpose>-<tenant-or-shared>-<env>`.
  Purpose vocabulary: `workflow-artifacts`, `evidence`, `clickhouse-cold`,
  `milvus-object`, `velero-backup`, `audit-chain-archive`, `backup-postgres`,
  `tenant-uploads`.
- Buckets are preallocated by `microservices/cloud-iac/` IaC, not created at
  runtime by application code. Application code receives the bucket name
  via Helm-values + manifest config.

### D-4. Secret references via OpenBao

- S3 access keys are sourced from OpenBao per ADR-0064 + ADR-0173 (open-
  standard primitives + no hardcoded credentials). The Helm values reference
  `openbao://secret/cloud-iac/seaweedfs/...` and a sidecar projects the live
  secret into the pod.

### D-5. Pre-signed URL surface (kernel-level)

- The `ObjectStore` trait exposes `presign_get(key, ttl)` and
  `presign_put(key, ttl, content_type)`. Pre-signed URLs are the canonical
  way application surfaces (Workflow Studio file uploads, Drive attachments)
  expose bytes to clients without proxying through application servers.
- TTLs are bounded: max 15 min for GET, 30 min for PUT. Longer-lived
  access must use authenticated streaming via the µservice's read-path API,
  not extended pre-signed URLs.

### D-6. Encryption + integrity

- Server-side encryption (SSE) MUST be enabled on every bucket. Keys are
  rotated quarterly per ADR-0043 secrets-rotation policy.
- Per-object SHA-256 checksum verification MUST be enabled. Mismatches emit
  to the audit chain as `class: ObjectIntegrityFailure`.

## Alternatives considered

### (a) MinIO — REJECTED

- **Pros:** broad adoption; mature S3 compatibility; strong tooling.
- **Cons:** licence transition in 2024 — MinIO Community edition is AGPL3,
  and increasingly the operationally important features (replication
  policy editor, IAM CLI surface, lifecycle GUI) are gated to the
  commercial Enterprise tier. Per ADR-0173 (vendor lock-in avoidance + open-
  standard primitive doctrine) AGPL3 plus a feature-gating commercial gate
  creates exactly the asymmetry the doctrine forbids: oyatie would have to
  rebuild several operationally critical surfaces or accept commercial
  licensing for what was previously free.
- **Rejected:** licensing direction + feature-gating asymmetry.

### (b) Garage — REJECTED for primary, NOT for sidecar use

- **Pros:** Rust-native; permissive (AGPL3 — but small enough that internal
  audit is tractable); excellent for edge / sidecar deployments.
- **Cons:** smaller community; less proven at oyatie's target scale of
  ~PB/cluster; AGPL3 still has the network-clause obligation; tooling
  ecosystem thinner than SeaweedFS or Ceph.
- **Rejected** as primary; **retained** as candidate for edge sidecar
  workloads (e.g. per-cell ephemeral object caches) in a future ADR.

### (c) Cloud-native S3 / GCS / Azure Blob only — REJECTED

- **Pros:** zero ops burden; battle-tested at hyperscaler scale.
- **Cons:** vendor lock-in (per ADR-0173 explicitly forbidden as the
  default). Sovereign packs (KR CSAP, EU GAIA-X) require on-prem option.
  Pricing exposure for warehouse-cold tier at oyatie's data shape would be
  punitive without long commitments.
- **Rejected** as the only choice; **retained** as a backend adapter via
  the same `ObjectStore` trait, used per regulatory pack when the pack
  permits public-cloud egress.

### (d) Ceph RGW today (skip SeaweedFS) — REJECTED for current scale

- **Pros:** the named scale-up target; mature; multi-PB-proven.
- **Cons:** operational complexity is high (rook + 5+ daemon types); oyatie's
  current scale does not justify the ops burden. Investing in SeaweedFS now
  + keeping the migration shape ready buys 2-3 years of simpler ops with
  no migration debt (because the kernel trait abstracts the backend).
- **Rejected today**; **scale-up path** when SeaweedFS ceiling approaches.

### (e) CHOSEN: SeaweedFS 4.22 primary + Ceph RGW scale-up path

## Consequences

### Positive

- Permissive licensing (Apache 2.0) with no AGPL contagion.
- Simple ops at current scale; clean upgrade path to Ceph for hyperscale.
- Single S3 API across the fleet; one Repository trait via
  `storage-object-store-kernel`.
- Pre-signed URLs are uniform across all bucket consumers.

### Negative

- SeaweedFS has a smaller commercial-support market than Ceph; ops
  escalation for severity-1 incidents has fewer vendor options. Mitigation:
  contract with SeaweedFS commercial support + retain Ceph adapter ready
  for emergency promotion.
- Migration to Ceph at the ceiling has dual-write cost (operationally
  manageable, but real). Mitigation: kernel-trait + per-bucket cutover
  shape designed in advance.
- SeaweedFS S3 API has minor compatibility gaps vs AWS S3 (e.g. some Lambda
  notification surfaces). Mitigation: any feature requiring strict AWS
  parity uses the AWS S3 adapter via the same trait (acceptable for cloud-
  hosted packs only).

### Neutral

- Forces every blob consumer to use the `ObjectStore` trait, not direct
  HTTP calls. This is a code discipline win but raises the bar for
  capability authoring.

## In-house roadmap

Per the user directive "wherever possible, support in-house tech stack —
like AWS, Google, Microsoft, Oracle" (2026-05-18), oyatie ladders this
substrate from vendor to in-house in three phases:

### Phase 0 — vendor-via-adapter (TODAY)

- SeaweedFS 4.22 is the primary; wrapped behind `oya-shared-object-store-
  kernel`'s `ObjectStore` trait. No application code touches SeaweedFS
  directly. The trait is the seam.
- Ceph RGW adapter is parallel-implemented for the scale-up path
  (≥800 TB or ≥8·10⁸ objects per cluster).
- AWS S3 / GCS / Azure Blob adapters exist for cloud-hosted regional
  packs that permit public-cloud egress.

### Phase 1 — adapter hardening (M02-M03 horizon; ~Q4 2026)

- Add adapters for the named scale-up targets (Ceph RGW) and named
  alternates (Garage for sidecar use).
- Pre-signed-URL surface stabilized; lifecycle-policy primitives
  abstracted (no SeaweedFS-specific surface leaking to call sites).
- Conformance harness validates every adapter against the same
  acceptance suite.

### Phase 2 — in-house object store (~Q2 2027 target)

- Build `oya-object-store-server`: a Rust-native, S3-compatible object
  store designed for oyatie's exact workload shape (artifact +
  evidence + warehouse-cold + audit-archive + Velero target).
- **Core features:**
  - Reed-Solomon erasure coding (10+4 default) with operator-tunable
    shape per bucket.
  - Active-active multi-region replication with conflict-free vector-
    clock metadata (per ADR-0142 CRDT portability trait shape).
  - Lifecycle policies (TTL, tier transition to deep archive).
  - Pre-signed-URL primitives natively integrated with OpenBao key
    rotation.
  - SPIFFE-ID based authentication (no static access keys); per ADR-0148.
- **Build trigger** (one of):
  - Single cluster hits ≥ 1 PB OR ≥ 10⁹ objects (SeaweedFS ceiling).
  - Multi-region active-active write coordination required (SeaweedFS
    does not natively coordinate active-active writes across regions).
  - Sovereign packs require a 100 % Oya-controlled binary in supply chain
    (current SeaweedFS supply chain has dependencies oyatie does not
    fully control).
- **Migration shape:** the kernel trait is unchanged; the new server
  becomes a third adapter alongside SeaweedFS + Ceph. Existing buckets
  dual-write during cutover; consistency is verified via the
  conformance harness.
- **Parallel to:** AWS S3 (in-house since 2006), Google Cloud Storage
  (Colossus underneath), Microsoft Azure Blob (Stream underneath),
  Oracle Object Storage. Every hyperscaler ladders from "use someone
  else's" to "build our own"; oyatie pre-stages the seam now.

### Phase 3 — federated object plane (~2028 horizon)

- `oya-object-store-server` instances federate across cells per ADR-
  0009; per-tenant locality is managed by the object plane itself, not
  by application code.
- Cross-pack data movement governed by ADR-0240-sovereign-cloud-per-
  regional-pack residency rules at the object layer (not application
  layer).

This roadmap is recorded so the seam (`ObjectStore` trait) is honored
across every consumer today; migration when triggered is a per-bucket
operational task, not an application rewrite.

## Industry sources

- **SeaweedFS** — Sina Weibo's deployment shape and tooling notes:
  <https://github.com/seaweedfs/seaweedfs> (2026-05-18).
- **Ceph at CERN** — *Practical experiences with the Ceph storage system at
  the ATLAS experiment*, J. Phys.: Conf. Ser. ATLAS-2024; CERN runs Ceph
  clusters totalling ~1.5 EB across LHC experiments.
- **DigitalOcean Spaces on Ceph RGW** — public engineering blog series;
  multi-PB commercial offering.
- **Western Digital ActiveScale** — commercial Ceph-derived object store at
  PB scale.
- **AWS S3 reference architecture** — *S3 Best Practices Design Patterns*,
  AWS Well-Architected Framework.
- **Stripe** — public engineering practice (ledger artifact archival via
  S3 + GCS dual-backend behind a Repository trait).
- **FOCUS 1.3** — FinOps Open Cost & Usage Specification 1.3 (ratified
  2025-12-05): <https://focus.finops.org/focus-specification/>.

## Verification

- Helm chart at `microservices/cloud-iac/iac/helm/seaweedfs/` renders.
- `storage/core/object-store-kernel/` `ObjectStore` trait + the
  SeaweedFS adapter compile and test green via `cargo test -p
  storage-object-store-kernel`.
- Tracked W1 storage-kernel files under this decision:
  `storage/core/object-store-kernel/Cargo.toml`,
  `storage/core/object-store-kernel/BUCK`,
  `storage/core/object-store-kernel/src/lib.rs`, and
  `registry/catalog/storage-object-store-kernel.yaml`.
- Bucket naming convention enforced by `oya-check-tenant-cost-labels-
  coverage` (advisory) and the IaC-side bucket preallocator.

## Amendment (2026-06-08, WAVE-1 Agentic Delivery Fabric convergence)

Amended in place (no tombstone; git history preserves the pre-amendment body). **ADR-0520** qualifies
SeaweedFS/Ceph as explicitly **TRANSITIONAL** behind the `object-store-kernel` interface: the W5
bespoke infinite-scale object-store (the "Phase 2 in-house object store" already foreseen in the
in-house roadmap above) sits behind that interface and SUPERSEDES SeaweedFS/Ceph at a parity-gated
cutover, consistent with the ADR-0482 bridge-discipline. The `ObjectStore` trait remains the seam; the
choice of SeaweedFS as today's primary and Ceph as the named scale-up path is unchanged for the
transitional period.

## Footnotes (versions verified 2026-05-18)

- SeaweedFS 4.22 (released 2026-04-29): <https://github.com/seaweedfs/seaweedfs/releases>.
- Ceph Squid 19.2.x: <https://docs.ceph.com/en/latest/releases/>.
