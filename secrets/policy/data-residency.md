---
doc_class: PolicySpec
title: Data Residency Contract
microservice: cloud-secrets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + axis-cloud-secrets
deciders: council-privacy, ops-security, axis-cloud-secrets, ops-legal
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/cloud-secrets/threat-model.md (T-I-04, T-I-05; HSM compromise + backup leak)
  - microservices/cloud-secrets/dpia.md (R-06, R-07, R-11; cross-pack misroute)
  - microservices/cloud-secrets/policy/secret-isolation.md
  - microservices/cloud-secrets/multi-region.md
review_cadence: annually + on every regional-pack activation
doc_status: published
---

# Data Residency Contract (cloud-secrets µservice)

## Purpose

Define which jurisdictions' tenant secrets live in which OpenBao + HSM partition; the cross-pack replication policy (forbidden); the legal-transfer mechanisms that gate any exception. This is the canonical residency artifact reviewed by EU DPAs (GDPR Arts. 44–50), the Korean PIPC (PIPA Art. 28 + 23-2), HIPAA tenants' Covered Entity counsel (BAA), and equivalent supervisory authorities.

## Residency Model

### Default: pack-pinning + cross-pack-replication-forbidden

Every tenant is assigned a primary pack at onboarding. The tenant's secrets and KEK material live in the pack's region-pinned OpenBao + HSM. **Cross-pack movement is forbidden by default for both secrets AND KEK material AND audit events.**

| Pack | Primary region(s) | OpenBao instance | HSM partition vendor | Postgres backend | Activated? |
|---|---|---|---|---|---|
| pack-kr | OCI ap-seoul-1 | kr-openbao-1 | Thales Luna (KR-FSS preference) | kr-pg-patroni-1 | YES (M01 launch tenant) |
| pack-eu | OCI eu-frankfurt-1 + eu-amsterdam-1 (DR pair) | eu-openbao-{1,2} | OCI Cloud-HSM (EU-resident) | eu-pg-patroni-{1,2} | Conditional (post-SCC) |
| pack-us | OCI us-ashburn-1 + us-phoenix-1 (DR pair) | us-openbao-{1,2} | OCI Cloud-HSM | us-pg-patroni-{1,2} | Conditional |
| pack-us-healthcare | OCI us-ashburn-1 (HIPAA-eligible) | us-hc-openbao-1 (isolated from non-HC) | OCI Cloud-HSM (HIPAA-eligible) | us-hc-pg-patroni-1 | Conditional (post-BAA) |
| pack-jp | OCI ap-tokyo-1 | jp-openbao-1 | OCI Cloud-HSM | jp-pg-patroni-1 | Conditional |
| pack-sg | OCI ap-singapore-1 | sg-openbao-1 | OCI Cloud-HSM | sg-pg-patroni-1 | Conditional |
| pack-au | OCI ap-sydney-1 + ap-melbourne-1 | au-openbao-{1,2} | OCI Cloud-HSM (AU-resident) | au-pg-patroni-{1,2} | Conditional |
| pack-in | OCI ap-hyderabad-1 + ap-mumbai-1 | in-openbao-{1,2} | OCI Cloud-HSM | in-pg-patroni-{1,2} | Conditional (DPDPA 2023) |
| pack-br | OCI sa-saopaulo-1 + sa-vinhedo-1 | br-openbao-{1,2} | OCI Cloud-HSM | br-pg-patroni-{1,2} | Conditional (LGPD) |
| pack-ae | OCI me-abudhabi-1 + me-dubai-1 | ae-openbao-{1,2} | OCI Cloud-HSM (UAE-resident) | ae-pg-patroni-{1,2} | Conditional |
| pack-ksa | OCI me-jeddah-1 + me-riyadh-1 | ksa-openbao-{1,2} | OCI Cloud-HSM (KSA-resident; NCA-aligned) | ksa-pg-patroni-{1,2} | Conditional (KSA PDPL + NCA) |

The "Activated?" column is updated at first-tenant onboarding per pack; activation triggers re-review of this document + the per-pack threat-model overlay + DPIA overlay.

### Pack-assignment routing

```text
Tenant onboarding
    ↓
gtm-customer-success: collects tenant's HQ + regulated-data declarations
    ↓
Pack-router (Cedar policy at microservices/cloud-secrets/policy/pack-routing.cedar):
    - HQ jurisdiction → primary pack
    - Regulated-data flag (PHI / FSS / EU-resident) → may force secondary
    - Conflict: ops-legal escalation
    ↓
per-tenant-namespace-controller: provisions namespace in pack-pinned OpenBao
    ↓
HSM partition: KEK generated in pack-pinned HSM; never exits
    ↓
SDK consumers in workload µservices: configured with pack-pinned OpenBao endpoint
    ↓
All secret reads + writes scoped to pack; never cross-pack
```

## Cross-Pack Replication Policy

### Default: forbidden

Cross-pack replication of any of the following is forbidden by default:

| Asset | Cross-pack replication |
|---|---|
| OpenBao KV entries | FORBIDDEN |
| OpenBao Transit DEKs | FORBIDDEN |
| HSM KEK material | FORBIDDEN (KEK never leaves HSM partition) |
| Postgres backups | FORBIDDEN |
| Audit-chain events for secret-access | FORBIDDEN (each pack has its own audit-chain instance) |
| Rotation policy definitions | Configuration is per-pack (Helm values + git overlays); rotation state per-pack |
| HSM attestation reports | per-pack; audit-chain seal pack-local |
| encryption-key BYOK material (ADR-0251 §D-10) | FORBIDDEN |

### Exception: tenant-executed SCCs (GDPR Art. 44–46)

Cross-border transfer of EU-resident data is permitted only when the tenant has executed an active Standard Contractual Clause or equivalent mechanism. The exception requires:

1. Active SCC on file at `microservices/cloud-secrets/legal/transfer-register.md`.
2. Receiving-pack jurisdiction has adequate-decision (Art. 45) or equivalent safeguard.
3. Transfer-purpose limited to specifically-named processing (e.g., "DR failover within EU pair eu-frankfurt-1 → eu-amsterdam-1"); ad-hoc transfer NOT authorised.
4. Audit-chain-emitted SCC-acknowledgement at every transfer event.
5. Per ADR-0131 + this contract, cross-pack movement of `SECRET`-class material is treated as a Sev-1 incident even with SCC — SCCs apply to processed personal data, NOT to raw key material.

### Exception: BCDR exercise (controlled, intra-pack)

Intra-pack BCDR drills (e.g., eu-frankfurt-1 → eu-amsterdam-1, us-ashburn-1 → us-phoenix-1, au-sydney-1 → au-melbourne-1) are permitted. Cross-pack BCDR is NEVER authorised.

### Exception: HIPAA BAA + DR failover (pack-us-healthcare)

Covered Entity tenants may have DR pair us-ashburn-1 + us-phoenix-1; failover is intra-region from HIPAA standpoint (both HIPAA-eligible). Cross-region (us-hc → us) requires separate tenant agreement; cross-pack (us-hc → eu) is forbidden.

## Retention by Jurisdiction × Asset Class

Retention is MAX of (asset class default, pack legal minimum, tenant-contracted DPA).

| Pack | Asset class | Statutory minimum | Default applied |
|---|---|---|---|
| pack-kr | `SECRET` (KV v2) | per rotation policy | 30d API key, 90d signing key, 365d KEK |
| pack-kr | `AUDIT` | PIPA Enforcement Decree Art. 30: ≥ 1y | ≥ 3y (KR-FSS sector 5y) |
| pack-kr | `SENSITIVE_PIPA_ART23` (tenant_id) | per tenant lifecycle | salted-hash; raw only in OpenBao |
| pack-eu | `SECRET` | per Art. 25 + 32 | per rotation; cryptographic-erasure on offboard |
| pack-eu | `AUDIT` | bounded by purpose (ROPA) | 2y default; 6y if pack-eu + financial-services overlay |
| pack-us-healthcare | `SECRET` | per HIPAA §164.312 | per rotation |
| pack-us-healthcare | `AUDIT` | HIPAA §164.316(b)(2): 6y | 6y |
| pack-us | `AUDIT` (PCI-DSS overlay) | PCI-DSS v4.0 §10.5.1: ≥ 1y; 3mo immediately available | ≥ 1y |
| pack-in | `SECRET` | per DPDPA 2023 §8 | per rotation |
| pack-au | `AUDIT` (APRA-CPS 234 overlay) | APRA-CPS 234: 7y on incident-related records | 7y |
| pack-ksa | `AUDIT` (SAMA Cybersecurity overlay) | SAMA: ≥ 5y | 7y |

The CI lane `governance-retention-conformance` validates OpenBao + Postgres + audit-chain retention against this table.

## KEK Lifecycle by Pack

| Pack | KEK location | KEK rotation cadence | Attestation cadence |
|---|---|---|---|
| pack-kr | Thales Luna HSM partition (KR-resident) | 365d + on-demand | 24h |
| pack-eu | OCI Cloud-HSM (EU-resident) | 365d | 24h |
| pack-us | OCI Cloud-HSM | 365d | 24h |
| pack-us-healthcare | OCI Cloud-HSM (HIPAA-eligible) | 365d + per HIPAA security review | 24h |
| pack-jp | OCI Cloud-HSM | 365d | 24h |
| pack-sg | OCI Cloud-HSM | 365d | 24h |
| pack-au | OCI Cloud-HSM (AU-resident) | 365d | 24h |
| pack-in | OCI Cloud-HSM | 365d | 24h |
| pack-br | OCI Cloud-HSM | 365d | 24h |
| pack-ae | OCI Cloud-HSM (UAE-resident) | 365d + per UAE PDPL review | 24h |
| pack-ksa | OCI Cloud-HSM (KSA-resident; NCA-aligned) | 365d + per SAMA review | 24h |

KEK rotation cascades through DEKs without secret-value access (KEK rotation re-wraps DEKs only).

## DSR + Tenant Offboard Cascade

Right-to-erasure (GDPR Art. 17 / PIPA Art. 36 / DPDPA §12 / LGPD Art. 18(VI)) honoured via the `dsr-cascade-runner` skill applied to cloud-secrets:

1. Tenant raises offboarding request.
2. `per-tenant-namespace-controller` seals the namespace (no further reads).
3. 30-day grace (configurable per pack DPA): namespace remains sealed but recoverable.
4. After grace: cryptographic erasure — DEKs destroyed (via OpenBao Transit `delete` + KEK rotation that abandons the old wrapping). Even if a hypothetical attacker captured ciphertext, recovery is cryptographically infeasible.
5. Audit-chain seal: `tenant_namespace_cryptographically_erased{tenant_id_hash, executed_at, dek_count_destroyed}`.
6. Audit log retained per pack retention (the audit references the tenant_id_hash; the raw tenant_id is destroyed).

Note: DEK destruction = data erasure for ciphertext-at-rest. For decrypted data already exfiltrated by consumer µservices, those µservices' own DSR cascades apply.

## Per-Pack Overlay Sections

### pack-kr (KR PIPA + PIPC + KR-FSS)

- **PIPA Art. 28 (storage period limitation)**: cryptographic-erasure on offboard.
- **PIPA Art. 23-2 (sensitive cross-border)**: forbidden; SCCs do not apply to KEK material.
- **PIPC Notice 2020-7 (overseas-transfer notification)**: pack-kr residency guarantee acknowledged in tenant DPA.
- **KR-FSS sector**: audit retention ≥ 5y; KEK in KR-resident HSM (Thales Luna preferred).
- **KR NCA cryptography guidance**: aligned cipher choices (AES-256-GCM, ChaCha20-Poly1305, RSA-4096, ECDSA P-384, Ed25519).

### pack-eu (GDPR + EDPB + Schrems II)

- **GDPR Art. 44–46**: SCCs apply only to *processed personal data*, not to raw KEK or secret material; cross-pack KEK transfer prohibited absolutely.
- **EDPB Recommendations 01/2020**: supplementary measures — pseudonymisation (salted-hash tenant_id) + EU-resident KEK encryption — documented in `legal/schrems-supplementary-measures.md` (Slice D).
- **eIDAS Art. 24 (qualified signature)**: HSM-backed Ed25519 signing supports qualified-signature workflows.

### pack-us-healthcare (HIPAA)

- **45 CFR §164.312(a)(2)(iv)**: encryption — HSM-backed KEK; AES-256-GCM at rest; TLS 1.3 in transit.
- **45 CFR §164.530(j)**: 6y retention on audit + policy.
- **HIPAA-eligible regions only**: us-ashburn-1 (OCI HIPAA-attested).
- **BAA-required**: tenant must sign BAA before pack-us-healthcare ingest enabled.

### pack-us (PCI-DSS overlay if payment data; SOC 2; NIST 800-53)

- **PCI-DSS v4.0 §3.5, §3.6, §3.7**: key management lifecycle.
- **PCI-DSS §10.5**: audit retention ≥ 1y; 3mo immediately available.
- **NIST SP 800-57 Part 1**: KEK rotation cadence aligned.

### pack-in (DPDPA + RBI)

- **DPDPA 2023 §8(5)**: reasonable security safeguards per threat-model.
- **RBI Master Direction on IT Governance §6.4**: cryptographic controls.
- **In-country storage**: per RBI directive on financial data.

### pack-br (LGPD + BACEN)

- **LGPD Art. 46**: security measures per threat-model.
- **BACEN Res. 4.893/2021 §29**: cryptographic controls.

### pack-au (Privacy Act + APRA-CPS 234)

- **APRA-CPS 234 §29–36**: information security capability; HSM aligned.
- **Privacy Act APP 11**: reasonable steps to protect; documented per threat-model.

### pack-jp / pack-sg / pack-ae / pack-ksa

Each pack's overlay at `regional-packs/<pack>/cloud-secrets-residency-overlay.md` carries the local data-residency law's citations.

## Verification

```bash
cargo run -p dev-cli -- gate validate retention-conformance --microservice cloud-secrets
cargo run -p dev-cli -- gate validate pack-routing-conformance --microservice cloud-secrets
cargo run -p dev-cli -- gate validate cross-pack-replication-forbidden --microservice cloud-secrets
```

- Annual residency audit: confirm each tenant's secrets reside in the assigned pack.
- Quarterly drill: induce a cross-pack write; verify rejection + alert.
- Quarterly HSM attestation review: verify reports from every pack.

## References

- ADR-0117 (Cloud-native infrastructure)
- ADR-0131 (Cloud split)
- `microservices/cloud-secrets/threat-model.md`
- `microservices/cloud-secrets/dpia.md`
- `microservices/cloud-secrets/policy/secret-isolation.md`
- `microservices/cloud-secrets/multi-region.md`
- `microservices/cloud-secrets/legal/{transfer-register, schrems-supplementary-measures, baa-template, dpa-template, sub-processors, ropa}.md` (Slice D)
- GDPR Arts. 44–50
- EDPB Recommendations 01/2020
- KR PIPA Arts. 23-2 + 28 + 29
- HIPAA 45 CFR §164.312 + §164.530(j)
- LGPD Art. 16 + Art. 33 + Art. 46
- DPDPA 2023 §8 + §10
- PCI-DSS v4.0 §3.5, §3.6, §3.7, §10.5
- SAMA Cybersecurity Framework 2017
- KSA NCA ECC-1:2018
