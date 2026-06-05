---
doc_class: ComplianceMatrix
template_id: TPL-COMPLIANCE
microservice: drive
status: Accepted
date: 2026-05-17
owner_team: council-privacy + ops-security
related_adrs: [ADR-0028, ADR-0117, ADR-0135, ADR-0140 (retired per ADR-0145), ADR-DRIVE-0001, ADR-DRIVE-0006]
doc_status: published
---

# Compliance Matrix — drive µservice

## Purpose

Enumerate compliance frameworks engaged by drive, the controls satisfied, and where each control is evidenced (per artifact, lane, or runbook).

## Frameworks engaged

### Globally enforced

| Framework | Scope | Mapping |
|---|---|---|
| SOC 2 Type 2 (2017 TSC + 2022 PoF) | CC1.x–CC9.x | §"SOC 2 Mapping" |
| ISO 27001:2022 | Annex A.5–A.8 | §"ISO 27001 Mapping" |
| GDPR | Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44 | §"GDPR Mapping" |
| OWASP ASVS v4.0.3 | full | embedded in threat-model.md |
| CIS Kubernetes Benchmark v1.9.0 | full | enforced on every chart |
| FIPS 140-3 | KMS / OpenBao Transit cryptographic module | §"Crypto + KMS" |
| NIST SP 800-57 | key management lifecycle | §"Crypto + KMS" |
| SLSA L3 | supply chain | §"Supply Chain" |
| NIST SSDF | secure development | §"Secure Development" |

### Pack-overlays

| Pack | Frameworks |
|---|---|
| pack-kr | KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2/33 + ISMS-P §2.1-2.12 + 전자문서법 Arts. 5/6/7 + KR-FSS supervisory regulations (5y retention) |
| pack-us | CCPA / CPRA + SEC 17a-4(f) + FINRA Rule 4511 |
| pack-us-healthcare | HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502/§164.504/§164.512/§164.514 + FDA 21 CFR Part 11 + state-level (CMIA / NY SHIELD) |
| pack-eu | GDPR + EDPB Guidelines 4/2019 + 9/2022 + NIS2 2022/2555 + eIDAS 910/2014 + EU AI Act Regulation 2024/1689 |
| pack-jp | APPI Arts. 17/18/20/21/23/24/26-2/27 |
| pack-sg | PDPA 2012 Parts III/IV/VI + MAS Notice 644 |
| pack-au | Privacy Act 1988 APP 1-13 + APRA-CPS 234 |
| pack-in | DPDPA 2023 §6-11 + RBI Master Direction on IT Outsourcing |
| pack-br | LGPD Arts. 6/7/11/14/18/33/38/46/48 + BACEN Res. 4.893/2021 |
| pack-ae | UAE PDPL Federal Decree-Law 45/2021 Arts. 5/6/9/15/22/23 |
| pack-ksa | KSA PDPL Royal Decree M/19/2021 + SAMA Cybersecurity Framework 2017 |

## SOC 2 Mapping

| TSC | Control | Drive evidence |
|---|---|---|
| CC1.1 | Demonstrates commitment to integrity | council-architecture + ops-security sign-off on threat-model + DPIA |
| CC2.1 | Communicates information | This compliance.md + PRD + runbooks |
| CC3.1 | Specifies risk objectives | threat-model.md + dpia.md |
| CC4.1 | Demonstrates evaluation | audit-chain emission per file lifecycle + LEAN check coverage |
| CC4.2 | Selects monitoring activities | observability dashboards + per-changeset evidence |
| CC5.1 | Selects + develops control activities | LEAN CI lanes + Cedar policies |
| CC6.1 | Logical access — restricts | per-tenant RLS + tenant-DEK + Cedar policies |
| CC6.2 | Authenticates | OIDC + MFA + per-tenant API keys + SigV4 |
| CC6.3 | Authorises | Cedar policies (`tenant-scope.cedar`, `ci-scope.cedar`, `auditor-scope.cedar`, `public-read.cedar`) |
| CC6.6 | Restricts physical access | inherited from cloud-k8s + cloud-secrets |
| CC6.7 | Restricts info flow | type-narrowed projections + LEAN checks + gVisor sandbox |
| CC6.8 | Prevents unauthorized software | branch-protection + signed commits + signed artifacts + SLSA L3 |
| CC7.1 | Detects security events | observability alerts + audit-chain + virus scan |
| CC7.2 | Monitors system components | SLO + burn-rate dashboards |
| CC7.4 | Responds to incidents | incident-response.md + runbooks |
| CC7.5 | Recovers from incidents | DR drills + backup retention; replication-factor 3 |
| CC8.1 | Manages changes | ADR-0110 ChangeSet + ADR-0139 SLO-gated promotion |
| CC9.1 | Identifies risks | residual-risk acceptance in threat-model.md |

## ISO 27001 Mapping

| Annex A Control | Drive evidence |
|---|---|
| A.5.7 (threat intelligence) | threat-model.md re-review triggers |
| A.5.10 (acceptable use) | policy/dual-context-isolation.md |
| A.5.14 (information transfer) | policy/data-residency.md |
| A.5.15 (access control) | Cedar policies + RLS + permissions BC |
| A.5.17 (authentication info) | OpenBao + per-tenant DEK rotation |
| A.5.23 (cloud service usage) | ADR-0117 |
| A.5.26 (response to security incidents) | incident-response.md |
| A.5.27 (lessons from incidents) | post-incident review process |
| A.5.28 (collection of evidence) | audit-chain seal |
| A.5.30 (ICT continuity) | multi-region.md |
| A.5.31 (legal + statutory) | This compliance.md per-pack overlays |
| A.5.32 (intellectual property) | dependencies + licenses in `catalog/*.yaml` |
| A.5.33 (records protection) | WORM tier + legal-hold |
| A.5.34 (privacy + PII) | dpia.md + policy/* |
| A.8.2 (privileged access rights) | OpenBao JIT + 2-person rule |
| A.8.3 (info access restriction) | RLS + Cedar |
| A.8.5 (secure authentication) | OIDC + MFA |
| A.8.7 (protection against malware) | ClamAV + OPSWAT scan pipeline |
| A.8.11 (data masking) | redaction in cross-tenant share projection |
| A.8.12 (data leakage prevention) | DLP scan pipeline + LEAN checks |
| A.8.15 (logging) | observability + audit-chain |
| A.8.16 (monitoring activities) | dashboards + alerts |
| A.8.20 (network security) | mesh mTLS + NetworkPolicy |
| A.8.21 (security of network services) | per-tenant API key + rate limits |
| A.8.23 (web filtering) | WAF at ingress |
| A.8.24 (cryptography) | tenant-DEK + audit-chain Ed25519 + Argon2id + FIPS 140-3 OpenBao |
| A.8.25 (secure development lifecycle) | LEAN gates + ADR-0139 SLO-gated promotion |
| A.8.26 (application security requirements) | per-microservice security artifacts |
| A.8.27 (secure system architecture) | ADR-0056 + ADR-0105 |
| A.8.28 (secure coding) | LEAN check `oya-check-cdc-parameters-pinned` + cargo fuzz on parsers |
| A.8.32 (change management) | ADR-0110 + branch-protection |
| A.8.33 (test information) | synthetic test tenants per `ci-scope.cedar` |
| A.8.34 (audit + protection of audit systems) | audit-chain immutability + 2-person rule on admin |

## GDPR Mapping

| Article | Drive evidence |
|---|---|
| Art. 5(1)(a) lawfulness | per-purpose lawful basis in dpia.md |
| Art. 5(1)(b) purpose limitation | dpia.md §2.4 |
| Art. 5(1)(c) data minimisation | type-narrowed cross-tenant share + redaction |
| Art. 5(1)(d) accuracy | tenant-edit UX + version history |
| Art. 5(1)(e) storage limitation | retention per pack + version pruning |
| Art. 5(1)(f) integrity + confidentiality | tenant-DEK + Ed25519 audit-chain |
| Art. 6(1) lawful basis | dpia.md §2.4 |
| Art. 9 special-category | pack-us-healthcare overlay + pack-kr flagged-file |
| Art. 13/14 transparency | tenant DPA template |
| Art. 17 right-to-erasure | DSR cascade + hold-vs-erasure policy |
| Art. 22 automated decision | T1 actions reversible within window; T2 HR-context REFUSED at Cedar |
| Art. 25 by design + default | type-system separation + Cedar policy + client-side E2E opt-in |
| Art. 28 processor agreement | DPA template |
| Art. 30 records of processing | RoPA template |
| Art. 32 security of processing | every STRIDE / LINDDUN mitigation |
| Art. 33 breach notification | incident-response.md 72h chain |
| Art. 35 DPIA | dpia.md |
| Arts. 44-50 transfers | SCC + transfer register + multi-region.md |

## Crypto + KMS (FIPS 140-3 + NIST SP 800-57)

| Control | Implementation |
|---|---|
| Symmetric encryption at rest | AES-256-GCM via OpenBao Transit (FIPS 140-3 module) |
| Envelope encryption | per-tenant DEK wrapped by per-pack KEK; KEK rotation 90d, DEK rotation 90d |
| Asymmetric signature | Ed25519 (audit-chain seal + share-link signing) |
| Password KDF | Argon2id (memory_cost=64MiB, time_cost=3, parallelism=4) per RFC 9106 |
| Client-side E2E (Personal pillar opt-in) | libsodium secretstream (XChaCha20-Poly1305) |
| TLS in transit | TLS 1.3 only; mTLS internal mesh |
| Key generation | NIST SP 800-90A DRBG; HKDF for per-link signing-key derivation |
| Key lifecycle | NIST SP 800-57: generate / distribute / use / revoke / archive / destroy; OpenBao audit log retained |

## Supply Chain (SLSA L3)

| Control | Implementation |
|---|---|
| Source integrity | signed commits required; branch-protection enforced |
| Build integrity | hermetic builds; reproducible release artifacts |
| Provenance | SLSA provenance attestation per artifact (build platform, build script, builder identity) |
| Dependency management | LTS pinning enforced via `oya-governance-version-pinning-conformance` lane |
| Vendoring | cargo vendor + checksum verification for sensitive dependencies (cryptography, secret-handling) |

## Secure Development (NIST SSDF)

| Practice | Implementation |
|---|---|
| PO (Prepare Organization) | this artifact + ADR-0133 industry-conformance program |
| PS (Protect Software) | signed commits + signed artifacts + secret-scanner CI lane |
| PW (Produce Well-Secured Software) | LEAN gates + Cedar policies + threat-model coverage |
| RV (Respond to Vulnerabilities) | incident-response.md + CVE scanner + dependency-update workflow |

## Pack-overlay detail: pack-kr (KR PIPA + ISMS-P + 전자문서법 + KR-FSS)

| PIPA Article | Drive evidence |
|---|---|
| Art. 15 (consent for collection) | tenant onboarding consent flow |
| Art. 17 (cross-border transfer) | default-residency + SCC clause |
| Art. 18 (use beyond stated purpose) | dpia.md purpose-limitation §2.4 |
| Art. 22-2 (sensitive personal info, identifier-based) | flagged-file Cedar policy |
| Art. 23 (sensitive personal info) | per-file sensitivity flag + access restrictions |
| Art. 23-2 (cross-border sensitive) | pack-pinning + SCC |
| Art. 24 (uniquely identifying) | hashed tenant ID + salt rotation |
| Art. 28 (storage period) | retention bounded per asset table; WORM tier for FSS tenants |
| Art. 29 (technical safeguards) | 12-safeguard mapping in threat-model.md |
| Art. 29-2 (data leakage prevention) | LEAN checks + DLP scan |
| Art. 33 (DPIA / 영향평가) | dpia.md |

| ISMS-P §§ | Drive evidence |
|---|---|
| §2.1 (information security policy) | this compliance.md + policy/* |
| §2.3 (asset management) | catalog/*.yaml |
| §2.5 (human security) | 2-person rule + JIT |
| §2.7 (access control) | RLS + Cedar |
| §2.9 (operational security) | runbooks/* |
| §2.10 (communications security) | mesh mTLS + WAF |
| §2.11 (cryptography) | tenant-DEK + audit-chain Ed25519 + Argon2id |
| §2.12 (incident management) | incident-response.md |

| 전자문서법 §§ | Evidence |
|---|---|
| Art. 5 (integrity of electronic documents) | audit-chain Ed25519 + WORM tier |
| Art. 6 (storage of electronic documents) | retention + legal hold + WORM |
| Art. 7 (e-signature equivalence) | OIDC + JIT |

## Pack-overlay detail: pack-us (CCPA / CPRA + SEC 17a-4(f) + FINRA 4511)

| Control | Citation | Drive implementation |
|---|---|---|
| Right to know | CCPA §1798.100 | per-user export per FR-17 |
| Right to delete | CCPA §1798.105 | file-store deletion orchestrator (reconciled with WORM if elected) |
| Sale of PD opt-out | CCPA §1798.120 | no sale; documented in `legal/sub-processors.md` |
| WORM storage | SEC 17a-4(f) | object-store compliance-mode object-lock per ADR-DRIVE-0006 |
| Records retention | FINRA Rule 4511 | WORM tier with FINRA-compliant retention floor (6y default) |
| SOC 2 | TSC 2017+2022 | annual SOC 2 Type 2 |

## Pack-overlay detail: pack-us-healthcare (HIPAA + BAA + FDA 21 CFR Part 11)

| 45 CFR §§ | Drive evidence |
|---|---|
| §164.308(a)(1)(ii)(A) risk analysis | dpia.md + threat-model.md |
| §164.308(a)(3) workforce security | OpenBao JIT + 2-person rule |
| §164.308(a)(4) info access management | Cedar + RLS |
| §164.310(a) facility access | inherited from cloud-k8s |
| §164.312(a) access control | RLS + Cedar |
| §164.312(b) audit controls | audit-chain Ed25519 + retention ≥ 6y via WORM |
| §164.312(c) integrity | audit-chain Merkle |
| §164.312(d) person authentication | OIDC + MFA |
| §164.312(e) transmission security | mesh mTLS |
| §164.314(a) BAA | legal/baa-template.md |
| §164.316 documentation | WORM ≥ 6y |
| §164.502(a) Permitted Uses (TPO) | tenant DPA |
| §164.502(b) Minimum Necessary | cross-tenant share type-narrowing |
| §164.504(e) BAA terms | BAA template |
| §164.512 disclosures permitted | dpia.md |
| §164.514 de-identification | redaction in share projection |
| FDA 21 CFR Part 11 §11.10 | OPSWAT multi-engine scan + audit-chain ensures electronic-records integrity |

State-level:
- CCPA Cal. Civ. Code §1798.100 et seq.
- CMIA Cal. Civ. Code §56 et seq.: medical info disclosure restrictions.
- NY SHIELD Act: breach notification + reasonable security.

## Pack-overlay detail: pack-eu (GDPR + EDPB + NIS2 + eIDAS + EU AI Act)

- **EDPB Guidelines 4/2019 (Art. 25)**: by-design + by-default verified in §4 of dpia.md.
- **EDPB Guidelines 9/2022 (breach notification)**: 72h chain in incident-response.md.
- **NIS2 (2022/2555)**: 24h + 72h + 1mo timelines when threshold-engaged.
- **eIDAS 910/2014 Art. 26**: audit-chain Ed25519 seal satisfies AdES.
- **Schrems II + Arts. 44-46**: SCC-only transfers + TIA when non-adequate.
- **EU AI Act**: T1 OCR / auto-tag / smart-search = limited-risk; T2 HR-context REFUSED at Cedar pending ADR-DRIVE-XXXX.

## Pack-overlay detail: pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `cloud/cloud-iac/sovereign-cloud-overlays/<pack>/drive-compliance-overlay.md`. Aligned 1:1 with the calendar µservice overlay matrix for consistency across the µservice catalog.

## Audit cadence

| Audit | Cadence | Owner |
|---|---|---|
| SOC 2 Type 2 | annually | external SOC 2 firm |
| ISO 27001:2022 | every 3 years (initial) + annual surveillance | external ISO firm |
| GDPR / EDPB DPA examination | on-tenant request + breach trigger | council-privacy |
| HIPAA OCR audit | on-trigger | external HIPAA firm |
| SEC 17a-4 audit | annually (broker-dealer tenant) | external 17a-4 attestor |
| PIPC examination | on-trigger | council-privacy |
| ANPD (Brazil) | on-trigger | council-privacy |
| Pen-test (boundary tests per threat-model.md) | annually | external pen-test firm |
| LEAN-gate evidence review | per-PR | reviewer-agent |

## References

- ADR-0028 (Bominal), ADR-0117, ADR-0135, ADR-0140.
- ADR-DRIVE-0001 through ADR-DRIVE-0006.
- `threat-model.md`, `dpia.md`, `policy/*`, `multi-region.md`, `incident-response.md`, `legal/*`.
- SOC 2 TSC 2017 + 2022 PoF.
- ISO 27001:2022 Annex A.
- GDPR (Regulation 2016/679); EDPB Guidelines 4/2019 + 9/2022.
- KR PIPA + ISMS-P + 전자문서법; PIPC Notice 2020-7.
- HIPAA 45 CFR §164; FDA 21 CFR Part 11.
- SEC 17a-4(f); FINRA Rule 4511.
- APPI; PDPA; APP; DPDPA; LGPD; UAE PDPL; KSA PDPL.
- OWASP ASVS v4.0.3; OWASP API Top 10 (2023); CIS Kubernetes Benchmark v1.9.0.
- FIPS 140-3; NIST SP 800-57; NIST SP 800-154; NIST SSDF; SLSA L3.
- RFC 9106 (Argon2); RFC 9110 (HTTP); RFC 7233 (HTTP Range); RFC 4918 (WebDAV); tus.io 1.0; AWS S3 SigV4.

---



## §day-one-cert-readiness
This anchor is closed for `drive` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `drive` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +13 more.
- Example: `T0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pack-overlay-roster
This anchor is closed for `drive` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §minor-protection
This anchor is closed for `drive` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `drive` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `T0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §platform-owner-indirection
This anchor is closed for `drive` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `drive` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`, `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.drive.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §detection-substrate-binding
This anchor is closed for `drive` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `drive` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-suggest` touches those data classes.
- Signal sources: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`; +12 more.
- Example event class: `oya.drive.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §investigation-binding
This anchor is closed for `drive` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `drive` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.drive.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §insider-threat-controls
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `drive` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`, `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`; +5 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `drive.dlp_virus_scan` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §threat-intelligence-feeds
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `drive` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`; +12 more.
- Example: `T0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §key-rotation-cadence
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.drive` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/drive/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.
- Example: `T0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §crypto-agility-plan
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `drive` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`, `microservices/drive/iac/helm/Chart.yaml`, `microservices/drive/iac/helm/templates/deployment.yaml`, `microservices/drive/iac/helm/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §pentest-and-bounty-cadence
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `drive` is in annual full-scope pentest and every major `T0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`, `microservices/drive/iac/helm/Chart.yaml`, `microservices/drive/iac/helm/templates/deployment.yaml`, `microservices/drive/iac/helm/templates/hpa.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `drive` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §facility-controls
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `drive` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §supply-chain-risk
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `drive` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/drive/catalog/oya-drive-dlp-virus-scan-adapter-clamav.yaml`, `microservices/drive/catalog/oya-drive-dlp-virus-scan-adapter-opswat.yaml`, `microservices/drive/catalog/oya-drive-file-store-adapter-garage.yaml`, `microservices/drive/catalog/oya-drive-file-store-adapter-postgres.yaml`, `microservices/drive/catalog/oya-drive-file-store-adapter-s3.yaml`, `microservices/drive/catalog/oya-drive-file-store-adapter-seaweedfs.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §critical-path-edge-cases
This anchor is closed for `drive` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `drive` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

## §data-classification
This anchor is closed for `drive` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `drive`; owner `axis-drive`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `dlp-virus-scan`, `file-store`, `folder-hierarchy`, `immutability-tier`, `permissions`, `preview`; +4 more.
- Capability records cited: `microservices/drive/capabilities/T0-suggest.yaml`, `microservices/drive/capabilities/T1-assist.yaml`, `microservices/drive/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar/policy artifacts cited: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +6 more.
- Runbook/IaC evidence: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/drive/contracts/asyncapi/drive-events.yaml`, `microservices/drive/contracts/openapi/drive.yaml`, `microservices/drive/contracts/proto/drive.proto`.
- Cedar binding: `microservices/drive/policy/auditor-scope.cedar`, `microservices/drive/policy/ci-scope.cedar`, `microservices/drive/policy/data-residency.md`, `microservices/drive/policy/dual-context-isolation.md`, `microservices/drive/policy/public-read.cedar`, `microservices/drive/policy/tenant-scope.cedar`.
- State/event binding: `drive.dlp_virus_scan`, `drive.file_store`, `drive.folder_hierarchy`, `drive.immutability_tier`, `drive.permissions`, `drive.preview`; +2 more.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`, `microservices/drive/slos/download-first-byte-latency.openslo.yaml`, `microservices/drive/slos/file-list-latency.openslo.yaml`, `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`, `microservices/drive/slos/preview-render-latency.openslo.yaml`, `microservices/drive/slos/search-latency.openslo.yaml`; +3 more.
- Runbook binding: `microservices/drive/runbooks/dlp-quarantine-release.md`, `microservices/drive/runbooks/immutability-tier-violation.md`, `microservices/drive/runbooks/object-storage-degraded.md`, `microservices/drive/runbooks/share-link-takeover-incident.md`, `microservices/drive/runbooks/sync-conflict-resolution.md`, `microservices/drive/runbooks/upload-multipart-stuck.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `drive`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `drive`.
- `policy-engine` supplies the signed Cedar corpus while `drive` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `drive` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `drive`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `drive` applies the most restrictive policy and emits a degraded-mode audit event.
- Failure mode: Cedar fragment mismatch. Behavior: fail closed for mutating actions, serve read-only where safe, and roll back to the prior soaked fragment.
- Failure mode: audit pipeline backpressure. Behavior: buffer locally with bounded queue, stop high-risk mutations before evidence loss, and alert SRE.
- Failure mode: regional or cell outage. Behavior: follow `multi-region.md`; do not cross a pack residency boundary to preserve availability.
- Failure mode: key or credential compromise. Behavior: revoke OpenBao lease, rotate signing/provider keys, quarantine impacted events, and replay idempotent work.

### Verification hooks
- `oya-governance-adr-adherence-matrix` reads this anchor as the documented answer for the corresponding row.
- `oya-governance-cross-consistency` checks field names, pack ids, audit event taxonomy, SecretReference shape, and layer enum consistency.
- `oya-governance-doc-link-resolves` must resolve every artifact path cited here before this can promote to BLOCKER.
- `oya-governance-abuse-defence-ux-floor` and `oya-governance-critical-path-coverage` apply when the anchor touches abuse defence or edge cases.
- `oya verify`/pre-push evidence should include marker absence, ≥50-line section count, JSON manifest parse status, and contract/schema validation where available.

### Structural notes from this pass
- Structural issue check: manifest, policy, contract, SLO/dashboard, runbook, and IaC evidence surfaces are present for this content pass.

