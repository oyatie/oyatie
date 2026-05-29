---
doc_class: MultiRegion
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: cloud-iac + axis-forms + ops-sre-reliability
review_cadence: annually + on every pack activation
doc_status: published
---

# Forms — Multi-Region Strategy

## Posture: Pack-pinned, Multi-AZ per pack, Single-region per pack default; DR pair where statute permits

Forms inherits the workflow-studio / sheets posture: tenant data is **pack-pinned**; cross-pack movement is forbidden by default (per `policy/data-residency.md`).

## Per-Pack Layout

| Pack | Primary region | DR region | DR mode | Notes |
|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | n/a (single-region) | Backup-restore only | PIPA Art. 28 + KR-FSS sector compatible |
| pack-eu | OCI eu-frankfurt-1 | OCI eu-amsterdam-1 | Active-passive within EU | GDPR cross-region OK intra-EU |
| pack-us | OCI us-ashburn-1 | OCI us-phoenix-1 | Active-passive within US | CCPA intra-state OK |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | OCI us-phoenix-1 (HIPAA-eligible) | Active-passive within US-HC | BAA-required |
| pack-jp | OCI ap-tokyo-1 | n/a | Backup-restore only | APPI |
| pack-sg | OCI ap-singapore-1 | n/a | Backup-restore only | PDPA |
| pack-au | OCI ap-sydney-1 | OCI ap-melbourne-1 | Active-passive | Privacy Act |
| pack-in | OCI ap-hyderabad-1 | OCI ap-mumbai-1 | Active-passive | DPDPA |
| pack-br | OCI sa-saopaulo-1 | OCI sa-vinhedo-1 | Active-passive | LGPD |
| pack-ae | OCI me-abudhabi-1 | OCI me-dubai-1 | Active-passive | UAE PDPL |
| pack-ksa | OCI me-jeddah-1 | OCI me-riyadh-1 | Active-passive | KSA PDPL + NCA |

## DR Targets (per pack)

| Metric | Target | Notes |
|---|---|---|
| RPO (data loss) | ≤ 60s | Citus continuous WAL ship to DR replica |
| RTO (service restore) | ≤ 15min | Automated failover (active-passive); manual escalation for cross-region restore |
| Backup retention | 35d hot + 12mo cold (object storage) | Per `policy/data-residency.md` retention table |
| Backup encryption | per-pack KMS | OpenBao + OCI Vault |

## Cross-Pack Traffic: Forbidden by default

Per `policy/data-residency.md`. Exceptions only with active SCC + audit-chain emission.

## CDN Edge Posture

Per-pack OCI CDN configuration:
- Static assets (form-builder WASM, form-renderer assets, design-system primitives) — global edges.
- Per-tenant form rendering — pack-resident edges only (CSP + cache-key includes tenant_id + pack).
- No cross-pack edge sharing for tenant-scoped data.

## WAF Posture

Per-pack OCI WAF configuration; managed rules + custom rules:
- Per-IP rate-limit (default 10 RPS; bursts to 50).
- Per-tenant + per-form rate-limit at Istio + WAF.
- OWASP Core Rule Set (CRS) v4.0.
- Captcha challenge issued on rate-limit OR bot-score threshold.

## Failover Sequence (active-passive packs)

1. Primary region health probe fails ≥ 60s.
2. Istio gateway WAN failover routes traffic to DR region.
3. Citus DR replica promoted to primary (≤ 60s WAL catch-up).
4. Valkey Sentinel re-elects.
5. Meilisearch DR replica online (read-only during catch-up).
6. SLO burn-rate alert escalates if RTO exceeded.
7. Failback when primary healthy + reconciled.

## BCDR Drill Cadence

- Quarterly: induce Postgres primary failover.
- Quarterly: induce CDN edge failover.
- Annually: full DR drill (cross-AZ within pack).
- Cross-pack restore NEVER drilled (forbidden).

## CDN Cache Key

Per-tenant cache key (`Cache-Tag: tenant=<id>; form=<id>; lang=<locale>; pack=<pack>`); invalidation on every form publish + retention-expiry.

## Region Activation Gating

A new pack activation requires:
1. Tenant SCC / BAA / DPA on file.
2. Pack-specific overlay manifests at `iac/kustomize/overlays/pack-<pack>/`.
3. Pack-specific Cedar policy overlay.
4. Pack-resident LLM provider configured at foundry-providers.
5. Pack-resident captcha provider configured.
6. Annual compliance review per ADR-0133.
7. DPIA per-tenant on first-tenant-in-pack.

## Verification

- `cargo run -p oya-dev-cli -- gate validate forms-pack-routing-conformance`
- Quarterly chaos drill ledger
- Annual DR audit

## References

- `policy/data-residency.md`.
- `capacity-model.md`.
- `compliance.md`.
- ADR-0117 cloud-native infra.
- ADR-0131 per-microservice flat layout.
- OCI regions documentation.
