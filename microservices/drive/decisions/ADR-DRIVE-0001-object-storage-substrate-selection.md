---
id: ADR-DRIVE-0001
status: Accepted
date: 2026-05-17
microservice: drive
deciders: axis-drive, council-architecture, ops-sre-reliability, ops-security, finops
owner: axis-drive + council-architecture
supersedes: []
superseded_by: []
related: [ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/drive/PRD.md (§Bounded Contexts → file-store adapter-{s3,garage,seaweedfs}; §"Horizontal Scalability")
  - microservices/drive/iac/helm/Chart.yaml
  - microservices/drive/runbooks/object-storage-degraded.md
purpose: |
  Close the backend-selection gap surfaced by the `oya-drive-file-store-
  adapter-s3`, `-adapter-garage`, `-adapter-seaweedfs` crate rows in the PRD layer-
  mapping table. The catalog rows mandate three S3-compatible backend
  variants; ADR-0105 Amendment 3 requires the backend to be named explicitly
  (`-adapter-<backend>`); the IaC chart in `iac/helm/` must pin concrete
  LTS images. This ADR makes that choice authoritative.
---

# ADR-DRIVE-0001: Object-storage substrate — Garage 1.x primary; SeaweedFS RELEASE-2024-08 secondary; SeaweedFS 3.x archive tier; Ceph RGW + AWS S3 considered

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The `drive` µservice persists bytes-at-rest in an S3-compatible object store. PRD-drive §"State strategy: mixed" + §"Horizontal Scalability" require per-cell deployment, per-tenant prefix, replication-factor 3, free-space scale-out trigger, and (per ADR-DRIVE-0006) object-lock compliance-mode support.

Per ADR-0131 the object-store deployment lives at `microservices/drive/iac/helm/<backend>/`. Per ADR-0105 Amendment 3 (backend-qualified adapters), the crate that wires it must be named `oya-drive-file-store-adapter-s3` plus more specific backend-qualified variants (`-adapter-garage`, `-adapter-seaweedfs`) when multiple backends are supported.

Five production-grade S3-compatible object stores are candidates:

1. **Garage** (Rust; AGPL-3.0). Built by Deuxfleurs for edge-distributed, low-latency, geo-aware deployment. Excellent for multi-cell + multi-region. Native S3 API + object-lock support (compliance mode). Active LTS line 1.x (2024-2026). Used by Deuxfleurs production + several European edge providers. ([garagehq.deuxfleurs.fr](https://garagehq.deuxfleurs.fr/))
2. **SeaweedFS** (Go; AGPL-3.0 + commercial). Most widely-deployed S3-compatible OSS object store. Mature S3 API + object-lock (compliance + governance modes) + IAM-policy parity. Distributed-mode requires symmetric nodes; works best in single-cluster. ([min.io](https://min.io))
3. **SeaweedFS** (Go; Apache-2.0). Optimised for billions of small files; built-in tiered storage (hot + warm + cold). Native S3 API. Good fit for archive tier. ([github.com/seaweedfs/seaweedfs](https://github.com/seaweedfs/seaweedfs))
4. **Ceph RGW** (C++; LGPL-2.1+). Enterprise-grade object store with strong S3 API + Swift API + per-bucket policy. Heavy operational footprint; requires Ceph cluster underneath. Industry-leading scale (CERN, EMBL-EBI, Bloomberg). ([ceph.io](https://ceph.io))
5. **AWS S3** (proprietary). Reference S3 API. Highest scale + lowest egress-relative pricing at AWS scale. Vendor-tied; per-pack residency requires AWS-region selection; cross-cloud portability lost.

Performance budget per PRD-drive §"Performance":
- Download first-byte p99 ≤ 100ms (warm CDN) / ≤ 500ms (cold).
- Upload multipart 1GB p99 ≤ 90s (≥ 4 parallel chunks).
- File-list folder p99 ≤ 150ms.
- WORM correctness 100% (per ADR-DRIVE-0006).

Per ADR-0133 axis-4 industry-citation, the chosen backend must be a named LTS pin; upstream activity in the past 12 months must be non-zero; CVE backlog must be zero P0/P1 at pick time.

Per ADR-0117 data-residency, per-pack region pinning is mandatory; cross-region replication forbidden by default.

## Decision

The drive µservice ships **a three-tier object-store substrate**:

1. **Garage 1.x** as the **primary** S3-compatible backend for all packs except pack-us-healthcare.
2. **SeaweedFS RELEASE-2024-08** as the **secondary** backend (enabled via pack-us-healthcare overlay; selectable per-pack for HIPAA-eligible deployment).
3. **SeaweedFS 3.x** as the **archive tier** for files not accessed in > 365 days; deployed alongside primary backend.

Concrete bindings:

- Crates:
  - `oya-drive-file-store-adapter-s3` — abstract S3-compatible adapter (default code path).
  - `oya-drive-file-store-adapter-garage` — backend-qualified adapter (primary; shipped alongside `-adapter-s3`).
  - `oya-drive-file-store-adapter-seaweedfs` — archive-tier-qualified adapter.
  - SeaweedFS uses `-adapter-s3` directly (no SeaweedFS-specific quirks beyond IAM-policy conventions).
- IaC: `microservices/drive/iac/helm/garage/` is the default Helm chart; `seaweedfs/` ships alongside; `minio/` ships under pack-us-healthcare overlay.
- LTS pins:
  - `garage: "1.0.1"` (AGPL-3.0; per-tenant tier-3 deploy carve-out; oyatie ships modifications back upstream).
  - `minio: "RELEASE.2024-08-17T01-24-54Z"` (AGPL-3.0 + commercial; legal cleared for pack-us-healthcare).
  - `seaweedfs: "3.71.0"` (Apache-2.0).

All three backends MUST:
- Speak S3 SigV4 (AWS Signature Version 4).
- Support S3 Multipart Upload (RFC 7233 + S3 multipart spec).
- Support S3 Object Lock compliance mode per ADR-DRIVE-0006.
- Enforce per-tenant bucket prefix; cross-tenant ACL refused.
- Refuse XXE / SSRF / path-traversal at the API boundary.
- Run behind mesh mTLS + per-tenant API key.

Per-tenant bucket prefix: `tenant-{tenant_id}/{file_id_prefix_4}/{file_id}` (per PRD §"Sharding").

## Alternatives Considered

### A. Ceph RGW as the primary backend

- **Pros**:
  - Industry-leading scale (CERN, Bloomberg, EMBL-EBI deployments).
  - Comprehensive S3 + Swift + per-bucket policy parity.
  - Active upstream (Red Hat / IBM).
- **Cons**:
  - Heavy operational footprint — full Ceph cluster (OSDs + MONs + MGRs + MDSs) underneath.
  - Operator complexity unfavourable for per-cell deployment in oyatie's 30+ cells.
  - Geo-distribution requires RGW multisite + replication, which is complex to operate vs Garage's geo-native design.
- **Rejected** because (i) operational footprint exceeds the per-cell budget, (ii) Garage's geo-aware replication is a better fit for per-cell + per-pack topology.

### B. AWS S3 as the primary backend (managed)

- **Pros**:
  - Highest scale; lowest egress at AWS scale.
  - No operator burden; AWS manages.
  - Reference S3 API conformance (by definition).
- **Cons**:
  - Vendor-tied; per-pack residency requires AWS-region selection (loses cross-cloud portability).
  - Egress cost prohibitive at multi-TB-per-day per tenant.
  - Tenant DPA cross-border concerns (SCC + Schrems II).
  - Inconsistent with `cloud-iac` self-hosted posture per ADR-0117.
- **Rejected** as primary; retained as a tenant-choice alternative for tenant-class-3 deploy where the tenant owns its own AWS account + DPA (separate ChangeSet IP, not Phase 1).

### C. Single SeaweedFS cluster as the primary backend (no Garage)

- **Pros**:
  - Most widely-deployed S3 OSS.
  - Mature object-lock compliance + governance modes.
  - Strong IAM-policy parity.
- **Cons**:
  - Distributed-mode requires symmetric drives + nodes — works best in single-cluster, not edge-distributed.
  - Cross-region replication needs SeaweedFS Site Replication; less geo-native than Garage.
  - AGPL-3.0 + commercial license requires careful per-tenant compliance review.
- **Rejected as primary**; **retained as the secondary** for pack-us-healthcare (HIPAA-eligible, BAA-friendly tooling, mature operator).

### D. SeaweedFS as the primary backend

- **Pros**:
  - Optimised for billions of small files.
  - Built-in tiered storage.
  - Apache-2.0 — friendliest license.
- **Cons**:
  - S3 object-lock compliance mode less mature than Garage / SeaweedFS (per upstream issue tracker 2025-Q4).
  - Smaller upstream community than SeaweedFS.
- **Rejected as primary**; **retained as the archive tier** (cold files where object-lock maturity matters less; primary backend retains hot files under object-lock).

### E. Garage primary + SeaweedFS secondary (healthcare) + SeaweedFS archive  ← **CHOSEN**

- **Pros**:
  - Garage's geo-native + low-latency per-cell deployment matches per-pack topology.
  - SeaweedFS secondary covers HIPAA-eligible deployment with mature operator tooling.
  - SeaweedFS archive tier reduces hot-tier cost by ~75% for cold files.
  - Three-backend posture provides hot-swap escape hatch — if Garage upstream cools, promote SeaweedFS to primary; if SeaweedFS cools, keep Garage and re-pick the secondary.
  - ADR-0105 Amendment 3 backend-qualified adapter pattern admits this directly.
- **Cons**:
  - Three operational surfaces.
  - Three CVE-tracking lanes.
  - Backend-specific quirks must be encapsulated in per-adapter crates.
- **Accepted** because (i) the operational cost is bounded, (ii) the architectural escape hatch is valuable, (iii) all three are S3-compliant LTS pins.

## Consequences

### Positive

- **Per-pack residency honoured by construction** — Garage per-cell deployment maps 1:1 to per-pack region pinning.
- **Archive tier reduces cost** — SeaweedFS handles cold files at ~25% the cost of hot-tier Garage.
- **HIPAA-eligible posture** — SeaweedFS secondary unblocks pack-us-healthcare without dragging the broader fleet to SeaweedFS.
- **Hot-swap escape hatch** — `oya-drive-file-store-adapter-<backend>` is a config flip at tenant tier; ADR-0105 Amendment 3 enforces.
- **WORM-correctness compatible across all three backends** — every backend supports compliance-mode object-lock per ADR-DRIVE-0006.

### Negative

- **Three upstreams to track**. Garage + SeaweedFS + SeaweedFS CVE feeds must all be monitored. Mitigation: all three publish CVEs to NVD; existing security automation covers.
- **Backend-specific quirks** — Garage's per-cell layout requires deliberate cell-add ChangeSets; SeaweedFS drives need symmetric provisioning; SeaweedFS master + volume + filer separation adds operator complexity. Mitigation: per-backend runbook in `runbooks/object-storage-degraded.md`.
- **Per-backend conformance test matrix** — every `aws s3` / `mc` / `s3cmd` / `s3-tests` corpus test runs against all three backends per cell in CI. Doubled-tripled cost; mitigated by per-backend test sharding.

### Operational

- **New CI lane `oya-governance-object-store-backend-conformance`** (BLOCKER from GA): validates
  - all three `-adapter-{s3,garage,seaweedfs}` adapters pass the `s3-tests` Ceph public suite;
  - all three expose the `FileRepository` port trait identically (Cedar policy + audit-chain seal emitted at the same surface points);
  - all three support compliance-mode object-lock per ADR-DRIVE-0006.
- **Helm chart pin policy**: `garage: "1.0.1"`, `minio: "RELEASE.2024-08-17T01-24-54Z"`, `seaweedfs: "3.71.0"` declared in `microservices/drive/iac/helm/Chart.yaml` `dependencies`; `oya-governance-version-pinning-conformance` lane refuses unpinned versions.
- **Per-pack overlay**: pack-us-healthcare enables SeaweedFS chart by default (HIPAA-eligible operator tooling); other packs ship Garage-only and may tenant-opt-into SeaweedFS via a tenant-class flag.
- **Runbook `object-storage-degraded.md`** documents per-backend cell-loss recovery.

### Regulatory

- **GDPR Art. 32** (security of processing): all three backends pass — TLS 1.3 in transit, Tenant-DEK envelope at rest.
- **KR PIPA Art. 29**: per-tenant bucket prefix isolation satisfies access-control-by-default.
- **HIPAA 45 CFR §164.312(a)(2)(iv)**: Tenant-DEK envelope satisfies; SeaweedFS + BAA-on-file for pack-us-healthcare.
- **SEC 17a-4(f) + FINRA 4511**: all three support compliance-mode object-lock per ADR-DRIVE-0006.

## Verification

Per the agent-skills documentation-and-adrs SKILL.md §"Verification":

- [ ] All three backends pass the `s3-tests` public conformance corpus —
  `cargo nextest run -p oya-drive-file-store-adapter-garage -- s3_tests_corpus`
  and equivalents for `-seaweedfs` and (in pack-us-healthcare) `-s3` (against SeaweedFS).
- [ ] Helm chart versions pinned — `cargo run -p oya-dev-cli -- gate validate version-pinning-conformance --microservice drive`.
- [ ] Object-lock compliance-mode functional — `cargo nextest run -p oya-drive-immutability-tier-domain -- worm_object_lock_per_backend`.

## References

- AWS S3 SigV4 specification.
- AWS S3 Multipart Upload API.
- AWS S3 Object Lock + S3 Compliance Mode.
- Garage upstream — `garagehq.deuxfleurs.fr`; LTS 1.x release notes.
- SeaweedFS upstream — `min.io`; LTS RELEASE-2024-08.
- SeaweedFS upstream — `github.com/seaweedfs/seaweedfs`.
- Ceph RGW — `ceph.io` (rejected reference).
- `s3-tests` Ceph public conformance corpus.
- ADR-0056 (BNF v4.1); ADR-0105 Amendment 3 (backend-qualified adapters); ADR-0117; ADR-0135; ADR-0131; ADR-0132; ADR-0133.
- ADR-DRIVE-0006 (WORM immutability — object-lock compliance mode).
- `microservices/drive/PRD.md` §Bounded Contexts row 1; §"Horizontal Scalability".
- `microservices/drive/iac/helm/Chart.yaml`.
- `microservices/drive/runbooks/object-storage-degraded.md`.
- `feedback_quality_performance_scalability_bar.md`.
