---
doc_class: MultiRegionPlan
template_id: TPL-MULTI-REGION
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-sre-reliability + cloud-iac
doc_status: published
---

# Multi-region + DR plan — slides µservice

## Topology

| Pack | Primary region | Secondary (DR) | RTO | RPO | Rationale |
|---|---|---|---|---|---|
| kr | ap-seoul-1 (OCI) | ap-tokyo-1 (read-replica DR) | 30s | 1s (CRDT cache) / 5s (deck spec) | PIPA residency; KR-FSS compliance for fintech tenants |
| eu | eu-frankfurt-1 | eu-paris-1 (active-active reads; primary failover) | 30s | 1s / 5s | GDPR residency; Schrems II adequacy |
| us | us-ashburn-1 | us-phoenix-1 (warm standby) | 30s | 1s / 5s | US baseline |
| us-healthcare | us-ashburn-1 (HIPAA-isolated) | us-phoenix-1 (HIPAA-isolated warm) | 30s | 1s / 5s | HIPAA-Business Associate region + BAA |
| jp | ap-tokyo-1 | ap-osaka-1 | 30s | 1s / 5s | APPI residency |
| sg | ap-singapore-1 | ap-melbourne-1 | 30s | 1s / 5s | PDPA SG; ASEAN proximity |
| au | ap-sydney-1 | ap-melbourne-1 | 30s | 1s / 5s | APP residency |
| in | ap-mumbai-1 | ap-hyderabad-1 | 30s | 1s / 5s | DPDPA + IN-region |
| br | sa-saopaulo-1 | sa-vinhedo-1 | 30s | 1s / 5s | LGPD residency |
| ae | me-dubai-1 | me-jeddah-1 | 30s | 1s / 5s | UAE PDPL residency |
| ksa | me-jeddah-1 | me-dubai-1 (regulator-permitted cross-GCC) | 30s | 1s / 5s | KSA PDPL residency |

## Active-active vs active-passive

| Surface | Mode | Rationale |
|---|---|---|
| editor-rest | active-active within pack | stateless; per-pack residency holds |
| real-time-collaboration-worker (CRDT WS) | active-active within pack; deck-shard via consistent-hash | single-writer per deck via Redis lease; lease pack-pinned |
| broadcast-mode-worker | active-active within pack; deck-shard | single-writer per broadcast session via Redis lease; LiveKit SFU pack-pinned via messenger |
| Postgres (Citus) | primary write + read-replicas; cross-region async replica for DR | RPO 5s via WAL async ship |
| Redis | primary cluster + sentinel; cross-region not replicated (ephemeral) | RPO 1s via in-memory; reconstructable from Postgres on cold-start |
| S3 deck snapshots + assets | cross-region replication (per-pack policy) | RPO 5s; bucket-replication |
| Export workers (PPTX/PDF/MP4 in gVisor) | active-active within pack | stateless; job-queue claims via Redis |
| CDN (WASM + theme/template gallery) | global multi-edge | TTL-cached; immutable WASM chunks |

## Failover playbook

1. **Detection**: pack SLI burn-rate alarm fires (10× burn 1h or 6× burn 6h); ops-sre-reliability on-call paged.
2. **Triage** (5min): confirm primary region degraded vs slides-pod-level issue; check messenger + sheets + foundry-runtime status; check CDN edge status.
3. **Region failover**: per `runbooks/`, promote secondary read-replica to primary; flip CDN origin DNS; drain primary editor-rest pods; flush Redis lease state to force re-acquisition on secondary; messenger LiveKit pack failover triggered.
4. **Verify**: editor REST `/health` + WS `/upgrade` on secondary; cargo-leptos WASM bundle SRI verifies; CDN warm-up for top 100 tenants per pack.
5. **Announce**: tenant comms via mail/banner; SLO burn-rate dashboard shared.
6. **Rollback**: when primary recovers and async replica catches up to within RPO, plan failback at next maintenance window.

## DR drill cadence

- Quarterly per pack (all 11 packs cycle through).
- Annual cross-pack failover drill (validates that pack isolation does NOT prevent within-pack failover; cross-pack collab REMAINS forbidden).
- Drill records under `evidence/dr-drills/slides/`.

## Cross-pack rules

- Editor session, deck content, assets, ACL state: **never** cross pack.
- Cross-pack collab: **forbidden**; refused at admission gate.
- AI invocations: foundry-runtime owns cross-pack residency; slides forwards within pack only.
- Audit-chain: per-pack ledger; never cross-pack.
- Broadcast viewers (attendee count): aggregate-only, never cross-pack.
- Public-read decks (link-anyone-with-link): per-pack rendered preview; viewer pack pinned at issue time (refused if pack mismatch with deck pack).

## Network design

- Per-pack VCN + private subnets per BC tier.
- Cross-µservice mTLS via SPIFFE identity (service-mesh).
- Egress allowlist per `iac/helm/*/values.yaml` networkPolicy.egress.
- No internet egress from gVisor export workers (offline transcode only).

## Observability + alerting per region

- Per-region Grafana stack (consumes observability µservice tenancy boundary).
- Per-pack burn-rate alarms feed Grafana OnCall.
- DR drill alarms (planned failover) suppress Sev-1 paging within drill window.

## Backup + restore

| Asset | Backup method | Restore RPO | Restore RTO | Encryption |
|---|---|---|---|---|
| Postgres | Continuous WAL + nightly snapshot | 5s | 30min | TDE + per-pack KMS |
| S3 deck snapshots | Cross-region replication + lifecycle (per-pack retention) | 5s | 30min | SSE-KMS per-pack |
| Redis CRDT cache | none (ephemeral; reconstruct from Postgres) | — | 1min cold-start | KMS-encrypted on disk if persisted |
| Audit-chain seals | append-only ledger + cross-region replication | 5s | 30min | Ed25519 signed; per-pack ledger |
| Per-pack themes/templates | signed bundles in S3 + CDN | 5s | 30min | Ed25519 signed; SSE-KMS |

## References

- ADR-0117 per-pack residency.
- ADR-0139 SLO-gated promotion.
- ADR-SLIDES-0005 broadcast-mode LiveKit reuse (cross-pack rules inherited from messenger).
- cloud-iac per-pack region IaC.
