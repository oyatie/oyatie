---
doc_class: PolicySpec
title: Data Residency Contract
microservice: mail
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-mail + ops-legal
deciders: council-privacy, ops-security, axis-mail, gtm-customer-success
related_adrs: [ADR-0117, ADR-0135, ADR-0130, ADR-0131]
related_artifacts:
  - microservices/mail/threat-model.md (T-I-08; cross-region replication)
  - microservices/mail/dpia.md (R-08; cross-border-misroute risk)
  - microservices/mail/policy/dual-context-isolation.md
  - microservices/mail/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (mail µservice)

## Purpose

Define which jurisdictions' tenant mail content + metadata + audit-chain seals + DKIM keys live in which Postgres/S3/Tantivy cluster, the cross-pack replication policy, and the legal-transfer mechanisms that gate any exception. Canonical residency artifact reviewed by EU DPAs (per GDPR Arts. 44-50), Korean PIPC (per PIPA Arts. 28 + 23-2), HIPAA Covered Entities (per BAA), and equivalent authorities in every active pack.

## Residency Model

### Default: pack-pinning

Every tenant assigned a primary pack at onboarding. Mail content (MIME blobs) + mailbox metadata + audit-chain seals + DKIM keys + retention ledger live in the pack-pinned region-cluster. Cross-pack movement forbidden by default.

| Pack | Primary region(s) | Cluster footprint | Activated? |
|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-mail-postgres-1, kr-mail-s3-1, kr-mail-tantivy-1, kr-mail-kms-1 | YES (M03 launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-mail-{postgres,s3,tantivy,kms}-{1,2} | Conditional (activated on first EU tenant SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-mail-* | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) — isolated from non-HC pack-us | us-hc-mail-* | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-mail-* | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-mail-* | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-mail-* | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-mail-* | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-mail-* | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-mail-* | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-mail-* | Conditional (KSA NCA cloud-residency) |

"Activated?" updated at first-tenant onboarding per pack; activation triggers re-review of this document + pack-overlay sections of `threat-model.md`, `dpia.md`, `compliance.md`.

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: collects tenant HQ jurisdiction + regulated-data declarations
  + dual-context pillar election (Professional always; Personal opt-in per user)
    ↓
Pack-router (Cedar policy in cloud-iac):
    - HQ jurisdiction → primary pack
    - Regulated-data flag (PHI, KR-FSS, etc.) → may force secondary pack (pack-us-healthcare overlay)
    - Conflict: ops-legal escalation
    ↓
OpenBao binds tenant → pack
    ↓
SMTP receiver advertised in DNS as pack-pinned MX
    ↓
All mail flows to pack's clusters; never cross-pack
```

Routing encoded as Cedar policy at `policy/pack-routing.cedar` (or fragment in `cloud-iac` µservice).

### Per-user Personal context residency

A user's Personal mailbox is pinned to the pack of the user's nationality jurisdiction at sign-up:
- KR national → pack-kr Personal mailbox
- EU national → pack-eu Personal mailbox
- US national → pack-us Personal mailbox

If a KR national works for an EU tenant (Professional context in pack-eu, Personal context in pack-kr), their two mailboxes are in DIFFERENT packs by design. The dual-context invariant per `dual-context-isolation.md` ensures isolation.

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any mail data is forbidden by default. Specifically:

- Postgres tables (mailbox metadata): replicate within-pack only.
- S3 MIME blobs: replicate within-pack only.
- Tantivy search indices: replicate within-pack only.
- KMS keys (per-tenant DEK, per-tenant DKIM key): pack-resident; not exportable.
- Retention ledger: pack-resident.
- Audit-chain seals: pack-resident.
- Per-tenant SMTP IP pool: pack-resident.

### Exception: tenant-executed SCCs (GDPR + UK + Swiss transfer mechanism)

Cross-border transfer of EU-resident mail data permitted only with active SCC per GDPR Arts. 44-46. Requires:

1. Active SCC on file at `legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequate-decision (GDPR Art. 45) or equivalent safeguard.
3. Transfer-purpose limited to specifically-named processing (e.g., "DR failover").
4. Audit-chain emission on transfer event.
5. Tenant notice on transfer event.

### Exception: HIPAA BAA + DR failover within HIPAA-eligible region

pack-us-healthcare DR pair us-ashburn-1 + us-phoenix-1: intra-region failover between BAA-eligible OCI regions is permitted. Cross-region (us-hc → eu) failover NOT authorised without separate BAA addendum.

### Exception: BCDR exercise (controlled, scheduled, intra-pack)

For BCDR validation, controlled cross-AZ failover within a pack is permitted (e.g., pack-eu eu-frankfurt-1 → eu-amsterdam-1 DR pair). Cross-pack BCDR not authorised.

### Exception: eDiscovery export sealed bundle download

A sealed eDiscovery export bundle is technically "moved out" of the pack when downloaded by the requesting compliance officer; per `policy/dual-context-isolation.md` + Bominal ADR-0215, this is:
- Allowed: download by tenant-internal compliance officer to their workstation (within same jurisdictional scope as tenant).
- Allowed with SCC: download to external counsel in non-adequate jurisdiction.
- Audit-chained: every download event sealed.

## Tagging by Jurisdiction

In addition to per-tenant `X-Scope-OrgID`, mail data carries jurisdiction labels for routing + retention enforcement:

```text
metadata_label:
  jurisdiction: kr | eu | us | us-hc | jp | sg | au | in | br | ae | ksa
  pack:         pack-kr | pack-eu | ...
  context_kind: Professional | Personal
  data_class:   one of the class taxonomy values per Bominal ADR-0028
```

Properties:
- `jurisdiction` set by tenant + user pack assignment.
- Tampering attempts detected by the pack-router enforcement layer.
- Mimir-equivalent metric retention keys on `(tenant, jurisdiction, data_class)`.

## Retention by Jurisdiction × Data Class

Retention windows = MAX of:
- Asset class default (per `threat-model.md` §"Assets & Data Classification").
- Pack legal minimum (statutory retention).
- Tenant-contracted retention (DPA-declared).
- Hold-engaged extension (legal hold prevents expiry).

| Pack | Data class | Minimum statutory | Default applied |
|---|---|---|---|
| pack-kr | mail content (Professional, non-financial) | KR PIPA Art. 28: bounded | 7y default; honour erasure |
| pack-kr | mail content (Professional, KR-FSS-financial-services) | KR 상법 + KR-FSS: 5y minimum | 7y default (5y floor) |
| pack-kr | mail content (Personal) | PIPA Art. 28: bounded; honour erasure | user-controlled |
| pack-kr | audit-chain | PIPA Enforcement Decree Art. 30: ≥ 1y | 3y default (5y for KR-FSS tenant) |
| pack-kr | DKIM key | rotation 90d per ISO A.5.17 | 90d |
| pack-eu | mail content (Professional, GDPR) | bounded by purpose | per tenant DPA; default 7y |
| pack-eu | mail content (Personal) | Art. 17 erasure | user-controlled |
| pack-eu | audit-chain | bounded by purpose; ROPA-declared | 2y default |
| pack-us-healthcare | mail content (PHI) | HIPAA: tenant state Medical Records Retention | MAX(HIPAA 6y, state, tenant DPA) |
| pack-us-healthcare | audit-chain | HIPAA §164.316(b)(2): 6y | 6y |
| pack-us | mail content (Professional) | bounded by purpose | per tenant DPA; default 7y |
| pack-us | audit-chain | bounded by purpose | 2y default |
| pack-jp | mail content (Professional) | APPI: bounded | per tenant DPA |
| pack-au | mail content (Professional) | Privacy Act APP 11 + APP 12: bounded | per tenant DPA |
| pack-in | mail content (Professional) | DPDPA 2023 §8(1)(g): storage limitation | per tenant DPA |
| pack-br | mail content (Professional) | LGPD Art. 16 | per tenant DPA |
| (all packs) | DKIM private keys | rotation 90d | 90d |
| (all packs) | TLS certs | rotation 30d | 30d |
| (all packs) | Per-tenant DEK envelope keys | rotation on KMS event | per KMS policy |

LEAN lane `oya-check-retention-floor-conformance` validates Postgres + S3 retention configs against this table.

## DSR (Data Subject Request) Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(V)-(VI)) honoured via `oya-dsr-cascade-runner` skill:

1. Tenant raises DSR on behalf of their end-user/recipient (joint controllership per Art. 26).
2. DSR runner identifies subject's identifiers (email address, hashed user-id, message-IDs).
3. Mailbox metadata + MIME blobs + search index entries + audit-chain index identified.
4. Soft-delete with 30-day grace; hard-delete after grace.
5. Audit-chain seal: `mail_dsr_executed{tenant, subject_hash, removed_message_count, timestamp}`.
6. Tenant notified within 30d SLA per GDPR; pack-specific shorter SLAs honoured (KR 30d, BR 15d, EU 30d).

Limitations (documented in DPIA R-09):
- Data older than retention window may already be deleted.
- Audit-chain entries linking DSR action itself are NOT erased (they ARE the erasure record).
- Legal-hold-engaged messages cannot be erased until hold released; user notified.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC)

- **PIPA Art. 28 (storage period)**: bounded; sensitive data minimal retention.
- **PIPA Art. 23-2 (sensitive data cross-border)**: forbidden by default; tenant DPA acknowledgement.
- **PIPC Notice 2020-7 (overseas-transfer)**: oyatie's pack-kr residency guarantee acknowledged in tenant DPA.
- **KR 전자문서법**: mail-as-document; audit-chain Ed25519 satisfies integrity, storage, verification (Arts. 5/6/7).
- **KR-FSS guidance** (financial-services tenants): mail retention floor 5y; KMS-in-KR; KR-resident operator access only.

### pack-eu (GDPR + EDPB + Schrems II + ePrivacy)

- **GDPR Arts. 44-46 transfers**: SCC-only; EU-list-of-adequate-countries; Schrems-II supplementary measures (pseudonymisation + EU-controlled KMS).
- **EDPB Recommendations 01/2020**: supplementary measures at `legal/schrems-supplementary-measures.md`.
- **GDPR Arts. 32 + 25**: pseudonymisation + EU-resident-key encryption.
- **ePrivacy Directive Art. 5**: e-mail confidentiality at EU edge.

### pack-us-healthcare (HIPAA)

- **45 CFR §164.530(j) (Records retention)**: ≥ 6y from creation.
- **HIPAA-eligible regions only**: OCI us-ashburn-1 + us-phoenix-1 per Oracle attestation.
- **BAA-required**: signed pre-onboarding.
- **Permitted Uses + Disclosures**: TPO; operations scope covers mail.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/mail-residency-overlay.md` carries local data-residency law citations. Pack-pinning + cross-pack-replication-forbidden apply universally.

## Verification

- `cargo run -p oya-dev-cli -- gate validate retention-floor-conformance --microservice mail` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate pack-routing-conformance --microservice mail` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cross-region-transfer-allowed-only-with-scc --microservice mail` — exit 0.
- Annual residency audit: confirm each tenant's data location matches assigned pack.
- Quarterly chaos drill: induce cross-pack write attempt; verify rejection + alerting.

## References

- ADR-0117: Cloud-native infrastructure (residency).
- ADR-0135: Connect dissolution; dual-context residency split.
- ADR-0130: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- `microservices/mail/threat-model.md` T-I-08.
- `microservices/mail/dpia.md` R-08 + R-13 + §2.2.
- `microservices/mail/policy/dual-context-isolation.md`.
- `microservices/mail/multi-region.md`.
- `microservices/mail/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md`.
- `regional-packs/<pack>/mail-residency-overlay.md`.
- OCI region documentation.
- GDPR Arts. 44-50.
- EDPB Recommendations 01/2020.
- KR PIPA Art. 23-2 + Art. 28 + PIPC Notice 2020-7.
- HIPAA 45 CFR §164.530(j).
- LGPD Art. 16 + Art. 33.
- DPDPA 2023 §8(1)(g).
