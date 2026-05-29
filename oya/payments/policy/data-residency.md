---
doc_class: PolicyDataResidency
template_id: TPL-DATA-RESIDENCY
microservice: payments
status: Accepted
date: 2026-05-20
owner_team: ops-security + ops-compliance + council-privacy
related_adrs: [ADR-0244, ADR-0248, ADR-0251]
companion_docs:
  - microservices/payments/ARCHITECTURE.md
  - microservices/payments/multi-region.md
  - microservices/payments/compliance.md
diataxis_quadrant: reference
doc_status: published
---

# Data Residency — payments µservice

> Per-jurisdiction data-residency rules. CN-PIPL is a hard constraint; all others are tenant-policy-configurable with default per-region cell-pinning.

---

## §1. Jurisdiction residency table

| Jurisdiction | Pack | Residency rule | Hard or soft? |
|---|---|---|---|
| KR | `pack-kr-fss` | KR cell only (KR-PIPA Art. 39 + KR-EFTA) | **Hard** for raw financial records (`kr-central-1`). |
| EU | `pack-eu-psd2-sca` | EU cell only (GDPR Art. 44-50 + EU SCC) | **Hard** for raw PII; soft for aggregates. |
| US | `pack-us-state-mtl` | Per-state preference; default no constraint | Soft (tenant-policy). |
| CN | `pack-cn-pipl-2021` | CN cell only; **NO cross-border egress** | **Hard** — Cedar FORBID on egress. |
| JP | `pack-jp-appi` | JP cell preferred | Soft. |
| SG | `pack-sg-pdpa` | SG cell preferred | Soft. |
| AU | `pack-au-aml-ctf` | AU cell preferred | Soft. |
| IN | `pack-in-rbi` | IN cell only (RBI data-localisation 2018) | **Hard** for payment data. |
| BR | `pack-br-lgpd-finance` | BR cell preferred | Soft. |
| AE / KSA | `pack-ae-pdpl` / `pack-ksa-pdpl` | Regional preference | Soft. |

## §2. Per-data-class residency rules

| Data class | Residency rule | Why |
|---|---|---|
| `charges` (ledger) | Pin to tenant's `region_pin` cell. | Financial-record sovereignty. |
| `payment_methods` (tokenised refs) | Pin to tenant's `region_pin` cell. | Linked to charges. |
| `payouts` | Pin to tenant's `region_pin` cell. | Financial-record sovereignty. |
| `sub_merchants` (KYB-doc-hashes) | Pin to tenant's `region_pin` cell. | KYC residency. |
| `audit-chain seals` | Per-cell; Merkle-root replicated to `governance` µservice with redaction. | Audit-trail sovereignty + cross-region attestation. |
| `dispute-evidence` (object-storage) | Pin to tenant's `region_pin` cell. | PII contained in evidence; subject's regional rights. |
| `reconciliation reports` | Pin to tenant's `region_pin` cell. | Financial-record sovereignty. |
| `aggregated metrics` (cross-tenant) | Aggregate cell with DP-noise. | Cross-tenant aggregation requires privacy-preserving compute. |
| `PSP credentials` (OpenBao) | Pin to tenant's `region_pin` cell. | Credential sovereignty + lower-blast-radius. |
| `read replicas` | Cross-region permitted with eventual-consistency. | Operator UX for global tenants. |

## §3. Cedar fragment

Data-residency enforcement is implemented as a Cedar fragment (`data-residency.cedar`) that overlays every action:

```cedar
// FORBID — cross-border egress from CN-PIPL pack-marked tenants
forbid (
  principal,
  action,
  resource
)
when {
  resource has tenant_id &&
  resource has compliance_packs &&
  "pack-cn-pipl-2021" in resource.compliance_packs &&
  context has caller_region &&
  context.caller_region != "cn"
};

// FORBID — cross-border egress from KR-marked tenants (raw financial records)
forbid (
  principal,
  action in [
    Action::"charge.read",
    Action::"payout.read",
    Action::"refund.read",
    Action::"settlement.read"
  ],
  resource
)
when {
  resource has compliance_packs &&
  "pack-kr-fss" in resource.compliance_packs &&
  resource has data_class &&
  resource.data_class == "RAW_FINANCIAL_RECORD" &&
  context has caller_region &&
  context.caller_region != "kr"
};
```

(Full Cedar fragment is committed alongside; this `.md` is the prose explainer per documentation-rigor.md §2.)

## §4. Cross-region read-replica policy

| Replica purpose | Allowed across regions? |
|---|---|
| Operator dashboard reads | Yes (within EU<->US, within Asia-Pacific; never to/from CN) |
| Tenant API reads | Yes (same restrictions) |
| ETL to analytics platform | Yes for aggregated only (no raw PII); never from CN |
| Backup snapshots | Per-region within continent only; CN to CN-DR only |

## §5. Sub-processors per region

| Sub-processor | Used in | Data classes flowing | DPA / SCC |
|---|---|---|---|
| Stripe US | us / eu-via-Stripe-EU / global | tokenised-charge | DPA signed; SCC for EU→US |
| Adyen EU | eu / au-via-Adyen | tokenised-charge | DPA signed |
| Toss KR | kr | charge + payout | KR-PIPA processor agreement signed |
| KakaoPay KR | kr | charge | KR-PIPA processor agreement |
| LINE Pay JP/TW/TH | jp/tw/th | charge | APPI processor agreement |
| WeChat / Alipay CN | cn | charge | PIPL processor agreement; no cross-border |

## §6. Configuration

Tenant configures residency at onboarding via tenant manifest:

```yaml
# tenant manifest excerpt
tenant_id: tenant_acme
region_pin: eu-west-1
compliance_packs:
  - pack-pci-dss-l1-v4
  - pack-eu-psd2-sca
  - pack-gdpr
data_residency:
  raw_financial_records: eu-only
  metrics_aggregation: eu-or-global-dp-noise
  read_replicas: eu-and-us-and-asia-pacific
```

The `oya-payments-charge-rest` layer reads tenant config + caller context + applies Cedar gate.

## §7. Drift detection

The `oya-governance-data-residency` CI lane verifies:

- Every charges row's `_region_pin_validated=true` flag is set.
- Cross-region object-storage replication policies match tenant residency.
- DR-pairs satisfy per-region residency.
- Object-storage bucket-policies block cross-jurisdiction reads from disallowed regions.

## §8. References

- [`ARCHITECTURE.md`](../ARCHITECTURE.md).
- [`multi-region.md`](../multi-region.md).
- [`compliance.md`](../compliance.md).
- [ADR-0244 — tenant scoping](../../../docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md).
- [ADR-0248 — cellular architecture](../../../docs/decisions/ADR-0248-amazon-shape-cellular-architecture.md).
- [ADR-0251 — compliance packs](../../../docs/decisions/ADR-0251-compliance-pack-primitive.md).
- GDPR Art. 44-50 — cross-border data transfer.
- PIPL Arts. 38-43 — cross-border restrictions.
- RBI data-localisation 2018 — RBI/2017-18/153.
