---
doc_class: CompliancePackOverlay
microservice: contract-lifecycle-management
pack_id: iso-27001
authoritative_source: ISO/IEC 27001:2022 + ISO/IEC 27002:2022
related_adrs: [ADR-0251, ADR-0244, ADR-0263]
date: 2026-05-21
---

# ISO-27001 Pack Overlay — CLM

ISO/IEC 27001:2022 is the international standard for an Information Security Management System (ISMS). The 2022 revision restructured Annex A controls from 114 to 93, grouped into 4 themes: Organizational (37), People (8), Physical (14), Technological (34).

## Active triggers

The `iso-27001` pack is **mandatory** for any paid tenant with European or international scope, and **recommended** for all tenants.

## Annex A control mapping (high-level)

### Organizational controls

- **A.5.1 Information security policies** — `compliance.md` + `threat-model.md`.
- **A.5.7 Threat intelligence** — `threat-model.md` + cross-emit to `detection` substrate.
- **A.5.9 Inventory of information and other associated assets** — manifest.json declares all artefacts.
- **A.5.10 Acceptable use of information and other associated assets** — Cedar policies in `policy/*.cedar`.
- **A.5.11 Return of assets** — termination workflow exports + returns contract artefacts.
- **A.5.15 Access control** — Cedar default-deny per ADR-0243.
- **A.5.23 Information security for use of cloud services** — deployment_context model + cell isolation.
- **A.5.24 Information security incident management planning and preparation** — `incident-response.md`.
- **A.5.30 ICT readiness for business continuity** — `multi-region.md` + `failure-modes.md`.

### People controls

- **A.6.1 Screening** — identity µservice principal verification.
- **A.6.3 Information security awareness, education and training** — `onboarding/legal-ops-first-week.md`.
- **A.6.6 Confidentiality or non-disclosure agreements** — NDA contract type in `taxonomies/contract-type-taxonomy.md`.

### Physical controls

- **A.7.4 Physical security monitoring** — delegated to deployment_context's physical layer (AWS/OCI for guest; tenant for on-prem/colo).
- **A.7.10 Storage media** — encryption-at-rest.
- **A.7.14 Secure disposal or re-use of equipment** — cryptographic erasure on key rotation.

### Technological controls

- **A.8.2 Privileged access rights** — Cedar + dual-control for sovereign-pack actions.
- **A.8.3 Information access restriction** — tenant-scoped projection (ADR-0244).
- **A.8.5 Secure authentication** — WebAuthn FIDO2 / mTLS / SPIFFE.
- **A.8.7 Protection against malware** — `threat-model.md` malware section.
- **A.8.8 Management of technical vulnerabilities** — CVE feed + dependency scanning.
- **A.8.9 Configuration management** — OpenTofu modules with state versioning.
- **A.8.10 Information deletion** — Cedar gate on delete; retention enforcement.
- **A.8.11 Data masking** — tenant-scoped PII redaction.
- **A.8.12 Data leakage prevention** — egress controls + audit-chain.
- **A.8.13 Information backup** — multi-region replication.
- **A.8.14 Redundancy of information processing facilities** — cell topology per ADR-0248.
- **A.8.15 Logging** — ADR-0263 observability emission contract.
- **A.8.16 Monitoring activities** — dashboards/ + SLOs.
- **A.8.17 Clock synchronization** — HLC default per ADR-0252.
- **A.8.18 Use of privileged utility programs** — restricted to break-glass with audit.
- **A.8.19 Installation of software on operational systems** — Foundry pipeline.
- **A.8.20 Networks security** — TLS 1.3 + mTLS + ECH + PQC hybrid (ADR-0253).
- **A.8.21 Security of network services** — service mesh + Cedar.
- **A.8.22 Segregation of networks** — tenant-scoped network policies (`iac/<context>/network-policy.yaml`).
- **A.8.23 Web filtering** — egress controls.
- **A.8.24 Use of cryptography** — FIPS-mode where applicable; PQC hybrid in transit.
- **A.8.25 Secure development life cycle** — Foundry pipeline + Cedar pre-merge.
- **A.8.26 Application security requirements** — `threat-model.md`.
- **A.8.27 Secure system architecture and engineering principles** — ADR-0105 + ADR-0131 + ADR-0145.
- **A.8.28 Secure coding** — Rust-strict + clippy unwrap/expect/panic deny.
- **A.8.32 Change management** — Foundry pipeline.
- **A.8.33 Test information** — `tests/`.
- **A.8.34 Protection of information systems during audit testing** — audit-scoped Cedar profile.

## Statement of Applicability (SoA)

CLM ships an SoA document at `iso-27001/statement-of-applicability.md` listing each of the 93 Annex A controls with: applicable (yes/no), justification, and reference to the implementing artefact.

## Audit evidence package

ISO 27001 audit evidence is a superset of the SOC-2 bundle plus the SoA.

## Cedar gate fragment

Similar to SOC-2; the same set of Cedar gates satisfies both.

## Composition with other packs

- `iso-27001` + `soc-2`: significant overlap; SoA bridges to AICPA TSC.
- `iso-27001` + `gdpr`: Annex A controls satisfy GDPR Article 32 security-of-processing.
- `iso-27001` + `hipaa-baa`: Annex A controls satisfy HIPAA Security Rule administrative + technical safeguards.

## Evidence on activation

- `oya.contract.lifecycle.management.pack.iso-27001.activated` audit event with the tenant's declared scope.
- Cedar policy compilation.
- SoA snapshot pinned at activation.

## Standards references

- ISO/IEC 27001:2022 — Information security, cybersecurity and privacy protection — Information security management systems — Requirements.
- ISO/IEC 27002:2022 — Information security, cybersecurity and privacy protection — Information security controls.
- ISO/IEC 27017:2015 — Code of practice for information security controls based on ISO/IEC 27002 for cloud services.
- ISO/IEC 27018:2019 — Code of practice for protection of personally identifiable information (PII) in public clouds acting as PII processors.
