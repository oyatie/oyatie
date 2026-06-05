---
doc_class: PolicyContract
template_id: TPL-POLICY
microservice: tasks
policy_id: POLICY-data-residency
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-tasks
related_adrs: [ADR-0117, ADR-0140 (retired per ADR-0145)]
doc_status: published
---

# Data Residency Policy — tasks µservice

## Purpose

Define where tasks data may be stored, processed, and replicated; how cross-border transfer is gated; and how each jurisdiction-pack's residency invariant is enforced. Tasks differs from calendar in that there is **no cross-tenant grant mechanism** — tasks are an internal-to-tenant primitive, so cross-pack data flow only happens via cross-µservice handoffs (workflow-engine bridge, calendar due-date bridge, mail/messenger inbound).

## Residency Invariants

### Invariant DR-01 — Pack-pinned primary storage

> Every tenant's task-store + project-list + dependency-edge Postgres rows + Valkey view-cache + Meilisearch search-index live in exactly one `pack-<jurisdiction>` cluster. The pack is determined at tenant onboarding and pinned in the `tenant_registry` (owned by `tenancy` µservice).

| Pack | Region | Substrate |
|---|---|---|
| pack-kr | OCI ap-seoul-1 | Postgres + Valkey + Meilisearch cluster KR-resident |
| pack-eu | OCI eu-frankfurt-1 | Postgres + Valkey + Meilisearch cluster EU-resident |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 | Postgres + Valkey + Meilisearch cluster US-resident |
| pack-us-healthcare | OCI us-ashburn-1 (BAA-eligible) | Postgres + Valkey + Meilisearch cluster US-resident, HIPAA-compliant |
| pack-jp | OCI ap-tokyo-1 | Postgres + Valkey + Meilisearch cluster JP-resident |
| pack-sg | OCI ap-singapore-1 | Postgres + Valkey + Meilisearch cluster SG-resident |
| pack-au | OCI ap-sydney-1 | Postgres + Valkey + Meilisearch cluster AU-resident |
| pack-in | OCI ap-mumbai-1 | Postgres + Valkey + Meilisearch cluster IN-resident |
| pack-br | OCI sa-saopaulo-1 | Postgres + Valkey + Meilisearch cluster BR-resident |
| pack-ae | OCI me-dubai-1 | Postgres + Valkey + Meilisearch cluster AE-resident |
| pack-ksa | OCI me-jeddah-1 | Postgres + Valkey + Meilisearch cluster KSA-resident |

### Invariant DR-02 — No default cross-pack replication

> Cross-pack replication is FORBIDDEN by default. Postgres logical replication slots, dump/restore, Meilisearch index export, or any other cross-pack path is refused at infrastructure layer (Postgres `pg_hba.conf` + Meilisearch ACL + network policy) and at LEAN-check layer (`oya-check-cross-pack-replication-prohibition`).

### Invariant DR-03 — Cross-border transfer gated by SCC

> Cross-pack data flow (e.g., a pack-eu workflow creates a task in pack-kr via workflow-engine bridge) is permitted ONLY when:
> 1. The tenant has executed Standard Contractual Clauses (SCCs) per GDPR Arts. 44–46 (or equivalent pack-local provision: KR PIPA Art. 23-2; APPI Art. 21; LGPD Art. 33).
> 2. The transfer is recorded in `microservices/tasks/legal/transfer-register.md`.
> 3. Cross-pack handoff approved at Cedar layer per `policy/tenant-scope.cedar`.
> 4. The tenant's DPA template carries the SCC clause.

### Invariant DR-04 — Cross-µservice handoff residency

> Cross-µservice handoffs (workflow-engine bridge, calendar due-date bridge, mail/messenger inbound) carry the originating tenant's pack tag; cross-pack handoffs are SCC-gated. Each handoff event carries `pack_tag` so the receiving µservice can validate cross-pack admission.

### Invariant DR-05 — REST ingress routes by tenant pack

> The REST + gRPC ingress routes by per-tenant pack tag derived from OIDC issuer + per-tenant API-key binding. Misroute is refused (HTTP 403 + audit emission).

### Invariant DR-06 — Backup + cold storage residency

> Backup snapshots + cold-storage exports remain in the same pack as the source. Cross-region backup replication is allowed ONLY for disaster-recovery within the same jurisdiction family.

### Invariant DR-07 — Audit-chain seal storage

> Audit-chain seals emitted by tasks are persisted by the `audit-chain` µservice; that µservice's residency policy governs. Tasks inherits the constraint that audit-chain seal records for pack-A tasks remain in pack-A audit-chain cluster.

### Invariant DR-08 — Search-index residency

> Meilisearch per-tenant index lives in the tenant's pack cluster only. Search-index re-build from Postgres occurs in-pack only; cross-pack rebuild forbidden.

### Invariant DR-09 — Employment-record retention floors per pack

> Tasks involving employment-context assignment (per ADR-TASKS-0006 + Cedar admission) carry per-pack retention floors:
> - pack-kr: 1095d (근로기준법 Art. 41).
> - pack-us: 3y default (state varies; tenant override per state-law).
> - pack-eu: per GDPR Art. 5(1)(e) storage limitation + national employment law.
> - pack-jp: per Japanese Labour Standards Act.
> - pack-au: 7y per Fair Work Act 2009.
> - pack-in: 3y per Industrial Disputes Act.
> - pack-br: 5y per CLT.

## Per-Pack Detail

### pack-kr (KR PIPA + 근로기준법 + ISMS-P + 전자문서법)

- **PIPA Art. 17 (cross-border)**: default cross-pack = forbidden.
- **PIPA Art. 23-2 (sensitive cross-border)**: explicit consent at tenant-of-tenant level.
- **PIPA Art. 28-2 (data destruction)**: retention upper bounds enforced.
- **근로기준법 Art. 41 (employment records retention)**: 3y minimum for employment-context tasks → 1095d retention floor.
- **PIPC enforcement**: tenant DPA includes pack-kr addendum.

### pack-eu (GDPR + EDPB + Schrems II + EU AI Act)

- **GDPR Arts. 44–50**: SCC-only for cross-pack.
- **Schrems II**: TIA for non-adequate jurisdictions.
- **EDPB Recommendations 01/2020**: supplementary measures (encryption-in-transit + tenant-DEK at rest).
- **NIS2**: incident-reporting timelines when threshold-engaged.
- **eIDAS**: AdES via audit-chain seals.
- **EU AI Act Art. 22**: T2 auto-assign in employment-context refused at Cedar layer for pack-eu until conformity ADR.

### pack-us-healthcare (HIPAA + state)

- **45 CFR §164.502(e) BAA**: BAA-bound tenant data stays in BAA-eligible region.
- **HIPAA breach notification ≤ 60 days**: integrated.
- **State-level**: CCPA / CMIA / NY SHIELD overlays.

### pack-us (CCPA + EEOC + state AI laws)

- **CCPA / CPRA**: subject rights per §6.
- **EEOC UGESP 1978 + Title VII**: T2 auto-assign in employment-context refused at Cedar until fairness-audit.
- **NY Local Law 144 (AEDT)**: T2 refused for pack-us-NY until AEDT audit.

### pack-jp (APPI + Japanese Labour Standards Act)

- **APPI Art. 17 / Art. 21 / Art. 27**: pack-pinning + cross-border consent.
- **Japanese Labour Standards Act**: employment-context retention floor.

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack data residency at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/tasks-data-residency-overlay.md`.

## Enforcement Layers

| Layer | Mechanism | Refusal at |
|---|---|---|
| Tenant onboarding | tenancy µservice assigns + pins pack | Onboarding gate |
| Network | Postgres `pg_hba.conf` + Meilisearch ACL + Kubernetes NetworkPolicy refuses cross-pack ingress | Network |
| Application | Pack tag in OIDC + per-tenant API-key binding; ingress routes by tag | API request |
| LEAN CI | `oya-check-cross-pack-replication-prohibition`, `oya-check-pack-pinning-coverage` | PR time |
| Audit | every Workflow event carries `pack_tag`; cross-pack flows emit explicit transfer record | Per-event |

## Verification

| Check | Cadence | Owner |
|---|---|---|
| LEAN: cross-pack replication prohibition | per-PR | axis-tasks |
| LEAN: pack-pinning coverage | per-PR | axis-tasks |
| Pen-test: cross-pack routing bypass | Annually | ops-security |
| Backup-residency audit | Quarterly | ops-sre-reliability |
| SCC compliance: transfer register review | Quarterly | council-privacy |
| Search-index residency audit | Quarterly | axis-tasks |

## References

- ADR-0117: data residency.
- ADR-0140: Cedar policy.
- `multi-region.md`, `compliance.md`, `legal/transfer-register.md`, `legal/dpa-template.md`, `legal/tia-template.md`.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Arts. 17, 23-2, 28-2; 근로기준법 Art. 41.
- HIPAA 45 CFR §164.502(e); 45 CFR Part 164 Subpart D.
- APPI Arts. 17, 21, 27.
- PDPA, MAS Notice 644, APP, APRA-CPS 234, DPDPA, LGPD, UAE PDPL, KSA PDPL.
- `microservices/calendar/policy/data-residency.md` — sibling reference template.
