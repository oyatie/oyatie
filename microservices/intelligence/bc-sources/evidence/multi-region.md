---
doc_class: MultiRegionPosture
microservice: foundry-evidence
status: Accepted
date: 2026-05-17
owner_team: ops-sre-reliability + council-privacy + axis-foundry-evidence
related_artifacts:
  - microservices/intelligence-evidence/PRD.md
  - microservices/intelligence-evidence/policy/data-residency.md
  - microservices/intelligence-evidence/runbooks/evidence-archive-migration.md
  - microservices/audit-chain/multi-region.md  (substrate)
doc_status: published
---

# foundry-evidence — multi-region posture

## Principle: pack-local data plane

Per `policy/data-residency.md` DR-02, evidence-pack data NEVER replicates across packs. Each pack carries its own audit-chain substrate chain; cross-pack chain merge is forbidden by cryptographic substrate design (Bominal ADR-0028 §"Chain locality").

This µservice's multi-region story is therefore **two-level**:
- **Cross-pack**: completely isolated. No cross-region data plane edges. Control-plane (manifests, schema-versions, claim-matrix) replicated.
- **Within-pack sub-region**: HA via active-active sub-region deployment (e.g., Frankfurt + Madrid for pack-eu; Ashburn + San Jose for pack-us). Within-pack replication is permitted because chain locality is preserved.

## Per-pack region map

| Pack | Primary region | Sub-region (HA) | Multi-region path |
|---|---|---|---|
| pack-kr | KR-Seoul | (single-region; KR PIPA Art. 28 restriction; HA via in-region AZ pairs) | active-active across 3 OCI Seoul ADs |
| pack-eu | EU-Frankfurt | EU-Madrid | active-active sub-region |
| pack-us | US-Ashburn | US-San Jose | active-active sub-region |
| pack-us-healthcare | US-Ashburn (HIPAA-eligible) | US-San Jose (HIPAA-eligible) | active-active sub-region |
| pack-jp | JP-Tokyo | (single-region; APPI) | in-region AZ pairs |
| pack-sg | SG | (single-region) | in-region AZ pairs |
| pack-au | AU-Sydney | (single-region) | in-region AZ pairs |
| pack-in | IN-Mumbai | (single-region; DPDP) | in-region AZ pairs |
| pack-br | BR-São Paulo | (single-region; LGPD) | in-region AZ pairs |
| pack-ae | AE-Abu Dhabi | (single-region) | in-region AZ pairs |
| pack-ksa | KSA-Riyadh | (single-region) | in-region AZ pairs |

## Active-active sub-region invariants (within-pack)

For packs with two sub-regions:

1. **Postgres logical replication** between sub-regions for the evidence index. Both sub-regions accept reads. Writes route to the current primary; cut-over via leader election.
2. **WORM blob** is single-substrate (substrate-managed cross-sub-region replication per `microservices/audit-chain/multi-region.md`).
3. **audit-chain bridge** runs in both sub-regions; each bridge worker has SPIFFE-bound mTLS to its sub-regional substrate endpoint.
4. **Workflow event bus** replicates within-pack; events delivered to both sub-regions' subscribers.
5. **Cedar policy** is replicated identically; LEAN lane `cedar-policy-fingerprint-match` blocks divergence.

## Single-region packs (in-region AZ HA)

For single-region packs:

1. Postgres deployed across 3 AZs (1 primary + 2 replicas).
2. Pack-builder + record-REST + bridge workers deployed across all 3 AZs with leader election.
3. WORM blob substrate replicated across AZs per Object Storage default.
4. RTO: ≤ 15 min for primary AZ failure (cut-over to secondary).
5. RPO: ≤ 1 s (period-aligned via substrate).

## Failover modes

### Sub-region failover (e.g., Frankfurt → Madrid)

Trigger: Frankfurt sub-region health-check fails for ≥ 5 min.

Procedure:
1. DNS-flip record_invocation REST endpoint to Madrid.
2. Postgres primary cut-over (logical replication promotes Madrid replica).
3. audit-chain bridge worker leader-elects to Madrid pod.
4. Workflow event subscriptions continue (bus is in-pack).
5. Frankfurt held in standby (read-only) once recovered; flip back per scheduled maintenance window.

RTO: ≤ 5 min. RPO: ≤ 1 s.

### AZ failover (single-region pack)

Trigger: primary AZ failure.

Procedure: standard k8s + Postgres HA cut-over; out-of-scope of foundry-evidence specifics.

RTO: ≤ 5 min. RPO: ≤ 1 s.

### Full-pack catastrophic failure (region down)

Trigger: entire pack region down (extremely rare; OCI region outage).

Procedure:
1. Tenant DPA notification.
2. Read continues from substrate-published Merkle roots if substrate is multi-region (substrate-defined; see `microservices/audit-chain/multi-region.md`).
3. Writes paused (no cross-pack route per data-residency).
4. Pack recovery follows substrate recovery; foundry-evidence Postgres re-syncs from substrate WORM blobs after substrate is restored.

RTO: ≤ 4 h (depends on substrate recovery). RPO: ≤ 1 s post-substrate-recovery.

This is an **intentional trade-off** per ADR-0117: data-residency wins over geographic redundancy. Cross-pack data plane is forbidden; per-tenant DPA may include a tenant-controlled out-of-pack archive as an additional resilience measure, but that archive is read-only and not part of the in-pack chain.

## DR drill cadence

| Drill | Cadence | Pass criterion |
|---|---|---|
| Sub-region failover | quarterly | RTO ≤ 5 min; RPO ≤ 1 s |
| AZ failover (single-region pack) | semi-annual | RTO ≤ 5 min; RPO ≤ 1 s |
| Substrate-down full-pack catastrophic | annual | recovery flow understood + RTO measurement |
| pack-region migration (Procedure B in `evidence-archive-migration.md`) | semi-annual | migration completes; data integrity preserved |

## Control plane (cross-pack replication permitted)

These are replicated globally:
- Schema versions registry (`/transparency/schema-versions`).
- Claim matrix (`/transparency/claim-matrix`).
- Framework profiles (`/transparency/framework-profiles`).
- Cedar policies (identical text across packs; pack-specific overlays as Cedar fragments).
- Helm charts + Kustomize bases.
- Capability catalog metadata.

Tenant data NEVER in this control plane.

## ADR-0133 honest gap

Cross-pack DR is not supported by design. The PRD is honest about this: pack-down means read-mostly until substrate recovers. Claim matrix declares "cross-pack RTO=not-supported" in the public `claim-matrix` endpoint.

## References

- `policy/data-residency.md`.
- `microservices/audit-chain/multi-region.md`.
- ADR-0117 (cloud-native infra).
- ADR-0028 (chain locality).
