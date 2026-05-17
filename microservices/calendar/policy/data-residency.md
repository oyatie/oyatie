---
doc_class: PolicyContract
template_id: TPL-POLICY
microservice: calendar
policy_id: POLICY-data-residency
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-security + axis-calendar
related_adrs: [ADR-0117, ADR-0140]
doc_status: published
---

# Data Residency Policy — calendar µservice

## Purpose

Define where calendar data may be stored, processed, and replicated; how cross-border transfer is gated; and how each jurisdiction-pack's residency invariant is enforced.

## Residency Invariants

### Invariant DR-01 — Pack-pinned primary storage

> Every tenant's event-store Postgres rows live in exactly one `pack-<jurisdiction>` Postgres cluster. The pack is determined at tenant onboarding and pinned in the `tenant_registry` (owned by `tenancy` µservice).

| Pack | Region | Substrate |
|---|---|---|
| pack-kr | OCI ap-seoul-1 | Postgres + Redis cluster KR-resident |
| pack-eu | OCI eu-frankfurt-1 | Postgres + Redis cluster EU-resident |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 | Postgres + Redis cluster US-resident |
| pack-us-healthcare | OCI us-ashburn-1 (BAA-eligible) | Postgres + Redis cluster US-resident, HIPAA-compliant |
| pack-jp | OCI ap-tokyo-1 | Postgres + Redis cluster JP-resident |
| pack-sg | OCI ap-singapore-1 | Postgres + Redis cluster SG-resident |
| pack-au | OCI ap-sydney-1 | Postgres + Redis cluster AU-resident |

### Invariant DR-02 — No default cross-pack replication

> Postgres replication factor is 3 within a single region's cluster (same pack). Cross-pack replication is FORBIDDEN by default. Logical replication slots, dump/restore, or any other cross-pack path is refused at infrastructure layer (Postgres `pg_hba.conf` + network policy) and at LEAN-check layer (`oya-check-cross-pack-replication-prohibition`).

### Invariant DR-03 — Cross-border transfer gated by SCC

> Cross-pack data flow (e.g., a KR tenant scheduling a meeting with an EU attendee that requires storing EU-resident metadata in KR) is permitted ONLY when:
> 1. The tenant has executed Standard Contractual Clauses (SCCs) per GDPR Arts. 44–46 (or equivalent pack-local provision: KR PIPA Art. 23-2; APPI Art. 21; LGPD Art. 33).
> 2. The transfer is recorded in `microservices/calendar/legal/transfer-register.md`.
> 3. The transfer is invitation-bound (cross-tenant availability projection — Invariant 4 of `event-isolation.md` — applies; only free/busy crosses, never raw content).
> 4. The tenant's DPA template carries the SCC clause.

### Invariant DR-04 — Cross-tenant attendee → cross-pack metadata

> When a tenant in pack-A invites an external attendee whose `tenant_pack` differs (pack-B), the invitation chain creates:
> - In pack-A: full event record + attendee email + RSVP state.
> - In pack-B: ONLY the attendee's own RSVP record + free/busy projection.
>
> The pack-B record carries a foreign-key reference to pack-A's event, but the dereferenced join is performed at access time via the cross-tenant grant (Invariant 4); the joined record never materialises at rest in pack-B.

### Invariant DR-05 — CalDAV ingress routes by tenant pack

> The CalDAV ingress (`ics-import-export-rest` REST endpoint) routes by per-tenant pack tag derived from OIDC issuer + per-tenant API-key binding. Misroute is refused (HTTP 403 + audit emission).

### Invariant DR-06 — Backup + cold storage residency

> Backup snapshots + cold-storage exports remain in the same pack as the source. Cross-region backup replication is allowed ONLY for disaster-recovery within the same jurisdiction family (e.g., us-ashburn-1 → us-phoenix-1 both in pack-us).

### Invariant DR-07 — Audit-chain seal storage

> Audit-chain seals emitted by calendar are persisted by the `audit-chain` µservice; that µservice's residency policy governs. Calendar inherits the constraint that audit-chain seal records for pack-A events remain in pack-A audit-chain cluster.

## Per-Pack Detail

### pack-kr (KR PIPA + ISMS-P + 전자문서법)

- **PIPA Art. 17 (cross-border transfer)**: forbids cross-border transfer without explicit consent. Default cross-pack = forbidden.
- **PIPA Art. 23-2 (sensitive data cross-border)**: requires explicit consent at tenant-of-tenant level. Sensitive-flagged events never cross pack-kr.
- **PIPA Art. 28-2 (data destruction)**: retention upper bounds enforced; non-essential data destroyed within statutory minimum.
- **PIPC enforcement**: tenant DPA includes pack-kr addendum specifying KR-resident retention.
- **Korean localisation**: pack-kr ships Korean lunar-calendar holidays; locale rules for input/search.

### pack-eu (GDPR + EDPB + Schrems II)

- **GDPR Arts. 44–50 (transfers)**: SCC-only for cross-pack.
- **Schrems II**: transfer impact assessment (TIA) required when cross-pack involves non-adequate jurisdictions; TIA template at `legal/tia-template.md`.
- **EDPB Recommendations 01/2020**: supplementary measures (encryption-in-transit + tenant-DEK encryption-at-rest) implemented.
- **NIS2**: when oyatie crosses threshold, incident-reporting timelines apply.
- **eIDAS**: AdES via audit-chain seals.

### pack-us-healthcare (HIPAA + state)

- **45 CFR §164.502(e) BAA**: BAA-bound tenant data stays in BAA-eligible region.
- **HIPAA breach notification ≤ 60 days**: integrated.
- **State-level**: CCPA / CMIA / NY SHIELD overlays per `compliance.md`.

### pack-jp (APPI)

- **APPI Art. 17 (purpose)**: declared at onboarding.
- **APPI Art. 21 (cross-border)**: pack-jp JP-resident.
- **APPI Art. 27 (cross-border consent)**: explicit at onboarding.

### pack-sg (PDPA + MAS Notice 644)

- **PDPA Part IV Retention Limitation**: retention bounded per asset table.
- **PDPA Part VI Transfer Limitation**: SCC-equivalent.
- **MAS Notice 644**: for financial-services tenants.

### pack-au (Privacy Act 1988 APP)

- **APP 8 (cross-border)**: tenant-DPA includes APP 8 clause.
- **APP 11 (security)**: encryption + audit-chain satisfies.
- **APRA-CPS 234**: for financial-services tenants.

### pack-in (DPDPA 2023)

- **§16 (cross-border transfer)**: as of 2026-05, DPDPA cross-border list pending; default residency in-IN until clarified.
- **§9 (children's data)**: parental consent verification inherited from tenant.

### pack-br (LGPD)

- **Arts. 33–36 (transfers)**: ANPD-approved SCCs.
- **Art. 38 RIPD**: this DPIA + threat-model satisfies.

### pack-ae (UAE PDPL) / pack-ksa (KSA PDPL)

- **UAE PDPL Art. 22 / KSA PDPL Art. 29 (cross-border)**: SCC-equivalent + DPA-approved.

## Enforcement Layers

| Layer | Mechanism | Refusal at |
|---|---|---|
| Tenant onboarding | tenancy µservice assigns + pins pack | Onboarding gate |
| Network | Postgres `pg_hba.conf` + Kubernetes NetworkPolicy refuses cross-pack ingress | Network |
| Application | Pack tag in OIDC + per-tenant API-key binding; ingress routes by tag | API request |
| LEAN CI | `oya-check-cross-pack-replication-prohibition`, `oya-check-pack-pinning-coverage` | PR time |
| Audit | every Workflow event carries `pack_tag`; cross-pack flows emit explicit transfer record | Per-event |

## Verification

| Check | Cadence | Owner |
|---|---|---|
| LEAN: cross-pack replication prohibition | per-PR | axis-calendar |
| LEAN: pack-pinning coverage | per-PR | axis-calendar |
| Pen-test: cross-pack routing bypass | Annually | ops-security |
| Backup-residency audit | Quarterly | ops-sre-reliability |
| SCC compliance: transfer register review | Quarterly | council-privacy |

## References

- ADR-0117: data residency.
- ADR-0140: Cedar policy.
- `multi-region.md`, `compliance.md`, `legal/transfer-register.md`, `legal/dpa-template.md`, `legal/tia-template.md`.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Arts. 17, 23-2, 28-2.
- HIPAA 45 CFR §164.502(e); 45 CFR Part 164 Subpart D.
- APPI Arts. 17, 21, 27.
- PDPA, MAS Notice 644, APP, APRA-CPS 234, DPDPA, LGPD, UAE PDPL, KSA PDPL.
