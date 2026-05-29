---
doc_class: Dashboard
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0251
  - ADR-0159
companion_docs:
  - microservices/feature-flags/compliance.md
  - microservices/feature-flags/dashboards/flag-state-overview.json
planned_enforcement_ref: oya-governance-pack-overlay-coverage
---

# Pack Override Coverage Dashboard

## Purpose

Cross-reference document for the `flag-state-overview.json` "Pack Overrides Active" panel. Lists the expected pack override roster per pack ID and verifies coverage completeness.

## Expected pack overrides per pack

| Pack ID | Flag key | Expected value | CI verification |
|---|---|---|---|
| `us-healthcare` | `phi-exposure-flag` | `off` | `oya-governance-pack-overlay-coverage` |
| `us-healthcare` | `ehr-auto-share-flag` | `off` | `oya-governance-pack-overlay-coverage` |
| `pci-dss` | `raw-pan-display` | `off` | `oya-governance-pack-overlay-coverage` |
| `pci-dss` | `cvv-retention-flag` | `off` | `oya-governance-pack-overlay-coverage` |
| `eu-ai-act` | `high-risk-ai-auto-decide` | `off` | `oya-governance-pack-overlay-coverage` |
| `eu-ai-act` | `ai-profiling-unrestricted` | `off` | `oya-governance-pack-overlay-coverage` |
| `fedramp-high` | `external-api-unrestricted` | `off` | `oya-governance-pack-overlay-coverage` |
| `fedramp-high` | `debug-mode-flag` | `off` | `oya-governance-pack-overlay-coverage` |
| `kr-fss` | `instant-large-transfer-flag` | `off` | `oya-governance-pack-overlay-coverage` |
| `kr-fss` | `cross-border-payment-flag` | `off` (until KYB) | `oya-governance-pack-overlay-coverage` |
| `gdpr-eu` | `cookie-analytics-default-on` | `off` | `oya-governance-pack-overlay-coverage` |
| `gdpr-eu` | `behavioral-profiling-flag` | `off` (until consent) | `oya-governance-pack-overlay-coverage` |

## Coverage metrics

- `oya_feature_flag_pack_override_total{pack_id, flag_key}` — count of tenants with each override active.
- `oya_feature_flag_pack_override_missing_total` — count of tenants where an expected override is NOT applied (should be 0).
- Alert: `oya_feature_flag_pack_override_missing_total > 0` → `pack-override-cascade.md` runbook.

## Audit trail

Every `PackFlagOverrideApplied` event is sealed per ADR-0028 and accessible via `runbooks/audit-replay.md`. QSA auditors (PCI DSS, SOC 2) may pull the full override history for their audit window.
