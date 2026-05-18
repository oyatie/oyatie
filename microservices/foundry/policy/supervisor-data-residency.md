---
doc_class: PolicySpec
title: Data Residency Contract (foundry-supervisor)
microservice: foundry-supervisor
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-foundry-control-plane
deciders: council-privacy, ops-security, axis-foundry-control-plane, gtm-customer-success
related_adrs: [ADR-0117, ADR-0139, ADR-0131]
related_artifacts:
  - microservices/foundry/threat-model.md (T-I-01, T-T-02, T-S-04)
  - microservices/foundry/dpia.md (R-07)
  - microservices/foundry/policy/supervisor-isolation.md
  - microservices/foundry/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (foundry-supervisor µservice)

## Purpose

Define which jurisdictions' tenant fleet-state, autonomy entitlements, deployment history, and supervision events live in which Postgres / Valkey cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. Canonical residency artifact reviewed by EU DPAs (GDPR Arts. 44–50), Korean PIPC (PIPA Art. 28 + Art. 23-2), HIPAA covered-entity counsel (BAA), and equivalent supervisory authorities in every active pack.

## Residency Model

### Default: pack-pinning

Every tenant is assigned a primary pack at onboarding. The tenant's fleet-state Postgres rows, autonomy entitlements (OpenBao secret tree), kill-switch state, deployment history, and supervision events are stored in the pack's region-pinned cluster. Cross-pack movement is **forbidden by default**.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-postgres-1 (HA), kr-redis-cluster-1 | YES (M01 launch) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-postgres-{1,2}, eu-redis-cluster-{1,2} | Conditional |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-postgres-{1,2}, us-redis-cluster-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) + us-phoenix-1 | us-hc-postgres-{1,2}, us-hc-redis-cluster-{1,2}; isolated from non-HC | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-postgres-1, jp-redis-cluster-1 | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-postgres-1, sg-redis-cluster-1 | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-postgres-{1,2}, au-redis-cluster-{1,2} | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-postgres-{1,2}, in-redis-cluster-{1,2} | Conditional |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-postgres-{1,2}, br-redis-cluster-{1,2} | Conditional |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-postgres-{1,2}, ae-redis-cluster-{1,2} | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-postgres-{1,2}, ksa-redis-cluster-{1,2} | Conditional |

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: HQ jurisdiction + regulated-data flags
    ↓
Pack-router (Cedar policy in cloud-iac)
    ↓
OpenBao assigns tenant → pack + materializes Postgres row + Kubernetes namespace + OpenBao secret tree
    ↓
Supervisor Operator picks up the new tenant ns; default kill-switch (disengaged) + default autonomy entitlement (T0)
```

## Cross-Pack Replication Policy

### Default: forbidden

- Postgres rows: intra-pack only (master + DR-pair replica).
- Valkey state: intra-pack only.
- OpenBao secret trees: intra-pack only (`bao` clusters are per-pack).
- Capability YAMLs: tenant-owned git; tenant decides replication.
- Supervision-event-bus: intra-pack only.
- Audit-chain seals: intra-pack only (each pack runs its own audit-chain instance).

### Exception: tenant-executed SCCs (GDPR)

Cross-border transfer of EU-resident control-plane data permitted only when:
1. Active SCC on file at `legal/transfer-register.md`.
2. Receiving pack jurisdiction has adequate-decision (GDPR Art. 45) or equivalent safeguard.
3. Transfer-purpose specifically declared (e.g., DR failover within DR pair).
4. Audit-chain-emitted SCC-acknowledgement at the moment of transfer.

### Exception: HIPAA BAA + DR failover

Covered-entity tenants in pack-us-healthcare may have DR pair us-ashburn-1 + us-phoenix-1 (both HIPAA-eligible); intra-pack failover authorized. Cross-pack (us → eu) NOT authorized without separate tenant agreement.

### Exception: BCDR exercise

Controlled intra-pack restore drills permitted in DR-pair packs. Cross-pack BCDR not authorized.

## Postgres Row Tagging by Jurisdiction

Every supervisor Postgres table carries:

```sql
jurisdiction TEXT NOT NULL CHECK (jurisdiction IN
  ('kr', 'eu', 'us', 'us-hc', 'jp', 'sg', 'au', 'in', 'br', 'ae', 'ksa')),
pack         TEXT NOT NULL CHECK (pack LIKE 'pack-%'),
data_class   TEXT NOT NULL    -- per Bominal ADR-0028 taxonomy
```

Tampering attempts (label-injection from untrusted code paths) caught by the LEAN lane `oya-check-jurisdiction-label-conformance`.

## Retention by Jurisdiction × Data Class

Retention = MAX(asset default, pack statutory minimum, tenant DPA).

| Pack | Data class | Statutory minimum | Default applied |
|---|---|---|---|
| pack-kr | `BEHAVIORAL_TENANT_PRODUCT` | KR commercial code: 5 y | 5 y |
| pack-kr | `SENSITIVE_PIPA_ART23` (autonomy entitlements) | PIPA Art. 28 (bounded; erasure on request) | 5 y default; honour erasure |
| pack-kr | `AUDIT` | PIPA Enforcement Decree Art. 30: ≥ 1 y | 3 y (KR-FSS sector guidance) |
| pack-eu | `BEHAVIORAL_TENANT_PRODUCT` | bounded; tenant DPA | 2 y default |
| pack-eu | `AUDIT` | bounded by purpose | 2 y default |
| pack-us-healthcare | `PHI` (in capability payloads if not redacted) | HIPAA: 6 y or state law | MAX(6 y, state, tenant DPA) |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6 y | 6 y |
| pack-jp | `PII_IDENTIFYING` | APPI: bounded; honour deletion | bounded |
| pack-au | `PII_IDENTIFYING` | Privacy Act APP 11 + APP 12: bounded | bounded |
| pack-in | `PII_IDENTIFYING` | DPDPA 2023 §8(1)(g) | bounded |
| pack-br | `PII_IDENTIFYING` | LGPD Art. 16 | bounded |
| (all packs) | `SECRET` | rotate per ISO 27001 A.5.17 | 30 d API keys; 90 d signing keys |

`oya-check-retention-conformance` LEAN lane validates Postgres + Valkey retention configs.

## DSR Cascade

Right-to-erasure per GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V)-(VI):

1. Tenant raises DSR on behalf of end-user (joint controllership).
2. DSR runner identifies end-user identifiers in fleet-state + deployment history + supervision events.
3. Per-row soft-delete with 30-day grace; hard-delete after grace.
4. Audit-chain seal: `dsr_executed{tenant, subject_hash, removed_rows_count, timestamp}`.
5. Tenant notified within 30 d (or strictest pack SLA: KR 30 d, BR 15 d, EU 30 d).

Limitations:
- Older-than-retention data may be deleted before DSR processed.
- Capability payloads (in supervision events) may carry derived identifiers that the redactor missed; best-effort within retention.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC)

- PIPA Art. 28: bounded retention.
- PIPA Art. 23-2: sensitive cross-border forbidden by default.
- PIPC Notice 2020-7: overseas-transfer notification covered in tenant DPA.
- KR-FSS sector: audit log retention ≥ 5 y for financial-services tenants; encrypted at rest with KMS keys in KR-resident KMS.

### pack-eu (GDPR + EDPB + Schrems II)

- GDPR Arts. 44–46: SCC-only; Adequacy + Schrems-II supplementary technical measures (pseudonymisation + EU-resident KMS keys).
- EDPB Recommendations 01/2020: supplementary measures documented in `legal/schrems-supplementary-measures.md`.
- GDPR Art. 32 + 25: pseudonymisation + EU-resident-key encryption.

### pack-us-healthcare (HIPAA)

- §164.530(j): ≥ 6 y retention.
- HIPAA-eligible regions only.
- BAA-required pre-deploy.
- TPO scope only.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Pack overlays at `regional-packs/<pack>/foundry-supervisor-residency-overlay.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate retention-conformance --microservice foundry-supervisor` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate pack-routing-conformance --microservice foundry-supervisor` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cross-region-transfer-allowed-only-with-scc --microservice foundry-supervisor` — exit 0.
- Annual residency audit.
- Quarterly chaos drill: induce cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117, ADR-0139, ADR-0131.
- `microservices/foundry/threat-model.md` T-I-01, T-T-02, T-S-04.
- `microservices/foundry/dpia.md` R-07.
- `microservices/foundry/policy/supervisor-isolation.md`.
- `microservices/foundry/multi-region.md`.
- `legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md` (Slice-D scope).
- Oracle Cloud Infrastructure region documentation.
- GDPR Arts. 44–50; EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28; PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + 33; DPDPA 2023 §8(1)(g).
