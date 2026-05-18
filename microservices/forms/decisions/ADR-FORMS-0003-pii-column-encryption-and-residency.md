---
id: ADR-FORMS-0003
title: PII column encryption — per-tenant DEK with envelope encryption (OpenBao root); per-pack residency
microservice: forms
status: Accepted
date: 2026-05-17
owner: axis-forms + ops-security + council-privacy
deciders: council-architecture, axis-forms, ops-security, council-privacy, council-legal-compliance
supersedes: []
superseded_by: []
related: [ADR-0117, ADR-0131, ADR-0140, ADR-FORMS-0001]
related_specs: [/specs/microservices/forms.json]
related_artifacts:
  - microservices/forms/PRD.md FR-06 + AC-08
  - microservices/forms/policy/data-residency.md
  - microservices/forms/threat-model.md §"T-I-04"
  - microservices/forms/dpia.md
  - microservices/forms/compliance.md §"1. GDPR" §"3. HIPAA"
doc_status: published
---

# ADR-FORMS-0003: PII column encryption — per-tenant DEK (envelope encryption with OpenBao-rooted KEK); per-pack residency

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

Forms stores tenant-collected PII (and where applicable, PHI, special-category Art. 9 data, FINANCIAL). At-rest encryption is mandatory per:
- GDPR Art. 32 (security of processing; pseudonymisation + encryption).
- HIPAA §164.312(a)(2)(iv) + (e)(2)(ii) (technical safeguards for ePHI).
- KR PIPA Art. 29 + Enforcement Decree Art. 30 (technical and managerial measures).
- LGPD Art. 46 (security measures).
- PCI DSS v4 Req 3 (when payment data present, even tokenised — though PAN is offloaded to fintech µservice).
- SOC 2 CC6.7 (encryption controls).
- ISO 27001:2022 A.5.34 + A.8.24 (cryptography + cryptographic key management).

At-rest encryption alone (full-disk LUKS / OCI block-storage encryption) is **insufficient** because:
1. A DBA with `SELECT` privileges sees raw PII bytes; full-disk encryption only protects against stolen physical media.
2. A backup restored to a non-production environment leaks PII unless column-level encryption travels with the backup.
3. A misconfigured Citus replica leak (cross-pack) would expose plaintext.

Column-level encryption requires choosing:
1. **Key hierarchy**: single tenant-pool key vs per-tenant DEK vs per-row DEK?
2. **KMS root**: OCI Vault vs HashiCorp Vault vs OpenBao vs CloudHSM?
3. **Algorithm**: AES-GCM-256 vs ChaCha20-Poly1305 vs Format-Preserving Encryption?
4. **Rotation cadence**: how often DEK rotates; how rolling re-encryption is performed without downtime?
5. **Per-pack residency**: where the KEK / DEK lives geographically?

## Decision

Adopt **envelope encryption** with:

### Key hierarchy

- **Root**: pack-resident **OpenBao** instance (per-pack, per `policy/data-residency.md`). OpenBao is the canonical secrets backbone per ADR-0117.
- **KEK** (Key Encryption Key): per-tenant; stored in OpenBao under `secret/forms/tenant/<tenant_id>/kek`. KEK rotated quarterly.
- **DEK** (Data Encryption Key): per-tenant + per-data-class; AES-256 key wrapped by tenant KEK; stored alongside the encrypted column data (wrapped form only).
- **Hot-cached DEK**: response-collector + export-worker fetch DEK once per session (≤ 5min TTL); fetch is mTLS to OpenBao.

### Algorithm

- **AES-256-GCM** for column encryption (authenticated encryption; FIPS 140-3 certified path available via OpenBao FIPS variant).
- **AES-256-KW** (RFC 3394) for DEK wrap by KEK.
- Random 96-bit nonces per row; nonce stored with ciphertext.
- AAD (Additional Authenticated Data) includes `(tenant_id, form_id, response_id, field_id, data_class)` — defends against ciphertext-block reuse across rows / fields.

### Per-pack residency

- OpenBao per-pack: `secret/forms/...` lives at the pack-resident OpenBao endpoint.
- KEK NEVER replicated cross-pack.
- Cross-pack DSR + DR fail-closed if KEK unavailable.

### Rotation

- **DEK rotation**: quarterly per-tenant; rolling re-encryption with dual-key window:
  - Phase 1: new DEK created + wrapped; reads accept both old + new DEK; writes use new DEK.
  - Phase 2: background re-encryption of all rows under old DEK with new DEK (per-tenant batch; ≤ 1M rows/hour throttle).
  - Phase 3: old DEK destroyed after re-encryption complete; verification via random-row decrypt audit.
- **KEK rotation**: quarterly per-tenant; OpenBao manages the rotation; wrapped DEKs re-wrapped on rotation.
- **Emergency rotation**: triggered by ops-security on any suspected key compromise; per `runbooks/pii-leak-incident-p0.md`.

### Per-data-class behaviour

| Data class | Encryption | Notes |
|---|---|---|
| NORMAL | not column-encrypted; full-disk only | non-PII fields (e.g., timestamps, response_id) |
| PII_IDENTIFYING | column-encrypted | email, name, phone |
| PII_QUASI_IDENTIFIER | column-encrypted | dob, postcode |
| SENSITIVE_GDPR_ART9 | column-encrypted + Art. 9 consent linkage | health, religion, ethnicity, sexual orientation, political opinion |
| PHI | column-encrypted + BAA-tenant-required + pack-us-healthcare-only | per HIPAA |
| FINANCIAL | column-encrypted + payment-µservice-tokenisation | per PCI DSS Req 3; no PAN ever stored here |
| BEHAVIORAL_TENANT_PRODUCT | not column-encrypted (analytics roll-ups; submitter-hash-anonymous) | retention shortest |
| SECRET | never persisted in Forms data store | per `tenant-scope.cedar` FORBID |

### Search

Per-pack Meilisearch index — PII columns NEVER indexed. Only non-PII fields searchable. AC-09 invariant.

### DEK in audit-chain

The audit-chain seal records `dek_version` per write; auditor can verify DEK lineage without decrypting.

## Alternatives Considered

### Alternative A — Single platform KEK shared across tenants

Use one platform-wide KEK; per-tenant DEK derived deterministically from KEK + tenant_id.

- **Pros**
  - Simpler key management.
  - Lower OpenBao operational footprint.
- **Cons**
  - Single-key compromise = all tenants compromised; blast radius equals all of Forms in a pack.
  - Tenant-side compliance (tenants in HIPAA / Art. 9 contexts) typically require tenant-isolated keys per their DPA.
  - Customer-managed keys (CMK) tier impossible later without re-architecture.
- **Rejected reason**: blast radius too wide. Per-tenant KEK is the industry standard for serious SaaS (Stripe, Snowflake CMK, AWS KMS-per-customer pattern).

### Alternative B — Per-row DEK (each row encrypted under a unique key)

Generate a fresh DEK per row; store wrapped DEK alongside the row.

- **Pros**
  - Maximum cryptographic separation; row compromise doesn't extend.
- **Cons**
  - Storage overhead (~50 bytes wrap per column per row).
  - Performance impact: DEK unwrap on every read (cannot be cached cluster-wide).
  - Export of 100k responses requires 100k DEK unwraps; export latency budget breached.
- **Rejected reason**: cost > benefit. AES-GCM with per-row nonces + per-(tenant, data_class) DEK is structurally safe; per-row DEK is overkill given AAD-bound nonces.

### Alternative C — Full-disk encryption only (no column encryption)

Rely on OCI block-storage AES + LUKS only.

- **Pros**
  - Cheapest; zero application complexity.
- **Cons**
  - Protects only against physical media theft; not against DBA insider OR misconfigured backup restore.
  - GDPR Art. 32 pseudonymisation expectation typically expects column-level for serious PII.
  - HIPAA "addressable" implementation specification §164.312(a)(2)(iv): encryption at rest typically implemented at column-level for PHI databases.
- **Rejected reason**: insufficient for the threat model + regulatory posture.

### Alternative D — Format-Preserving Encryption (FPE) for searchability

Use FPE (AES-FF1) so encrypted columns remain searchable / indexable.

- **Pros**
  - Searchable encryption; Meilisearch could index encrypted columns.
- **Cons**
  - FPE has known weaknesses (related-key attacks; lower margins of security).
  - NIST SP 800-38G updated 2019: FF3-1 weakness; only FF1 considered safe; even FF1 has caveats.
  - Tenant analytics over encrypted columns is rare; not a primary use case.
- **Rejected reason**: security-cost-of-FPE > value-of-searchable-encrypted-columns. AC-09 commits to never indexing PII, which is the cleaner security posture.

### Alternative E — Customer-managed keys (CMK) only; no platform DEK

Every tenant brings their own KEK (e.g., AWS KMS in tenant's account).

- **Pros**
  - Maximum tenant control.
  - "Keep your data" tier feature.
- **Cons**
  - Onboarding friction; small tenants cannot operate CMK.
  - Cross-pack consistency hard.
  - Forms can't operate on encrypted data without tenant key available; DSR cascades degrade if tenant KMS unavailable.
- **Rejected reason**: CMK is a Tier-G+ feature on top of the chosen design; CMK without platform default-DEK is too friction-heavy for Tier-2 tenants. Chosen design supports CMK as an opt-in supersession.

## Consequences

### Architectural

- The `oya-forms-postgres-adapter` (per IP-006) wraps every PII column write via the `oya-forms-crypto-domain` kernel.
- The crypto kernel exposes `Encrypt(tenant_id, data_class, plaintext, AAD) -> ciphertext` and the inverse.
- The OpenBao client SDK caches DEK ≤ 5min per session; cache invalidated on rotation event.
- The export-worker decrypts on a streaming basis; never materialises full plaintext in memory beyond chunk size.

### Downstream µservices

1. **tenancy**: tenant-onboarding creates per-tenant KEK + initial DEK in OpenBao.
2. **audit-chain**: every write event carries `dek_version`; chain seals dek_version for forensic replay.
3. **observability**: SLO `oya-forms-pii-encryption-correctness` = 100%; non-zero unencrypted PII write triggers Sev-1.
4. **drive** (cross-µservice for file uploads): drive enforces its own column / object encryption; Forms passes data-class declaration to drive at upload time.
5. **DSR runner**: erasure deletes both ciphertext rows AND wraps. Old DEK destroyed independently per rotation.
6. **All exports**: PII columns redacted by default; unredacted export requires entitlement (`has_pii_read_entitlement`) AND audit-chain seal on export.

### SLOs and CI lanes affected

- `oya-forms-pii-column-encryption-correctness` — exit 0; every PII write encrypted (AC-08).
- `oya-governance-citus-rls-enforced` — defense-in-depth on top of encryption.
- `oya-forms-pii-column-not-indexed` — Meilisearch index never sees plaintext PII (AC-09 + T-I-05).
- `oya-forms-dek-rotation-conformance` — quarterly rotation completion verified.

### Compliance + audit

- GDPR Art. 32 + 25: pseudonymisation by encryption; per-tenant DEK isolates blast radius.
- HIPAA §164.312(a)(2)(iv): addressable implementation specification fulfilled for ePHI.
- KR PIPA Art. 29: technical and managerial measures including encryption.
- LGPD Art. 46: appropriate security measures.
- PCI DSS v4 Req 3: covered (PAN tokenised at fintech; Forms never stores PAN).
- ISO 27001:2022 A.5.34 + A.8.24: cryptography + key management.
- SOC 2 CC6.7: encryption controls evidenced.

### Risk register

- **Risk**: OpenBao pack-resident outage → DEK unavailable → submit fail-closed; tenant outage. **Mitigation**: OpenBao HA per pack; runbook engages within 5min.
- **Risk**: DEK rotation fails mid-pass → dual-key window extended. **Mitigation**: idempotent re-encryption; resumable; chaos drill quarterly.
- **Risk**: Tenant deletion / migration → KEK destruction must cascade. **Mitigation**: tenant offboarding runbook destroys KEK at the end; cryptographic erasure complements DSR.
- **Risk**: NIST AES-256-GCM advisories (e.g., nonce reuse in 2^32 messages). **Mitigation**: 96-bit random nonce + AAD + per-(tenant, data_class) DEK keeps message count per-key well under 2^32; rotation accelerates if approached.
- **Risk**: Compliance regime evolves (e.g., post-quantum migration). **Mitigation**: ADR supersession path; KEM-wrap DEK is the post-quantum upgrade path.

## References

- NIST SP 800-38D — AES-GCM.
- NIST SP 800-57 Part 1 r5 — Key management.
- IETF RFC 3394 — AES Key Wrap.
- IETF RFC 5116 — Authenticated Encryption with Associated Data.
- GDPR Art. 32, Art. 25.
- HIPAA 45 CFR §164.312(a)(2)(iv) + (e)(2)(ii).
- KR PIPA Art. 29 + Enforcement Decree Art. 30.
- LGPD Art. 46.
- PCI DSS v4 Req 3.
- ISO 27001:2022 A.5.34 + A.8.24.
- SOC 2 CC6.7.
- OpenBao docs — `openbao.org/`.
- HashiCorp Vault encryption-as-a-service — `developer.hashicorp.com/vault/`.
- Stripe + Snowflake CMK pattern reference docs.
- `microservices/forms/policy/data-residency.md`.
- `microservices/forms/threat-model.md` T-I-04.
- ADR-0117 cloud-native infra.
- ADR-0131 per-microservice flat layout.
- ADR-FORMS-0001 (form-definition schema; `x-data-class` declaration).
