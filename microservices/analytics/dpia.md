# Analytics µservice — Data Protection Impact Assessment (DPIA)

**Authority:** GDPR Article 35, K-PIPA Article 33-2, ADR-0008 DUBO, ADR-0038 DSR cascade, ADR-0156 PII registry, ADR-0193
**Status:** Draft (2026-05-18 data-substrate batch)
**Owner:** council-analytics + axis-compliance
**Reviewers:** DPO (Data Protection Officer), council-tenancy, council-cloud (security)
**Review cadence:** Annual + on material change (new data class, new region, new sub-processor)
**Last reviewed:** 2026-05-18

## 1. Mandate

A DPIA is mandatory under GDPR Article 35(1) when processing "is likely to result in a high risk to the rights and freedoms of natural persons", and explicitly required by Article 35(3) for systematic monitoring or large-scale processing of personal data. The analytics µservice meets both triggers: it processes billions of audit-log rows per tenant per year, and those rows contain `principal_id` (data subject identifier) bound to actions taken by that subject. K-PIPA Article 33-2 imposes a parallel obligation for Korean tenants.

## 2. Scope of processing

| Dimension | Detail |
|---|---|
| Controller | The tenant organization (B2B model — tenant is the controller of its end-user data) |
| Processor | Oyatie operating the analytics µservice |
| Data subjects | Tenant end-users (employees, customers, machine principals) |
| Categories of data | Identifiers (`principal_id`, `tenant_id`), action metadata (event_type, axis, ts), business KPI counters, billing usage counters |
| Special categories (GDPR Art 9) | NONE in the canonical schema. `PHI` is excluded (covered separately by a HIPAA-pack µservice tenancy). Biometric / genetic / political data is not ingested. |
| Source of data | Per-µservice outbox (workflow execution events, audit events, billing counters) |
| Storage location | Per-tenant ClickHouse database; per-cell residency-bound |
| Retention | 90 d hot + cold-tier per workload class; audit-log 7 yr (legal); business KPI 1 yr; billing 7 yr (tax) |
| Disclosure | Tenant-internal only; cross-tenant query forbidden by design; regulator export under explicit Cedar-authorized path |

## 3. Necessity and proportionality (Article 35(7)(b))

The processing is necessary because:

- **Audit log** — Required by SOC 2 CC7.2 (system monitoring) and ISO 27001 A.12.4 (logging and monitoring). Tenant cannot prove regulatory compliance without it.
- **Business KPI rollups** — Tenant cannot operate the workflow product without dashboards showing execution counts, error rates, and percentile latencies.
- **Billing rollups** — Required to bill the tenant for usage; cannot deliver service without billing.

Proportionality is enforced by:

- Per-tenant database isolation (Section 4 below) — no broader collection than necessary.
- Cold-tier and delete TTL clauses — data is not retained beyond the legally-required retention windows.
- Cedar policy default-deny — every access is explicitly authorized.

## 4. Risk assessment

### Risk 1 — Cross-tenant data leak (High → Low after mitigation)

- **Inherent risk:** High. Multi-tenant warehouse with shared cluster substrate. A single misconfiguration could expose tenant A's audit log to tenant B.
- **Mitigations (defense in depth):**
  1. **Database-per-tenant** (`tenant_{tenant_id}`) — primary isolation; physical separation of tables.
  2. **Per-tenant ClickHouse RBAC** — `tenant_{tid}_reader` / `_writer` roles scoped to the per-tenant database.
  3. **Adapter-layer `assert_same_tenant`** — kernel-layer guard before SQL dispatch (IP-003).
  4. **Cedar policy** — `principal.tenant_id == resource.tenant_id` at the API gateway (IP-007, IP-008).
  5. **Row-level policy fallback** — for the rare cross-tenant table (fleet ops dashboards), row-level policy enforces tenant scoping.
  6. **Penetration test** — IP-014 acceptance includes a cross-tenant probe; CI lane gates the merge.
  7. **Audit-chain emission** — every query is logged; cross-tenant query patterns are detectable post-hoc and trigger forensic review.
- **Residual risk:** Low. Four independent layers must all fail simultaneously to leak data.

### Risk 2 — PII surfacing via audit-log search (Medium → Low)

- **Inherent risk:** Medium. Audit events may contain free-text payload that could embed PII not declared in the schema.
- **Mitigations:**
  1. **PII registry per ADR-0156** — every column has a `data_class` tag; columns tagged `PII` are filtered by Cedar at projection.
  2. **Payload regex scan** — IP-004 ingest pipeline runs a PII-pattern probe (email, phone, SSN, IBAN); matches are redacted before insertion.
  3. **Cedar `data_class` policy** — `audit-log-pii.cedar` filters PII columns unless the principal has `DUBO::"PII"` grant per ADR-0008.
- **Residual risk:** Low. Free-text fields are the highest-risk vector; the regex probe is the primary mitigation, with Cedar as backstop.

### Risk 3 — Right-to-erasure non-fulfillment (Medium → Low)

- **Inherent risk:** Medium. ClickHouse `MergeTree` is append-mostly; row-level deletes are eventually-consistent (`mutations`).
- **Mitigations:**
  1. **DSR cascade per ADR-0038** — tenant offboard drops the entire `tenant_{tenant_id}` database; cascade is atomic at the database level.
  2. **Proof-of-erasure event** — audit-chain entry signed at offboard time; cosign-attestable.
  3. **Cold-tier deletion** — TTL DELETE clause covers the long-tail; quarterly verification job confirms no orphan parts remain.
  4. **For per-row erasure (rare; subject opt-out without tenant offboard) — use `ALTER TABLE ... DELETE WHERE principal_id = '...'`**; mutation completes in <24h; verified by audit-chain.
- **Residual risk:** Low. Tenant offboard is the canonical path; per-row erasure is the exception.

### Risk 4 — Backup compromise (Medium → Low)

- **Inherent risk:** Medium. Backups land in S3-compat; tampered backup → silent data corruption on restore.
- **Mitigations:** Cosign-signed backup manifests per ADR-0039; signature verification on restore (IP-012); separate signing key per cell; OpenBao-managed.
- **Residual risk:** Low. Signing key compromise is the only remaining path; mitigated by HSM-bound signing key per ADR-0043.

### Risk 5 — Quota-bypass denial-of-service (Medium → Low)

- **Inherent risk:** Medium. A single misbehaving tenant could starve cluster resources.
- **Mitigations:** Per-tenant ClickHouse QUOTA (IP-011) + adapter-layer rate-limit + cluster `max_concurrent_queries` ceiling.
- **Residual risk:** Low. Three-layer rate-limit.

### Risk 6 — Cross-region data egress (Medium → None for StrictKR/StrictEU)

- **Inherent risk:** Medium. ClickHouse `remote()` function could in principle federate across cells.
- **Mitigations:** Cedar policy forbids tenant principals from `remote()` (IP-010); NetworkPolicy denies cross-cell traffic for kr-* / eu-* cells.
- **Residual risk:** None for StrictKR/StrictEU; Low for KrWithUsFailover (explicit DR path, audit-logged).

## 5. Lawful basis

- **GDPR Article 6(1)(b)** — Necessary for performance of contract (tenant-Oyatie service agreement).
- **GDPR Article 6(1)(c)** — Legal obligation (audit log for SOC 2 / ISO 27001 compliance; billing for tax).
- **K-PIPA Article 15(1)(2)** — Necessary to fulfill the service contract.
- **HIPAA — out of scope here**; HIPAA-pack tenancy uses a separate µservice deployment with BAA in place.

## 6. Cross-border transfers

- **StrictKR tenants:** No cross-border transfer. KR data stays in kr-* cells.
- **StrictEU tenants:** No cross-border transfer. EU data stays in eu-* cells.
- **KrWithUsFailover tenants:** Cross-border transfer only on declared DR scenario; transfer is logged + tenant-notified.
- **Global tenants:** Standard Contractual Clauses (SCCs) per GDPR Article 46; UK IDTA where applicable.

## 7. Data subject rights — operational implementation

| Right | Implementation | Latency commitment |
|---|---|---|
| Access (Art 15) | IP-008 audit-log query + IP-013 regulator export | ≤ 30 days per GDPR |
| Rectification (Art 16) | Not applicable to immutable audit log; for KPI rollups, source µservice corrects + MV reprojects | ≤ 30 days |
| Erasure (Art 17) | IP-002 offboard cascade OR per-row `ALTER TABLE ... DELETE` | ≤ 30 days |
| Restriction (Art 18) | Cedar policy lockdown per principal_id | Immediate |
| Portability (Art 20) | IP-013 regulator export in NDJSON | ≤ 30 days |
| Objection (Art 21) | Tenant-controlled (tenant is the controller) | Per tenant policy |

## 8. Consultation

The DPO is consulted on:
- New data class introductions (new column with PII flag).
- New region pack (KR / EU / KSA / UAE / US-healthcare overlay changes).
- New sub-processor (e.g., new S3-compat provider).
- Material risk-rating changes.

## 9. Sign-off

| Role | Name | Date |
|---|---|---|
| DPO | (Pending appointment) | — |
| council-analytics chair | (Pending) | 2026-05-18 (draft) |
| Compliance officer | (Pending) | — |

## 10. References

- GDPR Articles 6, 9, 15-21, 35, 46.
- K-PIPA Articles 15, 33-2.
- ADR-0008 DUBO, ADR-0038 DSR cascade, ADR-0039 supply chain, ADR-0043 secrets, ADR-0049 cross-region residency, ADR-0156 PII registry, ADR-0193.
- ICO DPIA guidance: https://ico.org.uk/for-organisations/guide-to-data-protection/guide-to-the-general-data-protection-regulation-gdpr/data-protection-impact-assessments-dpias/
