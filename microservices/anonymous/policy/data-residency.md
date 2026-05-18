---
doc_class: DataResidencyPolicy
template_id: TPL-POLICY-MD
title: Data Residency Policy — anonymous µservice
microservice: anonymous
status: Accepted
date: 2026-05-17
owner_team: axis-anonymous + ops-data + council-privacy + general-counsel
related_adrs: [ADR-0117, ADR-ANON-0003, ADR-ANON-0004]
related_artifacts:
  - microservices/anonymous/PRD.md §"NFR / Data residency"
  - microservices/anonymous/multi-region.md
  - microservices/anonymous/dpia.md
doc_status: published
---

# Data Residency Policy — anonymous µservice

## Purpose

Define where the `anonymous` µservice's data physically resides per regulatory pack, what cross-region replication is permitted, and what is **structurally refused** (cross-pack federation; cross-pack identity correlation).

## Pack-to-region binding

Per ADR-0117 (tenant pack jurisdiction), each tenant is bound to exactly one **regulatory pack**, which maps to a set of **primary regions** and **read-replica regions**.

| Pack | Primary regions | Read-replica regions | Cross-pack replication |
|---|---|---|---|
| pack-kr | ap-northeast-2 (Seoul) | – (none; PIPA Art. 28 cross-border restriction; cross-border outflow requires explicit consent per 정보통신망법) | REFUSED |
| pack-eu | eu-central-1 (Frankfurt), eu-west-1 (Dublin) | eu-west-3 (Paris) for DR | REFUSED (GDPR Art. 44; Schrems II) |
| pack-us | us-east-1 (N. Virginia), us-west-2 (Oregon) | us-east-2 (Ohio) for DR | REFUSED (state anti-doxxing laws; potential CCPA cross-border issues) |
| pack-us-healthcare | us-east-1 (HIPAA-covered VPC) | us-east-2 (HIPAA-covered DR) | REFUSED (HIPAA / 45 CFR §164) |
| pack-uk | eu-west-2 (London) | eu-west-1 (Dublin) for DR | REFUSED (UK OSA 2023 + IPA 2016) |
| pack-jp | ap-northeast-1 (Tokyo) | ap-northeast-3 (Osaka) for DR | REFUSED (通信の秘密 + APPI) |
| pack-sg | ap-southeast-1 (Singapore) | – | REFUSED (PDPA + MAS-TRM) |
| pack-au | ap-southeast-2 (Sydney) | – | REFUSED (Privacy Act 1988 APP 8; OSA 2021) |
| pack-in | ap-south-1 (Mumbai) | – | REFUSED (DPDPA 2023 §16; RBI Master Direction 2023) |
| pack-br | sa-east-1 (São Paulo) | – | REFUSED (LGPD Art. 33 cross-border) |
| pack-ae | me-central-1 (UAE) | – | REFUSED (UAE PDPL Art. 22) |
| pack-ksa | me-south-1 (Bahrain — KSA-compliant region) | – | REFUSED (KSA PDPL Royal Decree M/19/2021) |

## Cross-pack data flows that ARE permitted (and how they preserve invariants)

| Flow | Why permitted | Privacy preservation |
|---|---|---|
| Audit-chain Merkle root replication (root hash only; no content) | global integrity of audit-chain | Merkle root is cryptographic; reveals no content |
| Transparency-report aggregator (counts only; no per-tenant content) | EU DSA Arts. 27/28 + KR PIPA Art. 28 transparency obligation | aggregate counts; k-anonymity floor enforced |
| Per-pack BBS+ issuer-public-key registry replication | cross-pack issuer trust (rare; only for multi-jurisdiction enterprises) | public keys only; never private keys |
| Promotion-eligibility metric (from observability) | global SLO-gating per ADR-0130 | metric is aggregate; never per-tenant post content |

## Cross-pack data flows that are REFUSED

| Flow | Why refused | Source citation |
|---|---|---|
| Post body cross-pack replication | violates GDPR Art. 44 (transfer to third country); KR PIPA Art. 28; UAE PDPL Art. 22; etc. | per-pack regulatory regime |
| Affinity binding cross-pack replication | violates GDPR Art. 44 + I2 | per-pack |
| User_id ↔ post_id correlation cross-pack | structurally refused by I1 | I1 |
| Federation between packs (ActivityPub / AT Proto / Matrix) | violates I5 | ADR-ANON-0006 |
| Legal-process disclosure cross-pack | refused unless explicit MLAT (Mutual Legal Assistance Treaty) ratifies; even then, the disclosure executes in the user's pack, not in the requesting pack | ECPA / SCA + KR 통신비밀보호법; case-by-case |
| Cross-region trending aggregator | refused; trending is affinity-cluster-scoped, and affinity clusters are pack-scoped | ADR-ANON-0007 |

## Per-pack retention authority

(Reference; full detail in ADR-ANON-0004.)

| Pack | Default retention | Tenant-selectable upper bound | Hard-delete propagation SLO |
|---|---|---|---|
| pack-kr | 30 days | 90 days | p99 ≤ 5 s |
| pack-eu | 30 days | 60 days (GDPR Art. 5(1)(e) storage-limitation principle) | p99 ≤ 5 s |
| pack-us | 30 days | 90 days | p99 ≤ 5 s |
| pack-us-healthcare | 30 days | 60 days (HIPAA 45 CFR §164.530(j) governs PHI; we keep tight) | p99 ≤ 5 s |
| pack-uk | 30 days | 90 days | p99 ≤ 5 s |
| pack-jp | 30 days | 60 days (APPI) | p99 ≤ 5 s |
| (other packs) | 30 days | 60-90 days per local regulator | p99 ≤ 5 s |

## Storage component-by-component residency

| Component | Residency | Cross-region replication | Notes |
|---|---|---|---|
| Postgres `posts` table | primary region of pack | within-pack DR replica only | RLS + blinding columns enforced |
| Postgres `votes` table | primary region of pack | within-pack DR replica only | RLS |
| Postgres `affinity_attestation_bindings` table | primary region of pack | within-pack DR replica only | RLS + blinded-commitment columns |
| Postgres `legal_process_orders` table | primary region of pack | within-pack DR replica only; audit-chain seal additionally globally replicated as hash | sealed records |
| Redis feed-cache | primary region of pack | none (cache; reconstructed) | per-affinity-cluster prefix |
| Meilisearch hashtag index | primary region of pack | within-pack DR replica only | anonymised corpus |
| MLS DM ciphertext store (anonymous-DM via messenger) | primary region of pack | within-pack DR replica only | ciphertext only |
| OpenBao secrets | per-pack OpenBao cluster | none | secret references in code via `${openbao:secret/<path>}` |
| Audit-chain seals (hashes) | per-pack primary + globally-replicated as hash root | hash root global; full record per-pack | Merkle structure |

## Operator scope

Per `policy/tenant-scope.cedar` + `policy/auditor-scope.cedar`:

- An oyatie operator in pack-eu CANNOT read pack-kr's data (Cedar deny).
- An oyatie operator in any pack CANNOT correlate user_id ↔ post_id (Cedar deny; per I1).
- An auditor scope cannot read user content in any pack.

## Compliance evidence

| Pack | Evidence artifact | Audit cadence |
|---|---|---|
| pack-eu | DPA appointment + Records of Processing Activities (Art. 30) + DPIA (`dpia.md`) | annual |
| pack-kr | PIPC registration + KR-ISMS-P certification | annual |
| pack-us-healthcare | BAA with covered entities; SOC 2 Type 2 + HIPAA technical safeguards | annual SOC 2 + biennial HIPAA |
| pack-uk | UK ICO registration + DPIA + OSA 2023 risk-assessment | annual |
| pack-jp | APPI register + 通信の秘密 compliance audit | annual |
| pack-sg | PDPC notification + MAS-TRM v2021 audit (if financial-services tenant) | annual + per-tenant |
| pack-au | OAIC registration + APRA-CPS 234 audit (if financial-services tenant) | annual + per-tenant |
| pack-in | DPDPA appointment + RBI audit | annual |
| pack-br | ANPD notification + BACEN audit (if financial) | annual |
| pack-ae | UAE PDPL registration | annual |
| pack-ksa | KSA PDPL registration + SAMA Cybersecurity Framework audit | annual |

## References

- ADR-0117 — tenant pack jurisdiction
- ADR-ANON-0003 — legal-process workflow (cross-pack disclosure constraints)
- ADR-ANON-0004 — retention + deletion policy
- ADR-ANON-0006 — federation refusal
- GDPR Arts. 44-50; KR PIPA Art. 28; KR 통신비밀보호법; UK IPA 2016; HIPAA 45 CFR §164.530(j); UAE PDPL; KSA PDPL; APPI; LGPD; DPDPA 2023
- Schrems II (CJEU Case C-311/18, July 2020)
