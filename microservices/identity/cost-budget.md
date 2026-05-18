---
doc_class: CostBudget
template_id: TPL-COST-BUDGET
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-finops
horizon: 5y (2026 → 2031)
related_adrs: [ADR-0174, ADR-0187]
---

# Cost Budget — identity µservice (5-year TCO)

All figures USD. Per-pack costs aggregated across the 11 regulatory packs (kr, eu, us, us-healthcare, jp, sg, au, in, br, ae, ksa). Assumes 100K total users in year-1, growing to 10M by year-5.

## Phase 0 (Zitadel-via-adapter) — 2026-2027

### Per-pack compute

| Component | Sizing year-1 | Cost / month / pack | Notes |
|---|---|---|---|
| Zitadel pods (Go binary) | 3 replicas, 2 vCPU + 4GB each | $200 | KEDA-autoscale; floor 3 (HA) |
| Postgres event-store (pgcat-pooled) | db.r6g.xlarge (4 vCPU + 32GB) + read replica | $480 | per ADR-0179-pgcat |
| OpenBao secrets | shared with cloud-secrets pack | $0 (shared) | OPEX in cloud-secrets µservice |
| FIDO-MDS3 refresh worker | 0.25 vCPU + 256MB | $20 | runs hourly |
| HRIS adapter poller | 0.5 vCPU + 512MB | $40 | runs every 15min |
| Audit emitter | 0.5 vCPU + 512MB | $40 | per emission ≤ 300ms |
| **Per-pack subtotal** | | **$780/mo** | |

11 packs × $780 = **$8,580/mo** = **$103K/yr** at year-1.

### Per-pack network

| Item | Cost / month / pack | Notes |
|---|---|---|
| Envoy Gateway edge bytes-in | $50 | DDoS-shaped; modest in year-1 |
| Postgres replication (read-replica) | $20 | within pack only |
| **Per-pack network subtotal** | **$70/mo** | |

11 packs × $70 = **$770/mo** = **$9.2K/yr**.

### Storage

| Item | Year-1 GB | Cost / month / pack |
|---|---|---|
| Postgres event-store (10KB/user × 100K) | 1GB | $10 |
| Audit-chain seal artefacts (forwarded to audit-chain µservice) | $0 (offloaded) | |
| WebAuthn credentials (1KB/cred × 100K avg 2 creds) | 0.2GB | $5 |
| FIDO-MDS3 metadata cache | 0.5GB | $5 |
| **Per-pack storage subtotal** | | **$20/mo** |

11 packs × $20 = **$220/mo** = **$2.6K/yr**.

### HSM partition costs (regulated packs only)

Pack-kr, pack-eu, pack-us-healthcare, pack-ksa, pack-ae:

- AWS CloudHSM ev2: $1.40/hr × 730 hr/mo × 5 packs = **$5,110/mo** = **$61.3K/yr**.
- Thales Luna (pack-kr only, KR-FSS preference): +$2K/yr for partition lease.

### Personnel (FTE allocation)

| Role | FTE alloc | Annual cost (loaded) | Notes |
|---|---|---|---|
| axis-identity engineer | 2.0 | $480K | Rust kernel + adapter authoring |
| ops-security engineer | 0.5 | $120K | runbook ownership + DR drills |
| council-compliance reviewer | 0.25 | $80K | DPIA / SOC 2 / ISO 27001 |
| **Personnel subtotal year-1** | | **$680K** | |

### Year-1 total

| Bucket | Annual cost |
|---|---|
| Compute (Zitadel + workers) | $103K |
| Network | $9.2K |
| Storage | $2.6K |
| HSM partitions | $63.3K |
| Personnel | $680K |
| **Phase 0 year-1 total** | **$858K** |

### Year-2 (200K users)

Compute scales linearly with read-replica Postgres; HSM stays flat; personnel +0.5 FTE.

| Bucket | Annual cost |
|---|---|
| Compute | $206K |
| Network | $18K |
| Storage | $5K |
| HSM | $63K |
| Personnel | $800K |
| **Year-2 total** | **$1.09M** |

## Phase 2 (in-house `oya-identity-server`) — 2027-2031

Trigger: ≥10K tenants OR multi-region active-active needed. Estimated trigger: late 2027 / early 2028.

### One-time engineering cost (FTE budget for the swap)

| Component | FTE-quarters | Cost |
|---|---|---|
| `oya-identity-server` OIDC issuer implementation | 4 | $480K |
| WebAuthn relying-party in-house (already kernel-traited; bind real impl) | 1 | $120K |
| SAML 2.0 IdP server | 2 | $240K |
| SCIM 2.0 server (already in-house at kernel; bind to in-house store) | 1 | $120K |
| Event-store schema in-house (Postgres + outbox) | 2 | $240K |
| Zitadel → oya-identity-server migration tooling | 1 | $120K |
| Pen test + SOC 2 surface re-audit | 2 | $240K |
| **One-time Phase-2 transition cost** | **13 FTE-quarters** | **$1.56M** |

### Phase-2 steady state (year-3 to year-5)

| Bucket | Year-3 annual | Year-4 annual | Year-5 annual |
|---|---|---|---|
| Compute (drop ~30% from Zitadel-equivalent due to event-store integration in same Postgres) | $300K | $500K | $800K |
| Network | $40K | $80K | $160K |
| Storage | $10K | $20K | $50K |
| HSM | $80K | $100K | $120K |
| Personnel | $900K | $950K | $1.0M |
| Phase-2 transition amortised | $520K | $520K | $520K |
| **Annual total** | **$1.85M** | **$2.17M** | **$2.65M** |

## 5-year TCO summary

| Year | Total cost | Cumulative |
|---|---|---|
| 2026 | $858K | $858K |
| 2027 | $1.09M | $1.95M |
| 2028 | $1.85M (Phase 2 begins) | $3.80M |
| 2029 | $2.17M | $5.97M |
| 2030 | $2.65M | $8.62M |
| **5-year cumulative** | | **$8.62M** |

### Vendor lock-in cost avoidance

If we DID lock in to Auth0 / Okta Workforce / Microsoft Entra ID External Identities (per ADR-0173 §competitor comparison):

| Vendor | Per-user pricing | Year-5 MAU 10M | Annual cost year-5 |
|---|---|---|---|
| Auth0 (Enterprise) | $0.023/MAU | 10M | $2.76M |
| Okta Workforce (B2C add-on) | $0.020/MAU | 10M | $2.40M |
| Microsoft Entra ID External | $0.0325/MAU | 10M | $3.90M |

Plus per-MFA cost ($0.03/auth event) which for 10M users × 4 auths/month × 12 months × $0.03 = $14.4M/yr at AAL2-level.

**Vendor lock-in avoidance over 5-year horizon: ~$50M** — and that's BEFORE counting the air-gapped sovereign packs that none of those vendors can serve.

## Sensitivities

| Sensitivity | Year-5 impact |
|---|---|
| 10× user growth (100M users) | compute +5× (HSM stays linear with packs not users); +$2M/yr |
| 4 additional regulatory packs (e.g., id, tr, vn, mx) | +$0.4M/yr Zitadel HA + HSM |
| HSM type change (Thales Luna for all regulated) | +$60K/yr |
| Phase-2 trigger delays to year-4 | -$1.56M (transition cost amortised later); +Zitadel licence renewal sensitivity |
| Pen test finding requires extra engineering quarter | +$120K one-time |
