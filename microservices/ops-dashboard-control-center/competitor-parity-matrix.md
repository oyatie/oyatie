---
doc_class: Competitor-Parity-Matrix
status: accepted
date: 2026-05-20
owner: ops-sre-reliability
related_adrs:
  - ADR-0242
  - ADR-0243
  - ADR-0248
companion_docs:
  - microservices/ops-dashboard-control-center/PRD.md
  - microservices/ops-dashboard-control-center/ARCHITECTURE.md
planned_enforcement_ref: oya-governance-microservice-doc-suite
---

# Competitor Parity Matrix — ops-dashboard-control-center

Internal ops dashboard surface. Hyperscaler and industry precedents.

| Capability | AWS Internal Console | Stripe Internal Admin | Backstage Portal | OpsLevel | Port | oyatie ops-dashboard | Gap / Advantage |
|---|---|---|---|---|---|---|---|
| Per-action IAM/Cedar gate | ✓ IAM | ✓ (RBAC + audit) | ✗ (plugin-based) | ✗ | ✗ | ✓ Cedar v4.2 LTS default-deny | Advantage: Cedar is more expressive than IAM; contextual attributes (step-up class, audit_emission_confirmed) |
| Step-up auth on mutations | ✓ (MFA re-auth) | ✓ (MFA step-up) | ✗ | ✗ | ✗ | ✓ T2/T3 classes + hardware key quorum-2 | Parity + advantage on quorum-2 Cedar publish |
| Audit chain seal | ✓ CloudTrail | ✓ Stripe audit log | ✗ | ✗ | ✗ | ✓ Merkle seal per ADR-0028 + ADR-0263 | Parity |
| Tenant scope enforcement | ✓ (Org Units) | ✓ (Connect accounts) | ✗ | Partial | ✗ | ✓ RLS + Cedar + partner-agency sub-tenant | Advantage: Cedar cross-tenant FORBID + RLS defence-in-depth |
| UEBA / insider risk PRIMARY | ✓ AWS GuardDuty | Partial | ✗ | ✗ | ✗ | ✓ PRIMARY surface; UEBA baseline 90d; session recording | Parity with AWS; advantage over all others |
| Dark-mode default | ✓ (opt-in) | Partial | ✓ | ✓ | ✓ | ✓ default; high-contrast WCAG 2.2 AA | Parity + WCAG 2.2 AA advantage |
| Keyboard-driven (⌘K palette) | Partial | Partial | ✓ | Partial | ✓ | ✓ G→X shortcuts + ⌘K palette | Parity with Backstage/Port |
| Emergency-services bypass | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ LIFE-SAFETY HARD RULE; ADR-0298 | Unique advantage |
| PQC + ECH transport | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ X25519MLKEM768 + ECH RFC 9460 | Unique advantage |
| Multi-region sovereign cells | ✓ AWS Regions | Partial | ✗ | ✗ | ✗ | ✓ 4 cells + pack-level data-residency hard-stop | Parity with AWS |
| Evidence-pack export | Partial (CloudTrail) | Partial | ✗ | ✗ | ✗ | ✓ signed JSONL zstd + cosign | Advantage: offline-verifiable |
| Cedar fragment lifecycle (soak) | ✗ (SCP no soak) | ✗ | ✗ | ✗ | ✗ | ✓ ≥60s soak + quarantine + quorum-2 publish | Unique advantage vs AWS SCPs |
| Marketplace surfaces | ✗ | ✗ | ✓ plugins | ✓ scorecard | ✓ | ✗ (internal only; N/A) | N/A for internal tool |

## Not targeting parity with

- **PagerDuty / OpsGenie**: incident paging is integrated (on-call handoff), not replaced. oyatie ops-dashboard surfaces the ops control surface on top of PagerDuty.
- **Jira / Linear**: incident records are ops-dashboard-native but ticket integration is an adapter concern (IP-020).
- **Terraform Cloud / ArgoCD UI**: deployment execution is ArgoCD; this surface provides the approval gate + audit trail, not the executor.
