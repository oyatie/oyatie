---
doc_class: MultiRegion
template_id: TPL-MULTI-REGION
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: axis-payments + ops-sre-reliability
related_adrs: [ADR-0244, ADR-0248, ADR-0251, ADR-0252, ADR-0253]
companion_docs:
  - microservices/payments/PRD.md
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/compliance.md
  - microservices/payments/capacity-model.md
diataxis_quadrant: reference
doc_status: published
---

# Multi-Region — payments µservice

> Per-region PSP routing, cell-tier mapping, data-residency rules, DR / failover. CN-PIPL data-localisation enforced as a hard constraint.

---

## §1. Region map

| Region code | Geography | PSPs | Cell tier | Currencies (primary) |
|---|---|---|---|---|
| `us-east-1` | US East (N. Virginia) | Stripe US | Tier-2 | USD |
| `us-west-2` | US West (Oregon) | Stripe US | Tier-2 | USD |
| `us-central-fin-1` | US regulated-finance cell | Stripe US | Tier-1 | USD |
| `eu-west-1` | EU West (Ireland) | Adyen EU + Stripe EU | Tier-2 | EUR, GBP, CHF, SEK, DKK, NOK |
| `eu-central-1` | EU Central (Frankfurt) | Adyen EU + Stripe EU | Tier-2 | EUR, PLN, CZK |
| `eu-fin-1` | EU regulated-finance cell | Adyen EU | Tier-1 | EUR |
| `kr-central-1` | KR (Seoul) | Toss Payments + KakaoPay | Tier-1 | KRW |
| `jp-east-1` | JP (Tokyo) | LINE Pay + Stripe JP | Tier-2 | JPY |
| `sg-1` | SG (Singapore) | Stripe SG + LINE Pay SG | Tier-2 | SGD, MYR, THB |
| `au-1` | AU (Sydney) | Stripe AU + Adyen AU | Tier-2 | AUD, NZD |
| `in-1` | IN (Mumbai) | Stripe IN + Razorpay (post-MVP) | Tier-2 | INR |
| `br-1` | BR (São Paulo) | Stripe BR + Adyen BR | Tier-2 | BRL |
| `ae-1` | AE (Dubai) | Stripe ME | Tier-2 | AED |
| `ksa-1` | KSA (Riyadh) | Stripe SA (post-MVP) | Tier-2 | SAR |
| `cn-1` | CN (Shanghai, separate mainland cloud per PIPL) | WeChat Pay + Alipay | Tier-1 | CNY |

## §2. Per-region PSP routing rules

Routing decision is made by `oya-payments-charge-usecase` at charge-creation time, evaluating the tuple `(tenant.region_pin, charge.currency, charge.payment_method_kind, tenant.psp_preference, tenant.compliance_packs)`.

### 2.1 US flows

- Default PSP: Stripe US.
- Fallback (if tenant policy permits): Adyen US.
- Tier-1 cell (Tier-1 regulated finance): only Stripe US (PCI scope contained).

### 2.2 EU flows

- Default PSP: Adyen EU (interchange-plus pricing advantage).
- Fallback: Stripe EU.
- SCA (PSD2 RTS) enforced at edge per tenant policy.

### 2.3 KR flows

- Domestic: Toss Payments (cards) + KakaoPay (wallet).
- Tenant CHOICE: declare PSP preference at onboarding.
- KR-FSS oversight: audit-trail pull per [`runbooks/kr-fss-audit-pull.md`](runbooks/kr-fss-audit-pull.md).

### 2.4 JP / TW / TH flows

- LINE Pay (wallet) + Stripe JP (cards).
- LINE Pay audience: B2C-personal default; Stripe for B2B-work.

### 2.5 CN flows (PIPL hard constraint)

- WeChat Pay + Alipay only.
- **Hard constraint**: data **never leaves** the CN-cell. No cross-border egress under any circumstance.
- Tenant config: CN-tenant operators must accept the PIPL-cross-border-no clause at onboarding.
- Cedar gate `policy/data-residency.md` enforces.

### 2.6 SG / AU / IN / BR / AE / KSA flows

- Stripe + Adyen (where available) as default; per-region wallet adapters added per Wave-4.

## §3. Tier-1 vs Tier-2 cell deployment

Per ADR-0248 cellular architecture:

| Tier | Use | Properties |
|---|---|---|
| Tier-0 | Edge (TLS + WAF + Bot-Mgmt) | Stateless; CDN-edge POPs. |
| Tier-1 | Regulated-finance | Cloud Hypervisor + Kata pods; per-cell HSM; per-cell PCI scope. |
| Tier-2 | Default product | Kata pods; shared regional infra. |
| Tier-3 | Data-hardened (analytics) | Locked-down; cross-tenant aggregation with DP-noise. |

**Tenant cell-tier assignment**:

- Tenant with `compliance_packs[]` including `pack-pci-dss-l1-v4` → Tier-1 cell (US-fin-1 / EU-fin-1 / KR-central-1).
- Tenant default → Tier-2 cell (region-pinned per tenant.region_pin).

## §4. Per-region currency support

Currencies supported per region (ISO 4217):

| Region | Currencies |
|---|---|
| `us-east-1` / `us-west-2` / `us-central-fin-1` | USD, CAD, MXN |
| `eu-west-1` / `eu-central-1` / `eu-fin-1` | EUR, GBP, CHF, SEK, DKK, NOK, PLN, CZK, HUF, RON, BGN |
| `kr-central-1` | KRW |
| `jp-east-1` | JPY |
| `sg-1` | SGD, MYR, THB, IDR, PHP, VND |
| `au-1` | AUD, NZD |
| `in-1` | INR |
| `br-1` | BRL |
| `ae-1` | AED |
| `ksa-1` | SAR |
| `cn-1` | CNY (no cross-border) |

## §5. Data-residency rules (per ADR-0244 + ADR-0251)

| Data class | Residency rule |
|---|---|
| Charges (ledger) | Pinned to tenant's region_pin cell; no cross-region replication except DR pair. |
| KYB documents (raw) | Stays in PSP's region (we don't store raw; PSP holds). |
| KYB document hashes | Pinned to tenant's region_pin. |
| PSP credentials (OpenBao) | Pinned to tenant's region_pin cell. |
| Audit-chain seals | Per-cell; Merkle-root replicated to `governance` µservice with redaction. |
| Dispute evidence | Per-cell object-storage; SSE-KMS-tenant. |
| Reconciliation reports | Per-cell. |
| Aggregated metrics (cross-tenant) | Aggregate cell with DP-noise; never raw cross-cell. |

## §6. DR / failover

### 6.1 DR pairs

Per ADR-0248 shuffle sharding, each Tier-1 cell has a DR pair in a different fault domain:

| Primary | DR pair |
|---|---|
| `us-central-fin-1` | `us-west-fin-2` |
| `eu-fin-1` | `eu-north-fin-2` |
| `kr-central-1` | `kr-south-2` (per KR-DR-strategy) |
| `cn-1` | `cn-2` (within CN-mainland; PIPL satisfied) |

Tier-2 cells use cross-region RPO=15min / RTO=1h failover.

### 6.2 DR-failover procedure

| Step | Action | Time budget |
|---|---|---:|
| 1 | Detect primary cell unreachable (≥5 min) | 5 min |
| 2 | On-call confirms with PSP-availability (rule-out PSP issue) | 5 min |
| 3 | Declare Sev-1 + open `#inc-payments-<id>` | immediate |
| 4 | Activate DR cell: flip cell-router to DR | 2 min |
| 5 | Verify CRDB DR replica current (rpo ≤15min) | 5 min |
| 6 | Verify OpenBao DR sealed-state recovered | 5 min |
| 7 | Run smoke-test (synthetic charge / refund / payout) | 10 min |
| 8 | Notify tenants (status page + per-tenant webhook) | 10 min |
| 9 | Resume operations | — |
| 10 | Post-incident review within 5 business days | — |
| **Total RTO** | | **~35 min** |

### 6.3 Failback procedure

Once primary cell is healthy:

| Step | Action | Time budget |
|---|---|---:|
| 1 | Verify primary cell green for ≥1 h | 1 h |
| 2 | Reconcile DR-cell writes back to primary | 30 min |
| 3 | Quiesce DR cell (drain in-flight) | 30 min |
| 4 | Flip cell-router back to primary | 2 min |
| 5 | Smoke-test | 10 min |
| 6 | DR cell goes back to standby | — |

## §7. Multi-region read-replica pattern

For tenant operators viewing dashboards from outside their tenant's region_pin (e.g., a Global tenant's NYC-based ops viewing EU charges), we expose a **read-replica** with eventual consistency:

- Replica lag p99 ≤5s.
- Read-replica enforces same Cedar gate as primary.
- Writes always go to primary (regardless of caller region).

## §8. Cross-region requests

| Caller region | Target region | Behaviour |
|---|---|---|
| Same region | Same | Direct, sub-region latency. |
| Different region (same continent) | Different | Cross-cell read-replica or DR-pair route. |
| Cross-continent | Cross-continent | Read-replica only; writes route via tenant.region_pin (HTTP/3 long-haul). |
| CN ↔ non-CN | Blocked | PIPL hard constraint; Cedar `data-residency` FORBID. |

## §9. Latency budgets

| Path | p50 | p99 |
|---|---:|---:|
| Same-cell write | 50ms | 250ms |
| Cross-region read (same continent) | 100ms | 500ms |
| Cross-region read (different continent) | 250ms | 1000ms |
| DR-failover detect + flip | 5min | 7min |

## §10. References

- [`ARCHITECTURE.md`](ARCHITECTURE.md).
- [`capacity-model.md`](capacity-model.md).
- [`compliance.md`](compliance.md) — pack-overlay residency rules.
- [`runbooks/psp-outage.md`](runbooks/psp-outage.md).
- [`policy/data-residency.md`](policy/data-residency.md).
- [ADR-0248 — cellular architecture](../../docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md).
- [ADR-0251 — compliance packs](../../docs/decisions/ADR-0251-compliance-pack-primitive.md).
- [ADR-0252 — HLC + TrueTime](../../docs/decisions/ADR-0252-hlc-default-truetime-tier.md).
- [ADR-0253 — HTTP/3 + QUIC default](../../docs/decisions/ADR-0253-http3-quic-default-protocol.md).
- AWS shuffle-sharding pattern — `aws.amazon.com/blogs/architecture/shuffle-sharding`.
