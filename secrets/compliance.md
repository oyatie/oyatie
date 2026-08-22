---
doc_class: Compliance
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: ops-legal + council-privacy + ops-security
deciders: ops-legal, ops-security, council-privacy, council-architecture, axis-cloud-secrets
review_cadence: annually + on every regulation change + on every audit cycle
related_artifacts:
  - microservices/cloud-secrets/threat-model.md
  - microservices/cloud-secrets/dpia.md
  - microservices/cloud-secrets/policy/data-residency.md
  - microservices/cloud-secrets/policy/secret-isolation.md
doc_status: published
---

# Compliance: cloud-secrets µservice

This document maps cloud-secrets controls to legal + regulatory frameworks across active and conditional packs. Each citation is concrete and links to the artifact that satisfies it.

## Framework Index

- §1: KR PIPA (pack-kr) — Art. 29 + Enforcement Decree
- §2: HIPAA (pack-us-healthcare) — §164.312(a)(2)(iv) encryption + §164.308 + §164.316
- §3: GDPR (pack-eu) — Arts. 25 + 28 + 30 + 32 + 33 + 35
- §4: PCI-DSS v4.0 (pack-us + pack-kr when payment) — §3.5 + §3.6 + §3.7 + §8.6 + §10.5
- §5: SOC 2 Type 2 — CC6.1-8 + CC7.1-5 + A1.1-2
- §6: ISO 27001:2022 — A.5 + A.8 controls applicable
- §7: NIST SP 800-57 (Key Management — General)
- §8: FIPS 140-3 (Cryptographic Module Validation)
- §9: LGPD (pack-br) — Art. 46 + 48 + 50
- §10: APPI (pack-jp) — Art. 23
- §11: PDPA + MAS-TRM (pack-sg) — §24 + §9
- §12: Privacy Act + APRA-CPS 234 (pack-au) — APP 11 + §29-36
- §13: DPDPA + RBI (pack-in) — §8 + §6.4
- §14: UAE PDPL (pack-ae) — Art. 20
- §15: KSA PDPL + SAMA + NCA ECC (pack-ksa) — Art. 19 + §4.3.4 + ECC-1:2018
- §16: NIS2 + eIDAS (pack-eu critical-entities) — Art. 21(2)(h)

## §1: KR PIPA (pack-kr)

| Article | Requirement | Control | Evidence |
|---|---|---|---|
| Art. 23 | Sensitive personal data — explicit consent + extra safeguards | tenant_id treated as sensitive; salted-hash; explicit tenant DPA consent for processing | `policy/data-residency.md` §"pack-kr"; `dpia.md` §"R-01" |
| Art. 23-2 | Cross-border transfer of sensitive data — forbidden without explicit consent | cross-pack replication forbidden; SCC does not authorise sensitive-data KEK transfer | `policy/data-residency.md` "Cross-Pack Replication Policy" |
| Art. 28 | Storage period limitation | rotation cadence + cryptographic-erasure on tenant offboard | `policy/secret-isolation.md` §"TI-04"; `policy/data-residency.md` "DSR + Tenant Offboard Cascade" |
| Art. 29 | 안전성 확보조치 (safety control measures) | encryption (HSM-backed KEK + AES-256-GCM), access control (per-tenant + per-µservice scope), audit (audit-chain Ed25519-sealed) | `threat-model.md`; all mitigations |
| Enforcement Decree Art. 30 | Audit retention ≥ 1y | audit-chain ≥ 3y default (pack-kr); KR-FSS ≥ 5y | `policy/data-residency.md` "Retention by Jurisdiction" |
| Art. 33 | 개인정보 영향평가 (PIA) | full DPIA | `dpia.md` |
| Art. 36 | Right to deletion | DSR cascade | `policy/data-residency.md` "DSR + Tenant Offboard Cascade" |
| PIPC Notice 2020-7 | Overseas-transfer notification | pack-kr residency guarantee in tenant DPA | `dpia.md` §"R-06"; tenant DPA template |

## §2: HIPAA (pack-us-healthcare)

| Citation | Requirement | Control | Evidence |
|---|---|---|---|
| §164.308(a)(1)(ii)(D) | Information system activity review | audit-chain queryable by ops-security + auditors | `policy/auditor-scope.cedar` |
| §164.308(a)(3) | Workforce security | per-µservice SPIFFE scope; 4-eye break-glass | `threat-model.md` T-E-02 |
| §164.308(a)(4) | Information access management | per-tenant namespace + per-µservice scope | `policy/secret-isolation.md` |
| §164.308(a)(5)(ii)(D) | Password management | no passwords; OIDC + JIT short-lived; HSM-backed signing | `threat-model.md` |
| §164.312(a)(2)(iv) | Encryption + decryption | HSM-backed KEK; AES-256-GCM at rest | `policy/data-residency.md` "KEK Lifecycle"; `threat-model.md` T-I-04 |
| §164.312(b) | Audit controls | audit-emitter → audit-chain (Ed25519 + Merkle) | `threat-model.md` T-R-01 |
| §164.312(e)(2)(ii) | Encryption in transit | mTLS everywhere; TLS 1.3 | `iac/helm/openbao/values.yaml`; SDK configuration |
| §164.314 | Organizational — BAA | tenant BAA template | `legal/baa-template.md` (Slice D) |
| §164.316(b)(2) | Documentation retention 6y | audit-chain 6y retention in pack-us-healthcare | `policy/data-residency.md` |
| §164.530(j) | Retention 6y | per above | per above |

## §3: GDPR (pack-eu)

| Article | Requirement | Control | Evidence |
|---|---|---|---|
| Art. 5(1)(c) | Data minimisation | only salted-hash tenant_id + SPIFFE id + path-hash carried | `dpia.md` §"2.2 Data Inventory" |
| Art. 5(1)(f) | Integrity + confidentiality | HSM + mTLS + Ed25519-sealed audit | `threat-model.md` |
| Art. 25 | Data protection by design + default | default-deny Cedar; salted-hash; LEAN-A11 BLOCKER | `policy/*.cedar`; `policy/secret-isolation.md` |
| Art. 28 | Processor obligations | tenant DPA + sub-processor enumeration | `legal/{dpa-template,sub-processors}.md` |
| Art. 30 | Records of processing | ROPA at `legal/ropa.md` | `legal/ropa.md` (Slice D) |
| Art. 32(1)(a) | Pseudonymisation + encryption | salted-hash tenant_id + HSM-KEK-encrypted at rest | per above |
| Art. 32(1)(b) | Confidentiality + integrity + availability + resilience | threat-model mitigations | `threat-model.md` |
| Art. 33 | Breach notification within 72h | Sev-1 incident → tenant + DPA within 72h | `incident-response.md` |
| Art. 35 | DPIA | this DPIA | `dpia.md` |
| Arts. 44-50 | Transfer mechanisms | per-pack residency; SCC does not authorise KEK transfer | `policy/data-residency.md` |

## §4: PCI-DSS v4.0 (pack-us + tenants with payment data)

| Requirement | Control | Evidence |
|---|---|---|
| §3.5.1 | Render PAN unreadable (encryption) | applicable to consumers; cloud-secrets provides KEK + DEK lifecycle | `policy/data-residency.md` "KEK Lifecycle" |
| §3.5.2 | Strong cryptography | AES-256-GCM + RSA-4096 + ECDSA P-384 + Ed25519 | crypto choices in `threat-model.md` |
| §3.6.1 | Documented key management | this document + rotation policy + `runbooks/hsm-key-rotation.md` | per above |
| §3.6.4 | Key rotation per cryptoperiod | rotation scheduler enforces; cascade-rotation | `IP-010` |
| §3.6.7 | Key compromise procedure | revoke + cascade-rotate within ≤5s | `runbooks/secret-leak-detected.md` |
| §3.7 | Key management lifecycle | per `runbooks/hsm-key-rotation.md` | per above |
| §8.6 | Strong cryptography for credentials | HSM-backed signing | per `threat-model.md` |
| §10.2 | Audit log of access | audit-emitter + audit-chain | `threat-model.md` T-R-01 |
| §10.5.1 | Audit retention ≥ 1y; 3mo immediately available | audit-chain retention ≥ 1y; hot-storage 3mo | `policy/data-residency.md` |

## §5: SOC 2 Type 2

| Trust Service Criterion | Requirement | Control | Evidence |
|---|---|---|---|
| CC6.1 | Logical + physical access controls | Cedar default-deny + SPIFFE + HSM physical isolation | `policy/*.cedar`; `threat-model.md` |
| CC6.2 | Authentication | OIDC + MFA + JIT + SPIFFE for workloads | per above |
| CC6.3 | Authorization | per-tenant + per-µservice scope | `policy/secret-isolation.md` |
| CC6.6 | Transmission of confidential info | TLS 1.3 + mTLS | `iac/helm/openbao/values.yaml` |
| CC6.7 | Disposal of confidential info | cryptographic-erasure on offboard | `policy/data-residency.md` "DSR" |
| CC6.8 | Anti-malware + integrity | container scanning (Trivy) + signed images (cosign) | `iac/helm/*/values.yaml` (cosign) |
| CC7.1 | System monitoring | audit-chain + observability SLO | per above |
| CC7.2 | Anomalous activity | audit-emit `cross_*_attempt`; alarms | `threat-model.md` mitigations |
| CC7.3 | Security incidents | `incident-response.md` Sev ladder | per above |
| CC7.4 | Incident communication | Sev-1 → tenant ≤72h, regulator ≤24h | per above |
| CC7.5 | Recovery + restoration | runbooks + drills | `runbooks/*` |
| CC8.1 | Change management | PR-review + LEAN gates + reviewer-agent | per governance µservice |
| A1.1 | Capacity | `capacity-model.md` | per above |
| A1.2 | Availability | SLOs in `microservices/cloud-secrets/slos/*` | per above |

## §6: ISO 27001:2022 (Annex A controls applicable)

| Control | Title | Control | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | quarterly threat-model review + LEAN-A11 pattern updates | `threat-model.md` review cadence |
| A.5.10 | Acceptable use of info + assets | tenant DPA + operator-acceptable-use docs | `legal/dpa-template.md` |
| A.5.14 | Information transfer | mTLS + TLS 1.3 + signed audit events | per threat-model |
| A.5.15 | Access control | Cedar default-deny + OpenBao policy + SPIFFE | per policy artifacts |
| A.5.16 | Identity management | OIDC + SPIRE issuance | per threat-model |
| A.5.17 | Authentication information | rotation per ISO cadence; HSM-backed | `policy/data-residency.md` "KEK Lifecycle" |
| A.5.18 | Access rights | per-µservice scope; revocation push | per above |
| A.5.19-23 | Supplier relationships | sub-processor enumeration; HSM vendor SLA | `legal/sub-processors.md` |
| A.5.26 | Response to incidents | `incident-response.md` | per above |
| A.5.28 | Collection of evidence | audit-chain Merkle + Ed25519 non-repudiation | per above |
| A.5.30 | ICT readiness for business continuity | `multi-region.md`; DR drills | per multi-region |
| A.5.31-33 | Legal + compliance | this document + DPIA | per above |
| A.8.2 | Privileged access rights | 4-eye break-glass; JIT elevation | `threat-model.md` T-E-02 |
| A.8.3 | Information access restriction | per-tenant + per-µservice | per policy |
| A.8.5 | Secure authentication | mTLS + OIDC + MFA | per above |
| A.8.7 | Protection against malware | Trivy scans; cosign | per above |
| A.8.10 | Information deletion | DSR cascade | per data-residency |
| A.8.11 | Data masking | salted-hash tenant_id; `Secret<T>` newtype | per threat-model |
| A.8.12 | Data leakage prevention | LEAN-A11 BLOCKER + Loki redaction | `policy/secret-isolation.md` |
| A.8.13 | Information backup | encrypted Postgres backups | `iac/helm/postgres/values.yaml` |
| A.8.15 | Logging | audit-chain | per above |
| A.8.16 | Monitoring | observability SLOs + alarms | per above |
| A.8.20 | Networks security | mTLS + NetworkPolicy | `iac/kustomize/base/` |
| A.8.21 | Security of network services | per above | |
| A.8.23 | Web filtering | Envoy / Istio gateway with WAF | per `iac` |
| A.8.24 | Use of cryptography | HSM + AES-256-GCM + Ed25519 | per data-residency "KEK Lifecycle" |
| A.8.25-28 | Secure development lifecycle | LEAN gates + reviewer-agent + threat-model | per governance |
| A.8.30 | Outsourced development | sub-processor diligence | `legal/sub-processors.md` |

## §7: NIST SP 800-57 Part 1 (Key Management — General)

Key management lifecycle aligned with NIST SP 800-57:

- **Pre-activation**: KEK generated in HSM ceremony with 4-eye witness; attestation captured.
- **Active**: rotation cadence per pack + cryptoperiod (KEK 1y, signing 90d, API 30d).
- **Suspended**: revoke + cascade-rotate; audit-chain seal.
- **Deactivated / Destroyed**: cryptographic erasure on tenant offboard; HSM partition destroy.
- **Compromised**: incident response per `incident-response.md` Sev-1.

## §8: FIPS 140-3 (Cryptographic Module Validation)

| Requirement | Control |
|---|---|
| Cryptographic module validation | OCI Cloud-HSM (FIPS 140-3 Level 3) + Thales Luna (FIPS 140-3 Level 3) |
| Approved algorithms | AES-256-GCM, RSA-4096, ECDSA P-384, Ed25519 — all FIPS-approved |
| Key generation in approved module | HSM-side generation only |
| Attestation | daily attestation report; failure pages |

## §9-15: Other pack-specific frameworks

Detailed mappings live in `regional-packs/<pack>/cloud-secrets-compliance-overlay.md` per pack activation. Summary:

| Pack | Lead framework | Key citations |
|---|---|---|
| pack-br | LGPD | Art. 46 + 48 + 50 |
| pack-jp | APPI | Art. 23 |
| pack-sg | PDPA + MAS-TRM | §24 + §9 |
| pack-au | Privacy Act + APRA-CPS 234 | APP 11 + §29-36 |
| pack-in | DPDPA + RBI | §8 + §6.4 |
| pack-ae | UAE PDPL | Art. 20 |
| pack-ksa | KSA PDPL + SAMA + NCA | Art. 19 + §4.3.4 + ECC-1:2018 |

## §16: NIS2 + eIDAS (pack-eu critical-entities)

| Citation | Requirement | Control |
|---|---|---|
| NIS2 Art. 21(2)(h) | Cryptography | per FIPS + threat-model |
| eIDAS 910/2014 Art. 24 | Qualified signature | HSM-backed Ed25519 supports qualified-signature workflows |

## Verification

```bash
cargo run -p dev-cli -- gate validate compliance-mapping --microservice cloud-secrets
cargo run -p dev-cli -- gate validate retention-conformance --microservice cloud-secrets
cargo run -p dev-cli -- gate validate authority-cohesion
```

Annual third-party audit:
- SOC 2 Type 2 (annual)
- ISO 27001:2022 (annual surveillance + tri-annual recertification)
- PCI-DSS QSA (annual for tenants with payment data)
- HIPAA security assessment (per BAA cadence)
- Pack-specific regulator inspections per pack activation

## References

- `microservices/cloud-secrets/threat-model.md`
- `microservices/cloud-secrets/dpia.md`
- `microservices/cloud-secrets/policy/data-residency.md`
- `microservices/cloud-secrets/policy/secret-isolation.md`
- `microservices/cloud-secrets/incident-response.md`
- `microservices/cloud-secrets/runbooks/*.md`
- `microservices/cloud-secrets/legal/*.md` (Slice D)
- KR PIPA + Enforcement Decree
- HIPAA 45 CFR §164
- GDPR Regulation (EU) 2016/679
- PCI-DSS v4.0
- ISO 27001:2022 + ISO 27002:2022
- NIST SP 800-57 Part 1 Rev. 5
- FIPS 140-3
- NIS2 Directive (EU) 2022/2555
- eIDAS Regulation (EU) 910/2014
- LGPD Lei 13.709/2018
- APPI 2003 (as amended 2022)
- PDPA 2012 (SG) + MAS-TRM v2021
- Privacy Act 1988 (AU) + APRA-CPS 234
- DPDPA 2023 (IN) + RBI Master Direction
- UAE PDPL Federal Decree-Law No. 45/2021
- KSA PDPL Royal Decree M/19/2021 + SAMA Cybersecurity 2017 + NCA ECC-1:2018

---



## §day-one-cert-readiness
This anchor is closed for `cloud-secrets` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `cloud-secrets` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +9 more.
- Example: `audit-query` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pack-overlay-roster
This anchor is closed for `cloud-secrets` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `audit-query` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §bootstrap-trust-chain
This anchor is closed for `cloud-secrets` against ADR-0295 §D-2: Tier-1 bootstrap SPIFFE attestation and kill switch.

### Service-specific answer
- Bootstrap trust applies to `cloud-secrets` control-plane deployment, CI principals, and first-run OpenBao/SPIFFE bindings.
- Stage-1 trust root is offline-rooted and time-boxed; the kill switch disables bootstrap trust after the declared window even if later stages fail.
- Workload SVIDs protect API/worker surfaces for `cloud-secrets`.
- CI principals can run synthetic tests and publish evidence, but cannot read production tenant data or mint tenant-scoped credentials.
- Example: `audit-query` app pod starts only after SPIFFE identity, OpenBao policy, and Cedar CI-scope permits are all present.
- Bootstrap failures default to halt: no unauthenticated fallback and no long-lived bootstrap token.
- Evidence: sigstore/cosign attestation, audit-chain bootstrap event, branch-protection gate, and SLO smoke report.
- Tier-1 bootstrap status is listed here even for non-bootstrap services so auditors know whether the service inherits or owns the ceremony.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: SPIFFE/SPIRE workload identity is the reference pattern for the control shape described here.
- Precedent 2: AWS Nitro Enclaves attestation is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §platform-owner-indirection
This anchor is closed for `cloud-secrets` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `cloud-secrets` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`; +15 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `audit-query` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.cloud-secrets.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-substrate-binding
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `cloud-secrets` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `audit-query` touches those data classes.
- Signal sources: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`; +9 more.
- Example event class: `oya.cloud.secrets.audit.query.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §investigation-binding
This anchor is closed for `cloud-secrets` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `cloud-secrets` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.cloud-secrets.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `audit-query` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `audit-query` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §insider-threat-controls
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `cloud-secrets` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `cloud_secrets.cloud_secrets`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `cloud_secrets.cloud_secrets` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §threat-intelligence-feeds
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `cloud-secrets` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`; +10 more.
- Example: `audit-query` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §key-rotation-cadence
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.cloud-secrets` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/cloud-secrets/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.
- Example: `audit-query` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §crypto-agility-plan
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `cloud-secrets` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/iac/helm/hsm-operator/Chart.yaml`, `microservices/cloud-secrets/iac/helm/hsm-operator/values.yaml`, `microservices/cloud-secrets/iac/helm/openbao/Chart.yaml`; +7 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `audit-query` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pentest-and-bounty-cadence
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `cloud-secrets` is in annual full-scope pentest and every major `audit-query` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`, `microservices/cloud-secrets/iac/helm/hsm-operator/Chart.yaml`, `microservices/cloud-secrets/iac/helm/hsm-operator/values.yaml`, `microservices/cloud-secrets/iac/helm/openbao/Chart.yaml`; +13 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `cloud-secrets` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §facility-controls
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `cloud-secrets` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `audit-query` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §supply-chain-risk
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `cloud-secrets` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/cloud-secrets/catalog/cloud-secrets-audit-emitter-adapter-audit-chain-bridge.yaml`, `microservices/cloud-secrets/catalog/cloud-secrets-audit-emitter-api.yaml`, `microservices/cloud-secrets/catalog/cloud-secrets-audit-emitter-app.yaml`, `microservices/cloud-secrets/catalog/cloud-secrets-audit-emitter-kernel.yaml`, `microservices/cloud-secrets/catalog/cloud-secrets-audit-emitter-usecase.yaml`, `microservices/cloud-secrets/catalog/cloud-secrets-hsm-integration-adapter-hsm.yaml`; +19 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `audit-query` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `cloud-secrets` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `audit-query` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `audit-query` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §data-classification
This anchor is closed for `cloud-secrets` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `cloud_secrets.cloud_secrets`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `audit-query` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `cloud-secrets`; owner `axis-cloud-secrets`; tier `substrate`; audience `B2B_TENANT + INTERNAL_OPERATOR`.
- Bounded contexts used for this answer: `cloud-secrets`.
- Capability records cited: `microservices/cloud-secrets/capabilities/audit-query.yaml`, `microservices/cloud-secrets/capabilities/secret-reference-resolve.yaml`, `microservices/cloud-secrets/capabilities/secret-rotate.yaml`.
- API surfaces cited: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`; +10 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-secrets/contracts/asyncapi/cloud-secrets-events.yaml`, `microservices/cloud-secrets/contracts/openapi/cloud-secrets.yaml`, `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto`.
- Cedar binding: `microservices/cloud-secrets/policy/auditor-scope.cedar`, `microservices/cloud-secrets/policy/ci-scope.cedar`, `microservices/cloud-secrets/policy/data-residency.md`, `microservices/cloud-secrets/policy/public-read.cedar`, `microservices/cloud-secrets/policy/secret-isolation.md`, `microservices/cloud-secrets/policy/tenant-scope.cedar`.
- State/event binding: `cloud_secrets.cloud_secrets`.
- Capability binding: `audit-query`, `secret-reference-resolve`, `secret-rotate`.
- SLO binding: `microservices/cloud-secrets/slos/audit-log-completeness.openslo.yaml`, `microservices/cloud-secrets/slos/hsm-availability.openslo.yaml`, `microservices/cloud-secrets/slos/key-rotation-correctness.openslo.yaml`, `microservices/cloud-secrets/slos/pki-cert-issuance-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-resolve-latency.openslo.yaml`, `microservices/cloud-secrets/slos/secret-write-latency.openslo.yaml`.
- Runbook binding: `microservices/cloud-secrets/runbooks/audit-emission-backlog.md`, `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`, `microservices/cloud-secrets/runbooks/namespace-controller-restart.md`, `microservices/cloud-secrets/runbooks/openbao-restart.md`, `microservices/cloud-secrets/runbooks/rotation-cascade-recovery.md`, `microservices/cloud-secrets/runbooks/secret-leak-detected.md`.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-secrets`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-secrets`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-secrets` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-secrets` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-secrets`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation targets observable, versioned, reversible, tenant-scoped control evidence; this is target_non_claim until service-specific tests and cloud-ci evidence prove the property.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-secrets` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `governance-abuse-defence-ux-floor` and `governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.
