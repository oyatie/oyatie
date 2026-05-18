---
doc_class: ComplianceMap
template_id: TPL-COMPLIANCE-MAP
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-security + dpo-office
doc_status: published
---

# Compliance map — slides µservice

Cross-walk: each standard / regulation / certification → control → slides artifact + verification.

## Standards

### ISO 27001:2022

| Annex A | Slides control | Artifact |
|---|---|---|
| A.5.7 Threat intelligence | RustSec + GitHub Security Advisories subscription | `threat-model.md` §"Supply-chain threats" |
| A.5.10 Information classification | Per-field `#[data_class]` annotation | `oya-check-data-class` LEAN lane |
| A.5.15 Access control | Cedar v4.2 LTS default-deny; per-slide ACL | `policy/*.cedar` + ADR-SLIDES-0007 |
| A.5.23 Cloud-services | Multi-cloud + multi-region per pack | `multi-region.md` |
| A.5.24 Incident management | Sev-1 alarms + runbooks | `runbooks/` |
| A.5.30 ICT readiness for business continuity | DR drill + per-pack RTO/RPO | `multi-region.md` + `incident-response.md` |
| A.5.34 Privacy and protection of PII | DPIA + per-pack overlay | `dpia.md` + `policy/data-residency.md` |
| A.6.3 Information security awareness | Standards docs + training | `docs/standards/` |
| A.8.4 Access to source code | Branch protection + code review | `.github/branch-protection.yaml` |
| A.8.7 Protection against malware | ClamAV + OPSWAT dual-scan | `threat-model.md` T-T-03/04 |
| A.8.8 Management of technical vulnerabilities | cargo-deny + SBOM | `cargo deny check` |
| A.8.9 Configuration management | IaC + Helm + Kustomize | `iac/` |
| A.8.12 Data leakage prevention | Strict CSP + per-tenant isolation | `threat-model.md` §"Information Disclosure" |
| A.8.16 Monitoring activities | OpenSLO + Grafana dashboards + audit-chain | `slos/` + `dashboards/` |
| A.8.23 Web filtering | CSP + per-tenant CDN partition | helm `studio-cdn` values |
| A.8.24 Cryptography | TLS 1.3 + SSE-KMS + Ed25519 audit-chain | `dpia.md` §2.4 |
| A.8.25 Secure development life cycle | Foundry pipeline + admission gate | CI lanes |
| A.8.28 Secure coding | Clippy + per-µservice lints | `cargo clippy -- -D warnings` |

### SOC 2 Type 2

| Trust Service Criteria | Control | Artifact |
|---|---|---|
| Security CC1 Control environment | Workspace governance + ADR registry | `docs/MASTERPLAN.md` |
| Security CC6 Logical access | OIDC + Cedar + per-slide ACL | `policy/` |
| Security CC7 System operations | Runbooks + Sev-1 alarms + SLO burn-rate | `runbooks/` + `slos/` |
| Security CC8 Change management | ChangeSet IP + admission gate | `IP-*.md` |
| Security CC9 Risk mitigation | Threat-model + DPIA | `threat-model.md` + `dpia.md` |
| Availability A1 Availability | OpenSLO + multi-region DR | `slos/` + `multi-region.md` |
| Processing integrity PI1 Quality + accuracy | Loro CRDT no-silent-loss + audit-chain | AC-06 |
| Confidentiality C1 Confidentiality | RLS + per-tenant isolation + per-slide ACL | `threat-model.md` §"Info Disclosure" |
| Privacy P1-P8 (when applicable) | DPIA + per-pack consent + data subject rights | `dpia.md` |

### NIST SSDF v1.1

| Practice | Slides implementation | Artifact |
|---|---|---|
| PO.1 Define security requirements | This compliance map + PRD §Security | `PRD.md` + `compliance.md` |
| PO.5 Implement and maintain secure environments | IaC + per-pack isolation | `iac/` |
| PS.1 Protect all forms of code from unauthorized access + tampering | Branch protection + signed commits | `.github/branch-protection.yaml` |
| PS.2 Provide a mechanism for verifying software release integrity | SBOM + cosign signatures + SRI | release pipeline |
| PS.3 Archive + protect each software release | Per-pack release pointers + retention | `release/slides/{staging,production}` pattern |
| PW.4 Reuse existing well-secured software | LTS-pinned Loro / Leptos / Pandoc / WeasyPrint / ffmpeg / Chromium-headless / LiveKit / Postgres / Valkey | `Cargo.lock` + Helm `values.yaml` LTS pins |
| PW.5 Create source code by adhering to secure coding practices | Clippy + ASVS + per-lint gates | CI |
| PW.6 Configure compilation + build processes | cargo-leptos + WASM SRI | IP-014 |
| PW.7 Review + analyze human-readable code | Multi-spectrum review per axis-skills | `docs/standards/code-review.md` |
| PW.8 Test executable code | cargo nextest + e2e + load + proptest | `tests/` |
| RV.1 Identify + confirm vulnerabilities on an ongoing basis | RustSec + GitHub Security Advisories | supply-chain monitor |
| RV.2 Assess + prioritize + remediate vulnerabilities | RSL — Repo Security Log | `evidence/security/` |

### SLSA L3

- Builds run in hermetic, parameterless, isolated runner per cloud-iac substrate.
- Provenance generated per artifact (in-toto attestation + cosign signature).
- Build configuration is signed + verifiable.
- Source attested to GitHub commit SHA.

### CIS Kubernetes Benchmark v1.10

- runAsNonRoot + readOnlyRootFilesystem + drop ALL capabilities per `iac/helm/*/templates/deployment.yaml`.
- PodSecurityPolicy / Pod Security Standards `restricted` profile.
- NetworkPolicy egress allowlist (no default-allow).
- Per-pack namespace isolation.

### OWASP ASVS v4

| Level | Slides target | Verification |
|---|---|---|
| V1 Architecture | L3 (high-assurance) | per-µservice ADRs + per-bc kernel/port |
| V4 Access Control | L3 | Cedar v4 default-deny + per-slide ACL |
| V5 Validation, Sanitization, Encoding | L3 | sanitization at embed-bridge boundary; CSP strict |
| V8 Data Protection | L3 | encryption + per-pack residency |
| V9 Communications | L3 | TLS 1.3 + WSS + cert pinning |
| V11 Business Logic | L2 | Cedar policy preview before save (AC-15) |
| V13 API + Web Service | L3 | OpenAPI 3.2.0 + AsyncAPI 3.1.0; OIDC-bound at every entry |
| V14 Configuration | L3 | IaC + Helm + Kustomize; secrets via OpenBao only |

### ISO 32000-1 (PDF 1.7) + PDF/A-1b + PDF/A-2u

- PDF export targets PDF/A-1b (archival baseline) + PDF/A-2u (Unicode-mapped archival).
- WeasyPrint or Chromium-headless emits PDF/A-conformant output.
- PAdES signatures for legally-signed PDFs (per eIDAS where applicable).

### ECMA-376 (OOXML — PresentationML)

- PPTX import via Pandoc bridge (best-effort).
- PPTX export via bespoke OOXML serializer over the round-trippable subset (ADR-SLIDES-0003).
- 95% of round-trippable subset preserved byte-for-byte on import → emit → reimport.

### ISO/IEC 26300 (ODF — OpenDocument Presentation 1.3)

- ODP import + export support; round-trip subset validated.

### WCAG 2.2 AA

| SC | Slides implementation |
|---|---|
| 1.1.1 Non-text content | Alt-text suggestion (T1) + manual override |
| 1.4.3 Contrast (Minimum) | Color-contrast validator in accessibility BC |
| 1.4.11 Non-text Contrast | Color-contrast validator |
| 1.4.12 Text Spacing | Theme typography respects user-set spacing overrides |
| 2.1.1 Keyboard | All canvas interactions keyboard-reachable |
| 2.3.3 Animation from Interactions | `prefers-reduced-motion` honored (ADR-SLIDES-0004) |
| 2.4.7 Focus Visible | Focus-ring on every canvas primitive |
| 3.3.7 Redundant Entry | Authoring forms remember prior entries within session |
| 4.1.2 Name, Role, Value | ARIA role per canvas primitive (svg `role` + `aria-*`) |
| 4.1.3 Status Messages | Save/conflict/AI-status via ARIA live regions |

### EU AI Act (Regulation (EU) 2024/1689)

- Art. 6 + Annex III: T2 ai-content-generation evaluated per-invocation for high-risk Annex III contexts (employment, credit, legal, medical). High-risk → refused by default; per-pack override.
- Art. 13 transparency: T2-generated decks carry an indelible provenance watermark.
- Art. 14 human oversight: T2 outputs require explicit human accept before save (ai-content-generation BC enforces).
- Art. 16: foundry-runtime is the AI risk-classification authority; slides forwards verdict + stamps audit row.
- Art. 50 transparency to deployer + affected persons: per-pack notice text in editor UI.

### eIDAS (Regulation (EU) 910/2014)

- Where applicable, exported PDFs (PDF/A-2u) carry PAdES-baseline signatures for legal force.
- Signing keys held in pack-pinned HSM (OpenBao Transit + KMS); never embedded in code.

## Regulations (per pack)

### EU pack — GDPR

| Article | Slides | Artifact |
|---|---|---|
| Art. 5 (principles) | data minimization (broadcast attendee aggregate-default; AI hash + 90d retention) | DPIA §2.2 |
| Art. 6 (lawfulness) | (b) contract, (c) legal obligation, (f) legitimate interest | DPIA §2.1 |
| Art. 9 (special category) | EU pack overlay enforces explicit consent flag | DPIA §3.2 |
| Art. 13/14 (information) | tenant T&C + in-editor notice for AI + broadcast | UI notice strings |
| Art. 15 (access) | export via SDK | sdk-plan.md |
| Art. 16 (rectification) | version-history restore + Cedar evaluation | AC-08 |
| Art. 17 (erasure) | cryptographic delete on retention expiry | retention scheduler |
| Art. 20 (portability) | PPTX/ODP/PDF/MP4 export | AC-02 + IP-011 |
| Art. 22 (automated decisions) | T2 ai-content-generation is decision-support only; explicit human accept required | ADR-SLIDES-0006 |
| Art. 25 (data protection by design) | Cedar default-deny + per-slide ACL + per-pack residency | architecture |
| Art. 28 (processor) | tenant-as-controller; slides-as-processor; DPA template | sdk-plan.md §"Tenant agreement" |
| Art. 30 (records) | audit-chain Ed25519 seal end-to-end | audit-chain |
| Art. 32 (security) | TLS 1.3 + SSE-KMS + per-tenant isolation | DPIA §2.4 |
| Art. 33 (breach notification) | Sev-1 alarm + 72h notification SLA | incident-response.md |
| Art. 35 (DPIA) | this DPIA | dpia.md |
| Art. 44 (transfers) | per-pack residency; SCC + adequacy where applicable | multi-region.md |

### KR pack — PIPA + 전자문서법

- PIPA Art. 17 (provision to third parties), Art. 18 (out-of-scope use), Art. 28 (technical/managerial protective measures), Art. 34 (breach notification).
- 전자문서법 (Framework Act on Electronic Documents) §§4-7 — electronic record retention (presentations stored as PDF/A-1b for legal record per tenant choice).

### US pack — CCPA/CPRA + state laws

- CCPA right to know/delete/opt-out — supported via SDK.
- CPRA sensitive personal information — handled per-pack consent.

### US-healthcare pack — HIPAA + HITECH

| §§ | Slides | Artifact |
|---|---|---|
| §164.308 administrative safeguards | per-tenant access control + audit + workforce-training reference | DPIA §3.2 + policy/ |
| §164.310 physical safeguards | cloud-iac data center attestations | inherited |
| §164.312 technical safeguards | encryption + access control + audit + integrity + transmission security | DPIA §2.4 + threat-model.md |
| §164.314 organizational requirements | BAA template | sdk-plan.md |
| §164.316 policies + procedures | this compliance map + DPIA | docs |
| §164.404 breach notification | 60d Notice + HHS reporting | incident-response.md |
| §164.530(c) safeguards | per-pack PHI redaction in AI flows | DPIA §3.2 |
| §164.530(j) retention | 6y minimum | retention scheduler |

### JP pack — APPI

- Personal Information Protection Commission notification.
- Cross-border transfer consent + transparency.

### SG pack — PDPA Singapore

- Consent + purpose limitation + protection + notification.

### AU pack — Privacy Act 1988 (APPs)

- APP 1 (open + transparent management), APP 5 (notification), APP 8 (cross-border disclosure), APP 11 (security).

### IN pack — DPDPA 2023

- Data Principal rights; consent; cross-border transfers per Schedule 1.

### BR pack — LGPD

- ANPD requirements; data subject rights.

### AE pack — UAE PDPL

- Controller-processor agreement; cross-border data transfer.

### KSA pack — KSA PDPL

- SDAIA registration; cross-border data transfer per PDPL Art. 29.

## Certifications + audits

- **SOC 2 Type 2**: planned subsequent-to-GA-tier-promotion; quarterly internal control test cycle.
- **ISO 27001:2022**: planned subsequent-to-GA-tier-promotion.
- **HIPAA**: US-healthcare pack — annual third-party assessment.
- **PCI-DSS**: not in scope for slides directly (no cardholder data processed in slides; foundry/payment-handling out-of-scope).
- **FedRAMP**: out-of-scope at first launch; revisit if US Federal customers materialize.

## Evidence

- All evidence stored under `evidence/` (gitignored beyond canonical evidence pointer files); per ADR-0123 audit-grade evidence retention.

## References

- ISO 27001:2022 Annex A.
- SOC 2 Trust Services Criteria.
- NIST SSDF v1.1.
- SLSA Level 3.
- CIS Kubernetes Benchmark v1.10.
- OWASP ASVS v4.
- ISO 32000-1 (PDF 1.7); ISO 19005-1 (PDF/A-1); ISO 19005-2 (PDF/A-2).
- ECMA-376 (OOXML).
- ISO/IEC 26300 (ODF).
- W3C WCAG 2.2.
- W3C Subresource Integrity.
- W3C Media Queries Level 5 (`prefers-reduced-motion`).
- EU GDPR; EU AI Act; eIDAS Regulation.
- HIPAA (45 CFR §§160 + 164); HITECH Act.
- KR PIPA + 전자문서법 + 전자거래기본법.
- APPI; PDPA SG + AU; DPDPA 2023; LGPD; UAE PDPL; KSA PDPL.
