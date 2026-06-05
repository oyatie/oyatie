---
doc_class: PolicyDocument
title: Data Residency Policy
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + council-privacy + ops-compliance
deciders: ops-security, council-privacy, council-architecture
related_adrs: [ADR-0117, ADR-0131]
related_artifacts:
  - microservices/governance/dpia.md
  - microservices/governance/compliance.md
  - microservices/governance/iac/kustomize/overlays/pack-kr/kustomization.yaml
review_cadence: quarterly + on every new pack onboarding
doc_status: published
---

# Data Residency Policy: governance µservice

## Purpose

Define where governance µservice data MAY be stored, processed, and read; enforce per-pack regulatory residency requirements; specify cross-pack transfer mechanisms.

## Data inventory + residency

Per `dpia.md` §3 data inventory + `threat-model.md` §"Assets & Data Classification". The following rows govern residency-bound data:

| Asset | Per-pack residency rule | Mechanism |
|---|---|---|
| Findings (Postgres `findings` table) | MUST reside in-pack (e.g., pack-kr findings in OCI ap-seoul-1) | Per-pack Postgres replica; per-pack key prefix on logical replication |
| Evidence blobs (S3) | MUST reside in-pack S3 bucket | Per-pack OCI Object Storage bucket; bucket region-locked at creation |
| Audit-chain seals | MAY replicate cross-pack with cryptographic guarantees | Ed25519 + Merkle; original seal in-pack; quarterly root may aggregate cross-pack |
| PR-author identity (Postgres) | MUST reside in-pack | Per-pack Postgres |
| Rule packs (git repo) | Global; not residency-bound | Code; pseudonymous; no PII |
| Industry-baseline pins | Global; not residency-bound | Code |
| Aggregation indices (`docs/prds/INDEX.md` etc.) | Global; not residency-bound | Code; non-PII |
| OpenBao secrets | Per-pack OpenBao | Per-pack instance; KMS in-pack |
| External-auditor JIT tokens | Per-pack OpenBao | Issued in-pack; valid only against in-pack endpoints |

## Per-pack overrides

### pack-kr (KR PIPA + KR commercial code)

- All Finding + evidence + PR-author identity data MUST reside in KR (OCI ap-seoul-1 region).
- KMS keyring MUST be in KR (per OCI KMS regional availability).
- Cross-border transfer: refused by default; allowed only via Art. 28 KR PIPA contractual basis + per-tenant consent + Korea-specific SCCs.
- Retention: 5y for commercial-code-bound records; aligned with `dpia.md` §1 retention.
- Sensitive-info (RRN, fingerprint, health info per KR PIPA Art. 23): refused at sanitiser stage; lane refuses to emit Findings containing such fields.

Implementation: `iac/kustomize/overlays/pack-kr/kustomization.yaml` pins:
- Postgres replica region = ap-seoul-1
- S3 bucket region = ap-seoul-1
- KMS keyring = oci.kms.ap-seoul-1
- IAM compartment scoped to KR-only operators

### pack-us-healthcare (HIPAA + HITECH)

- All Finding + evidence data containing PHI MUST reside in a HIPAA-eligible OCI region (us-ashburn-1 or us-phoenix-1).
- BAA required with controller + tenant + any downstream processor.
- Retention: 6y per HIPAA §164.316(b)(2).
- PHI scrub: evidence sanitiser pass rejects medical-record-number patterns; lane refuses Findings containing PHI.
- Isolated from non-HC pack-us (separate Postgres + S3; separate IAM; separate audit-chain seal queue).

### pack-eu (GDPR + EDPB)

- All Finding + evidence data MUST reside in EU/EEA region (default: OCI eu-frankfurt-1).
- Cross-border transfer to non-adequate jurisdictions: SCCs (2021/914) + supplementary measures + TIA (per `compliance.md` annex).
- Cross-border to KR: adequacy decision (2021/2071) suffices.
- Cross-border to JP: adequacy decision (2019/419) suffices.
- Cross-border to US: DPF participation OR SCCs.
- Retention: bounded by Art. 5(1)(e) storage limitation + audit-replay necessity; default 7y for AUDIT; non-AUDIT subject to data-subject erasure rights.

### pack-us (CCPA/CPRA + sector-specific)

- All Finding + evidence data MUST reside in US region (default: OCI us-ashburn-1).
- CCPA / CPRA: "sale of personal info" refused at policy level (no monetisation).
- VCDPA / CPA / CTDPA / UCPA: data-subject rights (access, deletion, correction, portability) via Application Shell.

### pack-jp (APPI)

- Tokyo region (OCI ap-tokyo-1).
- Sensitive personal info (APPI Art. 26-2): refused at sanitiser stage.
- Cross-border: APPI Art. 24 — consent OR equivalent-protection assessment.

### pack-sg (PDPA)

- Singapore region (OCI ap-singapore-1).
- Transfer-limitation per §26: refused absent comparable protection mechanism.

### pack-au (Privacy Act + APRA-CPS 234)

- Sydney region (OCI ap-sydney-1).
- APP 8 cross-border accountability: controller remains responsible for overseas-recipient compliance.
- APRA-CPS 234 (when tenant is APRA-regulated): additional security controls per `compliance.md`.

### pack-in (DPDPA 2023)

- Mumbai region (OCI ap-mumbai-1).
- DPDPA + sector-specific (RBI, IRDAI) cross-border restrictions vary by sector; tenant onboarding flow captures sector.

### pack-br (LGPD)

- São Paulo region (OCI sa-saopaulo-1).
- Cross-border per Art. 33: adequacy OR safeguards (SCCs equivalent).

### pack-ae (UAE PDPL)

- Dubai region (OCI me-dubai-1).
- Cross-border per UAE PDPL Art. 22: adequacy OR consent OR contractual mechanism.

### pack-ksa (PDPL + SAMA Cybersecurity Framework)

- Jeddah region (OCI me-jeddah-1).
- Cross-border per PDPL Art. 29: adequacy OR consent OR contractual mechanism.
- SAMA-CSF additional security controls for financial-sector tenants.

## Cross-pack transfer

| Transfer direction | Mechanism | Documentation requirement |
|---|---|---|
| pack-kr → pack-eu | KR-EU adequacy (2021/2071) | No additional mechanism; logged |
| pack-eu → pack-jp | EU-Japan adequacy (2019/419) | No additional mechanism; logged |
| pack-eu → pack-kr | EU-Korea adequacy (2021/2071) | No additional mechanism; logged |
| pack-eu → pack-us | DPF participant OR SCCs | DPF certificate OR SCCs filed in `compliance.md` |
| pack-eu → pack-au / pack-sg / pack-in / pack-br / pack-ae / pack-ksa | SCCs + TIA | TIA in `compliance.md` annex per pack |
| pack-us-healthcare → other packs | Refused by default | Bucket-prefix lock + IAM denial; override requires BAA chain + ops-compliance approval |
| Any pack → external-auditor (read-only) | JIT-scoped OIDC + per-audit DPA addendum | DPA addendum signed; audit window logged |

## Enforcement

| Mechanism | Where |
|---|---|
| Postgres replica region pinning | `iac/helm/postgres/values.yaml` per overlay |
| S3 bucket region-lock | `iac/terraform/<pack>/object-storage.tf` |
| KMS keyring region | `iac/terraform/<pack>/kms.tf` |
| Cedar `data-residency.cedar` | runtime ABAC at API gateway |
| Lane `oya-check-data-class` | refuses unannotated fields |
| Lane `oya-check-cross-ref-validity` | refuses cross-pack write paths |
| Quarterly residency-audit | per-pack region verification + bucket-policy review |

## Verification

- Per-pack deploy includes `microservices/governance/tests/integration/residency-<pack>.rs`.
- Quarterly: run the governance residency-audit control-plane operation for `<pack>` and attach Buck2/Prow evidence.
- On new-pack onboarding: residency policy review by council-privacy + ops-security; sign-off required before pack accepts traffic.

## References

- ADR-0117 (data-residency).
- ADR-0131 (per-microservice flat layout — pack overlays).
- KR PIPA, HIPAA, GDPR, APPI, PDPA-SG, Privacy Act + APRA-CPS 234, DPDPA, LGPD, UAE PDPL, KSA PDPL + SAMA-CSF — cited inline.
- EU SCCs 2021/914; EU adequacy decisions; EU-US DPF.
- `microservices/governance/dpia.md` §6 cross-border transfers.
- `microservices/governance/compliance.md` ROPA + TIA.
- `microservices/observability/policy/data-residency.md` (shape reference).
