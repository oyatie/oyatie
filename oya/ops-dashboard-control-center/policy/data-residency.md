---
doc_class: Policy
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0244
  - ADR-0248
  - ADR-0251
companion_docs:
  - microservices/ops-dashboard-control-center/compliance.md
  - microservices/ops-dashboard-control-center/multi-region.md
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
planned_enforcement_ref: oya-governance-adr-adherence-matrix
---

# Data Residency Policy — ops-dashboard-control-center

## §1 Residency hard-stops

Per ADR-0248 sovereign-cell model and ADR-0251 compliance-pack overlays:

| Pack | Region | Hard-stop | Mechanism |
|---|---|---|---|
| `oya-pack-eu` | `eu-west-1` | Audit log + operator PII MUST NOT leave EU | Cedar data-residency gate; Kafka MirrorMaker 2 topic-level ACL blocks cross-region replication for EU-origin events |
| `oya-pack-kr` | `ap-northeast-2` | K-ISMS: access log MUST stay in KR | Separate ClickHouse cluster in KR cell; no cross-region replication of KR audit logs |
| `oya-pack-us` (FedRAMP) | `us-east-1` US-Gov zone | FedRAMP data must not leave US jurisdiction | US-Gov zone cell; separate Kafka cluster; no cross-border topics |
| `oya-pack-jp` | `ap-northeast-1` | APPI: personal data stays in JP | Per-cell audit ClickHouse; no replication to other regions |
| All packs | Home region | Default: operator audit log stays in operator's home-region cell | Cross-region replication of read-path state only (cluster health signals, posture snapshots); audit logs never cross-region |

## §2 Cross-region audit log export

Cross-region export of audit logs requires:
1. Explicit legal basis documented in DPIA.
2. Per-pack transfer mechanism (e.g., GDPR Art. 46 SCCs for EU→US; KR-PIPA Art. 17 for KR→US).
3. Cedar data-residency gate PERMIT with `cross_region_transfer_legal_basis` context field set.
4. Audit event `DataResidencyEnforced` emitted with justification.

## §3 Enforcement

Cedar fragment `policy/cedar/admin-action-authorization.cedar` includes data-residency enforcement inline: `DataResidencyEnforce` action class evaluated on every cross-region data access. IaC: `iac/prod-network-policy.yaml` Cilium L4 policy blocks cross-region Kafka topic writes for sovereign cells.

## §4 Portability

Per ADR-0276: per-tenant export in signed JSONL zstd. Export scoped to operator's home region. Cross-region portability requires legal basis as above.
