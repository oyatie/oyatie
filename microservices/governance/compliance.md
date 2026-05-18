---
doc_class: ComplianceDocument
title: Compliance + Meta-Compliance Posture
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-compliance + ops-security + council-privacy + axis-foundry
deciders: ops-compliance, ops-security, council-privacy, council-architecture
related_adrs: [ADR-0028, ADR-0117, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/governance/dpia.md
  - microservices/governance/threat-model.md
  - microservices/governance/policy/data-residency.md
review_cadence: quarterly + on every regulatory change + annual external audit
doc_status: published
---

# Compliance + Meta-Compliance Posture: governance µservice

## Purpose

The governance µservice **enforces** compliance for every other oyatie µservice (via fitness lanes that verify SOC 2 / ISO 27001 / GDPR / pack-specific controls). This document declares the governance µservice's **own** compliance posture — its meta-compliance — i.e., the compliance posture of the µservice that enforces compliance.

If this document is wrong, oyatie's overall compliance posture is hollow.

## Frameworks tracked

| Framework | Scope | Authority |
|---|---|---|
| SOC 2 Type 2 | Trust Services Criteria (TSC 2017) — Security, Availability, Processing Integrity, Confidentiality, Privacy | AICPA |
| ISO/IEC 27001:2022 | ISMS scope: all of oyatie including governance µservice | ISO + accredited certification body |
| ISO/IEC 27017:2015 | Cloud-specific controls overlay | ISO |
| ISO/IEC 27018:2019 | Cloud PII protection overlay | ISO |
| SLSA v1.0 | Build L3 + Source L3 + Isolation L3 (target) | OpenSSF / Linux Foundation |
| NIST SSDF SP 800-218 rev 1 | Secure software development framework | NIST |
| OWASP ASVS 4.0.3 | Application security verification | OWASP |
| CIS Benchmarks | Kubernetes, Docker, Postgres, Linux | CIS |
| OpenSSF Best Practices Badge | OSS-style best practices (gold-tier target) | OpenSSF |
| GDPR | EU pack | EU + national DPAs |
| KR PIPA + KR-ISMS-P | pack-kr | KCC + PIPC |
| HIPAA + HITECH | pack-us-healthcare | OCR |
| APPI | pack-jp | PPC |
| PDPA-SG | pack-sg | PDPC |
| Privacy Act 1988 + APRA-CPS 234 | pack-au | OAIC + APRA |
| DPDPA 2023 | pack-in | Data Protection Board |
| LGPD | pack-br | ANPD |
| UAE PDPL | pack-ae | UAE Federal |
| KSA PDPL + SAMA-CSF | pack-ksa | SDAIA + SAMA |

## SOC 2 Type 2 control mapping

| TSC | Control | governance implementation |
|---|---|---|
| CC1.1 | Organisation demonstrates commitment to integrity + ethical values | docs/AGENTS.md + CLAUDE.md + ADR-0133 honesty principles |
| CC1.2 | Board oversight | council-architecture + council-privacy quarterly review |
| CC2.1 | Internal communication of objectives | PRD + IP + ADR chain; published to all µservices |
| CC3.1 | Risk assessment | threat-model.md (quarterly) + dpia.md (annual) + failure-modes.md (quarterly) |
| CC4.1 | Monitoring + evaluation | dashboards/*.json + Grafana OnCall + quarterly external audit |
| CC5.1 | Control activities | the ~50 fitness lanes themselves; this is the meta-control |
| CC6.1 | Logical access controls | Cedar tenant-scope + ci-scope + auditor-scope + public-read fragments; SPIFFE workload identity |
| CC6.2 | New users provisioning | OpenBao OIDC + JIT elevation; quarterly access review |
| CC6.3 | Separation of duties | per-runner SPIFFE; CODEOWNERS two-reviewer rule-pack edits; break-glass procedure |
| CC6.6 | Boundary protection | Envoy mTLS + per-runner network-policy + S3 bucket policy |
| CC6.7 | Data transmission | mTLS everywhere; HMAC-SHA256 on GitHub webhook; signed-commits |
| CC7.1 | Detection of anomalies | dashboards Grafana alerts; audit-chain seal verification; quarterly STRIDE+LINDDUN review |
| CC7.2 | Response to incidents | incident-response.md + on-call rotation |
| CC7.3 | Remediation + RCA | postmortem per failure-mode |
| CC7.4 | Evidence of operating effectiveness | every Finding signed + sealed; quarterly audit reports; 7y retention |
| CC7.5 | System monitoring | observability µservice + OnCall |
| CC8.1 | Change management | every change via PR + ~50-lane suite; ADR successor-IP for material changes |
| A1.1 | Availability commitments + meeting them | SLO at 99.95% per PRD; SLO authoring under slos/ |
| C1.1 | Confidential information identification | dpia.md §3 data inventory |
| C1.2 | Disposal | retention windows per data-residency.md; KMS key-destroy at retention end |
| PI1.1 | Processing integrity commitments | lane-execution.md invariants 1, 6, 11 (determinism + idempotency + replayability) |
| P1.1 | Privacy notice | dpia.md §3 right-of-information; PR-template footer; Application Shell notice |

## ISO 27001:2022 Annex A control mapping

| Control | Title | governance implementation |
|---|---|---|
| A.5.1 | Information security policies | this document + threat-model.md + dpia.md + ADR-0133 |
| A.5.7 | Threat intelligence | quarterly STRIDE+LINDDUN + supply-chain vendor recency lane |
| A.5.8 | Information security in project management | per-PR ~50-lane suite |
| A.5.10 | Information classification | data-class lane + Bominal ADR-0028 taxonomy |
| A.5.15 | Access control | Cedar fragments + SPIFFE + OpenBao JIT |
| A.5.16 | Identity management | OIDC + per-runner SPIFFE; identity rotation 30d |
| A.5.17 | Authentication information | OpenBao with 30-90d rotation |
| A.5.23 | Information security for cloud services | iac/ per-pack overlays + CIS Kubernetes Benchmark |
| A.5.26 | Response to information security incidents | incident-response.md + runbooks/ |
| A.5.30 | ICT readiness for business continuity | failure-modes.md RTO/RPO + multi-region.md DR plan |
| A.5.31 | Legal, statutory, regulatory + contractual requirements | this document + dpia.md + per-pack overlays |
| A.5.32 | Intellectual property rights | license-policy lane |
| A.5.33 | Protection of records | 7y retention + object-lock + audit-chain seal |
| A.5.34 | Privacy + protection of PII | dpia.md + tenant-scope.cedar |
| A.5.35 | Independent review of information security | annual external SOC 2 + ISO 27001 audit; auditor-scope.cedar |
| A.5.36 | Compliance with policies, rules + standards | the ~50 fitness lanes |
| A.5.37 | Documented operating procedures | runbooks/ (6 runbooks at M01; expanding) |
| A.8.2 | Privileged access rights | CODEOWNERS; break-glass procedure; quarterly access review |
| A.8.3 | Information access restriction | Postgres RLS; S3 IAM; Cedar fragments |
| A.8.5 | Secure authentication | SPIFFE + OIDC + signed-commits + HMAC webhook |
| A.8.7 | Protection against malware | container image scanning (Trivy via `oya-check-supply-chain`) |
| A.8.11 | Data masking | evidence sanitiser per `oya-check-data-class` lane |
| A.8.12 | Data leakage prevention | Cedar fragments + sanitiser + outbound allow-list |
| A.8.15 | Logging | every action emits audit record |
| A.8.16 | Monitoring activities | OnCall + dashboards + quarterly review |
| A.8.20 | Networks security | mTLS + per-runner NetworkPolicy + Envoy WAF |
| A.8.21 | Security of network services | mTLS + scoped PATs |
| A.8.22 | Segregation of networks | per-pack cluster isolation; ARC runner pool isolation |
| A.8.23 | Web filtering | outbound allow-list per runner |
| A.8.24 | Use of cryptography | Ed25519 + AES-GCM via KMS; key rotation 30-90d |
| A.8.25 | Secure development lifecycle | SLSA L3 + per-PR ~50-lane suite |
| A.8.26 | Application security requirements | OWASP ASVS v4 mapping |
| A.8.27 | Secure system architecture + engineering principles | clean-arch per `feedback_clean_architecture_requirements.md` |
| A.8.28 | Secure coding | rustfmt + clippy -D warnings + `cargo deny check` |
| A.8.30 | Outsourced development | not applicable; agentic dev team is in-house |
| A.8.31 | Separation of development, test + production environments | per-pack overlays; staging vs production refs per ADR-0130 |
| A.8.32 | Change management | per-PR + CODEOWNERS + ADR successor-IP |
| A.8.34 | Protection of information systems during audit testing | auditor-scope.cedar + JIT scope |

## SLSA v1.0 mapping

| Track | Level | governance posture |
|---|---|---|
| Build | L3 | ARC runners ephemeral; hardened image; build provenance signed via `oya-check-supply-chain` |
| Source | L3 | Signed commits enforced; branch-protection; non-repudiable history; audit-chain seal on every commit |
| Isolation | L3 | per-runner SPIFFE; per-runner NetworkPolicy; tmpfs-only filesystem |
| Provenance | L3 | SLSA-provenance manifest emitted with every release tag; verified at deploy |

## NIST SSDF SP 800-218 mapping

| Practice | governance implementation |
|---|---|
| PO.1 Define security requirements | this document + threat-model.md |
| PO.3 Implement supporting toolchains | the ~50 fitness lanes + iac/ |
| PO.4 Define + use criteria for software security checks | rule packs + `lane-execution.md` invariants |
| PS.1 Protect all forms of code | signed-commits + branch-protection + audit-chain seal |
| PS.2 Provide a mechanism for verifying software release integrity | SLSA L3 provenance + Ed25519 verify |
| PS.3 Archive + protect each software release | 7y retention + object-lock |
| PW.1 Design software to meet security requirements | secure-by-default policy fragments |
| PW.4 Reuse existing, well-secured software | vendor recency + supply-chain lane |
| PW.6 Configure compilation, build + assembly processes for security | rustfmt + clippy + cargo deny |
| PW.7 Review + analyze code for vulnerabilities | per-PR + Trivy + CodeQL (planned per IP successor-IP) |
| PW.8 Test executable code for vulnerabilities | per-PR + integration + e2e |
| RV.1 Identify + confirm vulnerabilities | Renovate + supply-chain lane |
| RV.2 Assess, prioritize + remediate vulnerabilities | severity classification + remediation IPs |

## ROPA (Record of Processing Activities; Art. 30 GDPR + KR PIPA Art. 30 + APRA-CPS 234)

| Activity | Categories of data subjects | Categories of personal data | Purpose | Recipients | Cross-border | Retention | Security |
|---|---|---|---|---|---|---|---|
| Per-PR fitness gating | PR authors (internal + tenant ops) | OIDC subject, email, name, commit metadata | Software-quality enforcement; SLSA L3 non-repudiation | internal axis-foundry + ops-security; external auditors during JIT window | per `data-residency.md` | 7y AUDIT view; 2y non-AUDIT | TLS + at-rest encryption + Ed25519 seal |
| Finding emission | PR authors | as above + Finding hash | Audit-replayability; SOC 2 CC7.4 | internal + auditors | per `data-residency.md` | 7y | as above |
| Evidence blob storage | PR authors | as above + lane output transcripts (occasionally containing user-attributable data) | Audit-replayability | internal + auditors | per `data-residency.md` | 7y; object-lock | SSE-KMS + sanitiser |
| Audit-chain sealing | PR authors | Finding hashes + identity | Cryptographic audit trail | internal | per `data-residency.md` | indefinite (Merkle history) | Ed25519 + HSM where available |
| External-auditor read | Auditors (lawful third-party) | per-audit-window scope | Regulatory audit | auditor only | per-audit DPA | per-audit window only | OIDC JIT + auditor-scope.cedar |

## Cross-Border Transfer Mechanisms

See `policy/data-residency.md` for full per-pack transfer matrix. Per-pack DPAs / SCCs / adequacy decisions logged below per Art. 30(1)(e):

| Transfer | Mechanism | Signed | Reviewer |
|---|---|---|---|
| pack-eu → pack-us (auditor) | EU-US DPF | required before first pack-eu auditor session; tracked at `microservices/governance/dpa-pack-eu-to-us-dpf.md` (signature recorded inline on first auditor onboarding) | council-privacy |
| pack-eu → pack-au/sg/in/br/ae/ksa | SCCs 2021/914 + TIA | required before first pack-eu→pack-X auditor session; per-pack SCC instance at `microservices/governance/scc-pack-eu-to-<pack>.md` + TIA recorded inline | council-privacy |
| pack-us-healthcare → other | refused absent BAA chain | n/a | ops-compliance |

## Annual external audit posture

| Audit | Cadence | Auditor | Last completed | Next planned |
|---|---|---|---|---|
| SOC 2 Type 2 | annual | A-LIGN (selected per ops-compliance vendor matrix; alternates: Schellman, Insight Assurance) | n/a (M01 baseline) | 2027-Q2 |
| ISO 27001 | annual surveillance; 3y recertification | BSI Group (selected per ops-compliance vendor matrix; alternates: SGS, DNV) | n/a | 2027-Q2 |
| KR-ISMS-P | annual | KCC (Korea Communications Commission) | n/a | 2027-Q2 |
| HIPAA-SOC 2 (when pack-us-healthcare onboarded) | per BAA | A-LIGN (HIPAA-extended SOC 2 module; same vendor as primary SOC 2 audit for chain-of-evidence simplicity) | n/a | gated by pack-us-healthcare onboarding; planned 30 days after first BAA signature |

## Quarterly self-audit

| Quarter | Axis | Findings | Remediation IPs |
|---|---|---|---|
| 2026-Q2 | all 6 axes (baseline) | findings table populated from `cargo run -p oya-dev-cli -- gate scan --quarterly` output on first quarter close; baseline-quarter row updates to count + breakdown post-run | filed under `microservices/governance/IP-M01-AUDIT-*-NNN.md` |

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance --microservice governance` — exit 0.
- Quarterly compliance review by ops-compliance + council-privacy.
- Annual external audit per the table above.

## References

- `microservices/governance/dpia.md` (Right of Access flows).
- `microservices/governance/threat-model.md` (control coverage detail).
- `microservices/governance/policy/data-residency.md` (per-pack overlays).
- SOC 2 Trust Services Criteria 2017 — `aicpa-cima.com`.
- ISO 27001:2022 — `iso.org/standard/27001`.
- SLSA v1.0 — `slsa.dev/spec/v1.0/levels`.
- NIST SSDF SP 800-218 rev 1 — `csrc.nist.gov/publications/detail/sp/800-218/final`.
- OWASP ASVS v4.0.3 — `owasp.org/www-project-application-security-verification-standard/`.
- `microservices/observability/compliance.md` (shape reference).
