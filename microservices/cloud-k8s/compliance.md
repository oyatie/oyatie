---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-cloud, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0121, ADR-0123, ADR-0130, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/cloud-k8s/threat-model.md
  - microservices/cloud-k8s/dpia.md
  - microservices/cloud-k8s/policy/cluster-isolation.md
  - microservices/cloud-k8s/policy/data-residency.md
  - microservices/cloud-k8s/incident-response.md
review_cadence: annually + on every enforced-framework version update
doc_status: published
---

# Compliance Control-to-Framework Mapping (cloud-k8s µservice)

## Purpose

Canonical control-to-framework mapping for cloud-k8s. Tells an external auditor (SOC 2 Type 2 / ISO 27001:2022 / GDPR DPA / KR PIPC / HIPAA OCR / KR-CSAP / etc.) exactly which control implementation satisfies which framework clause, with pointers to evidence. Continuous-compliance-evidence emission keeps this matrix machine-verifiable; the `oya-governance-compliance-evidence-recency` lane enforces freshness.

## Enforced Frameworks

### SOC 2 Type 2 (2017 TSC + 2022 Points of Focus)

| TSC | Control objective | Implementation | Evidence |
|---|---|---|---|
| CC1.1 | Integrity + ethical values | CODEOWNERS + signed-commit | `docs/standards/code-review.md` + branch-protection.yaml |
| CC1.2 | Board oversight | Council-architecture quarterly review | `docs/teams/council-architecture.md` |
| CC1.3 | Organizational structure | RACI matrix per µservice | `microservices/cloud-k8s/CODEOWNERS` |
| CC1.4 | Commitment to competence | Onboarding + training | `docs/standards/onboarding.md` |
| CC1.5 | Accountability | Per-µservice SLO + on-call | PRD §Performance + `incident-response.md` |
| CC2.1 | Communication | Status page + tenant comms | `runbooks/*` |
| CC2.2 | Internal communication | Slack + incident channels | `incident-response.md` |
| CC2.3 | External party communication | DPA + BAA + tenant onboarding | `legal/dpa-template.md` |
| CC3.1 | Risk identification | Annual threat-model + DPIA | `threat-model.md` + `dpia.md` |
| CC3.2 | Risk to entity objectives | Multi-spectrum review per IP | `evidence/multispectrum/` |
| CC3.3 | Risk of fraud | audit-chain Ed25519 seals + 2-person rule | `policy/cluster-isolation.md` |
| CC3.4 | Significant change risk | PR review + LEAN | branch-protection.yaml |
| CC4.1 | Internal monitoring | LEAN CI lanes + SLOs | `/specs/quality/lanes.yaml` |
| CC4.2 | Deficiency communication | audit-chain emission per state transition | ADR-0028 |
| CC5.1 | Control activities | 50+ LEAN lanes (governance µservice) | governance µservice |
| CC5.2 | Technology controls | Cedar + multi-tenancy + signed commits + signed images | `policy/*.cedar` |
| CC5.3 | Policy + procedure deployment | Per-µservice runbooks + standards | `runbooks/*` |
| CC6.1 | Logical + physical access | kubernetes-api-proxy + OIDC + MFA + Cedar + JIT via OpenBao | `policy/*.cedar` |
| CC6.2 | Auth + authz | OIDC + per-component SPIFFE | `policy/cluster-isolation.md` |
| CC6.3 | Adds/removes access | OpenBao access lifecycle + audit | OpenBao audit log |
| CC6.6 | Logical access control | Cilium NetworkPolicy + Istio AuthorizationPolicy + RBAC | `policy/cluster-isolation.md` |
| CC6.7 | Information transmission + disposal | Istio mTLS in transit + KMS at rest + DSR cascade | `policy/data-residency.md` |
| CC6.8 | Vulnerability management | Trivy + Grype + cargo deny + weekly CVE | `/specs/supply-chain.json` |
| CC7.1 | System operations | HA control plane + cluster autoscaler + HPA | `capacity-model.md` |
| CC7.2 | Monitoring system inputs | Self-observability + OnCall | `failure-modes.md` |
| CC7.3 | Anomaly evaluation | Cilium flow anomaly + audit-chain anomaly detection | observability µservice |
| CC7.4 | Incident response | Severity-classified + runbooks | `incident-response.md` |
| CC8.1 | Change management | PR + LEAN + branch protection | this changeset |
| CC9.1 | Risk mitigation | Multi-AZ + DR pair + automated rollback | `multi-region.md` |
| CC9.2 | Vendor risk | Sub-processor list + per-vendor DPA | `legal/sub-processors.md` |

**Privacy Criteria (P1–P8):**

| P# | Criterion | Implementation |
|---|---|---|
| P1 | Notice | Tenant DPA + onboarding notice |
| P2 | Choice + consent | OpenBao tenant-resolver |
| P3 | Collection | Workload-layer redactor + data_class |
| P4 | Use, retention, disposal | Retention matrix; DSR cascade |
| P5 | Access | Tenant operators read own data |
| P6 | Disclosure | Sub-processor list + transfer register |
| P7 | Quality | audit-chain integrity |
| P8 | Monitoring + enforcement | Continuous-compliance-evidence lane |

### ISO 27001:2022 (Annex A)

| Annex A | Control | Implementation | Evidence |
|---|---|---|---|
| A.5.7 | Threat intelligence | Annual threat-model | `threat-model.md` |
| A.5.10 | Acceptable use | Cedar policy | `policy/*.cedar` |
| A.5.14 | Information transfer | mTLS + cross-pack forbidden | `policy/data-residency.md` |
| A.5.15 | Access control | OIDC + Cedar + RBAC + NetworkPolicy | `policy/cluster-isolation.md` |
| A.5.17 | Authentication information | OpenBao rotation (30d/90d/1y) | OpenBao audit |
| A.5.18 | Access rights | Terraform-managed RBAC | `iac/terraform/` |
| A.5.23 | Cloud services security | OCI HIPAA-eligible regions + per-pack pin | `policy/data-residency.md` |
| A.5.24 | Incident planning | Playbook | `incident-response.md` |
| A.5.25 | Event assessment | Severity classification | `incident-response.md` |
| A.5.26 | Incident response | Runbooks | `runbooks/*` |
| A.5.27 | Learning from incidents | Postmortem template + ADR successor-IP | `runbooks/postmortem-template.md` |
| A.5.28 | Evidence collection | audit-chain Ed25519 | ADR-0028 |
| A.5.30 | ICT readiness | Multi-AZ + DR | `multi-region.md` |
| A.5.31 | Legal + statutory | This document | this file |
| A.5.32 | IP rights | License policy CI | `oya-check-license-policy` |
| A.5.33 | Records protection | etcd encryption + audit-chain immutability | `policy/data-residency.md` |
| A.5.34 | PII privacy | DPIA + DSR + Cedar | `dpia.md` |
| A.8.2 | Privileged access | JIT via OpenBao + 2-person rule | OpenBao audit |
| A.8.3 | Information access restriction | Multi-tenancy + Cedar | `policy/cluster-isolation.md` |
| A.8.4 | Source code access | CODEOWNERS + branch-protection | branch-protection.yaml |
| A.8.5 | Secure authentication | OIDC + MFA + SPIFFE | `policy/cluster-isolation.md` |
| A.8.6 | Capacity management | HPA + autoscaler + capacity-model | `capacity-model.md` |
| A.8.7 | Malware protection | Trivy + Grype + signed images (Cosign) | `.github/workflows/cosign.yml` |
| A.8.8 | Vulnerability management | Weekly CVE scan + admission refusal | governance µservice |
| A.8.11 | Data masking | Workload-layer PII redactor | observability µservice |
| A.8.12 | Data leakage prevention | Cross-tenant query refusal + Cilium NetworkPolicy | `policy/cluster-isolation.md` |
| A.8.13 | Backup | etcd snapshot 5-min cadence | `runbooks/etcd-quorum-recovery.md` |
| A.8.14 | Redundancy | HA control plane + RF ≥ 3 | `multi-region.md` |
| A.8.15 | Logging | audit-chain + Loki structured | ADR-0028 |
| A.8.16 | Monitoring | Self-observability + OnCall | `failure-modes.md` |
| A.8.20 | Networks security | Cilium NetworkPolicy + Istio mTLS | `policy/cluster-isolation.md` |
| A.8.21 | Network services security | Same | Same |
| A.8.22 | Network segregation | Per-namespace + per-pack cluster boundary | `policy/cluster-isolation.md` |
| A.8.23 | Web filtering | Envoy WAF + OWASP CRS | ingress-controller |
| A.8.24 | Cryptography | TLS 1.3 + Ed25519 + AES-256-GCM | `policy/encryption.md` |
| A.8.25 | Secure development | LEAN lanes + PR review | `docs/standards/*` |
| A.8.26 | Application security | OpenAPI + Cedar + LEAN | `contracts/openapi/*` |
| A.8.27 | Secure architecture | Clean architecture (ADR-0056 + ADR-0105) | ADRs |
| A.8.28 | Secure coding | Cedar fuzz + cargo clippy + cargo deny | LEAN |
| A.8.30 | Outsourced development | n/a (in-house) | – |
| A.8.31 | Supply chain | Cosign + Trivy + Grype + SBOM | `.github/workflows/` |
| A.8.32 | Change management | PR review + LEAN | branch-protection.yaml |
| A.8.33 | Test information | Synthetic data only in non-prod | `docs/standards/testing.md` |
| A.8.34 | Audit testing protection | Auditor JIT + scoped reads | `policy/auditor-scope.cedar` |

### GDPR

| Art. | Requirement | Implementation | Evidence |
|---|---|---|---|
| 5(1)(a) | Lawful, fair, transparent | Tenant notice + joint-controllership | DPA |
| 5(1)(b) | Purpose limitation | DPIA §2.4 | DPIA |
| 5(1)(c) | Data minimisation | Workload-layer redactor | OTel config |
| 5(1)(d) | Accuracy | audit-chain integrity | LEAN |
| 5(1)(e) | Storage limitation | Retention matrix | `policy/data-residency.md` |
| 5(1)(f) | Integrity + confidentiality | etcd at-rest + Istio mTLS | `policy/cluster-isolation.md` |
| 5(2) | Accountability | This + DPIA + ROPA | `legal/ropa.md` |
| 6 | Lawful basis | Art. 6(1)(b) + 6(1)(c) + 6(1)(f) | DPIA §2.4 |
| 9 | Special category | Art. 9(2)(h) for PHI; explicit consent for KR sensitive | DPIA §4 |
| 13 + 14 | Information to subjects | Tenant notice | DPA |
| 17 | Erasure | DSR cascade | `policy/data-residency.md` |
| 22 | Automated decision-making | N/A (operational scheduling, not legal effect) | DPIA §6 R-04 |
| 25 | Privacy by design | Pseudonymisation + multi-tenancy + DSR | `policy/cluster-isolation.md` |
| 28 | Processor terms | DPA | `legal/dpa-template.md` |
| 30 | Records of processing | ROPA | `legal/ropa.md` |
| 32 | Security of processing | threat-model + cluster-isolation + Cedar | `threat-model.md` |
| 33 | Breach notification (72h) | incident-response procedure | `incident-response.md` |
| 35 | DPIA | This DPIA | `dpia.md` |
| 36 | Prior consultation | Not triggered | DPIA §7 |
| 44–46 | Cross-border | SCC-only | `legal/transfer-register.md` |

## Industry-Specific Frameworks

### CIS Kubernetes Benchmark v1.9

Enforced continuously by `oya-check-cis-k8s-benchmark` LEAN lane. Full control mapping at `microservices/cloud-k8s/evidence/cis-k8s-benchmark/<scan-date>.json`. Highlights:

| Section | Controls | Implementation |
|---|---|---|
| 1. Master Node Security Configuration | 1.1–1.4 (file perms, kube-apiserver flags, controller-manager flags, scheduler flags) | Helm + Kustomize set; LEAN lane verifies |
| 2. etcd | 2.1–2.7 (cert files, peer cert, peer client cert, peer auto-tls, client cert auth, peer-client-cert-auth, encryption) | Helm values enforced; KMS envelope |
| 3. Control Plane Configuration | 3.1, 3.2 (authentication + logging) | kube-apiserver flags |
| 4. Worker Node Security Configuration | 4.1 (file perms) + 4.2 (kubelet flags) | containerd + kubelet config |
| 5. Policies | 5.1 (RBAC), 5.2 (Pod Security Standards), 5.3 (Network Policies), 5.4 (Secrets), 5.7 (General) | Kyverno + Cilium + RBAC manifests |

### NSA/CISA Kubernetes Hardening Guide v1.2

Enforced by `oya-check-nsa-k8s-hardening` LEAN lane. Mapping in `microservices/cloud-k8s/evidence/nsa-k8s-hardening/`. Highlights: pod security; network separation; authentication + authorization; audit logging; upgrade + application practices.

### KR-CSAP (Cloud Security Assurance Program)

For KR-resident tenants. Cross-mapped at `regional-packs/pack-kr/compliance-overlay.md`:
- 가용성 (Availability) → multi-AZ + HA control plane
- 기밀성 (Confidentiality) → mTLS strict + at-rest encryption
- 무결성 (Integrity) → audit-chain seal
- 접근통제 (Access Control) → Cedar + RBAC
- 격리 (Isolation) → per-pack cluster boundary + namespace policy
- 로깅 (Logging) → audit retention ≥ 5y

## Per-Pack Frameworks

### pack-kr (KR PIPA + ISMS-P)

| Section | Requirement | Implementation |
|---|---|---|
| KR-ISMS-P §2.5 (인적보안) | JIT + background checks | OpenBao JIT + ops-security |
| KR-ISMS-P §2.6 (암호화) | TLS 1.3 + AES-256-GCM | A.8.24 |
| KR-ISMS-P §2.7 (접근통제) | Cedar + RBAC + 2-person rule | `policy/cluster-isolation.md` |
| KR-ISMS-P §2.8 (운영) | Runbooks + DR | `runbooks/*` |
| KR-ISMS-P §2.9 (사고관리) | 72h PIPC notification | `incident-response.md` |
| KR PIPA Art. 23 | Sensitive data tenant-id correlation | DPIA R-14 salt rotation |
| KR PIPA Art. 23-2 | Cross-border forbidden | `policy/data-residency.md` |
| KR PIPA Art. 28 | Storage limitation | retention matrix |
| KR PIPA Art. 29 | Technical safeguards | cross-mapped |
| KR PIPA Art. 29-2 | Encryption | A.8.24 |
| KR PIPA Art. 34 | Breach notification (72h) | `incident-response.md` |
| KR 전자문서법 Art. 5/6/7 | Electronic document integrity | Ed25519 audit-chain |

### pack-us-healthcare (HIPAA)

| §164 | Requirement | Implementation |
|---|---|---|
| §164.308(a)(1)(ii)(A) | Risk analysis | `threat-model.md` + `dpia.md` |
| §164.308(a)(1)(ii)(B) | Risk management | DPIA §6 |
| §164.308(a)(3) | Workforce security | OpenBao JIT |
| §164.308(a)(4)(ii)(B) | Access authorization | Cedar + RBAC |
| §164.308(a)(5) | Awareness + training | onboarding |
| §164.308(a)(6) | Security incident | `incident-response.md` |
| §164.308(a)(7) | Contingency plan | `multi-region.md` |
| §164.310(a) | Facility access | OCI HIPAA-eligible regions |
| §164.312(a)(1) | Access control | multi-tenancy + Cedar |
| §164.312(a)(2)(iv) | Encryption + decryption | etcd KMS-envelope + per-PV |
| §164.312(b) | Audit controls | audit-chain |
| §164.312(c)(1) | Integrity | Ed25519 + Cosign |
| §164.312(d) | Authentication | OIDC + MFA + SPIFFE |
| §164.312(e)(1) | Transmission security | Istio mTLS + TLS 1.3 |
| §164.314(a)(1) | BAA | `legal/baa-template.md` |
| §164.316(a)+(b)(2) | Retention ≥ 6y | retention matrix |
| §164.502(a) | TPO | DPIA §2.4 |
| §164.502(b) | Minimum necessary | workload-layer redactor |
| §164.514 | De-identification | pseudonymisation |
| §164.404 / §164.406 / §164.408 | Breach notification | `incident-response.md` |

### pack-eu (GDPR + EDPB + eIDAS + NIS2 + DORA)

- GDPR Arts. cited above.
- EDPB Guidelines 4/2019 (Art. 25) + 9/2022 (breach).
- eIDAS 910/2014 Art. 26: Ed25519 audit-chain = AdES.
- NIS2 incident-reporting 24h+72h+1mo.
- DORA 2022/2554 operational-resilience testing.

### pack-jp (APPI)

| Art. | Requirement | Implementation |
|---|---|---|
| APPI Art. 17 | Purpose of use | DPIA §2.4 |
| APPI Art. 18 | Purpose limitation | DPIA |
| APPI Art. 20 | Security control measures | `policy/cluster-isolation.md` |
| APPI Art. 21 | Entrustee supervision | sub-processors list |
| APPI Art. 23 | Third-party provision | DPA |
| APPI Art. 24 | Cross-border | `policy/data-residency.md` |
| APPI Art. 26-2 | Breach reporting | `incident-response.md` |
| APPI Art. 27 | Sensitive data consent | DPA |

### pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cloud-k8s-compliance-overlay.md`:
- pack-sg: PDPA 2012 + MAS Notice 644
- pack-au: Privacy Act 1988 APP 1–13 + APRA-CPS 234
- pack-in: DPDPA 2023 + RBI Master Direction on IT Outsourcing 2023
- pack-br: LGPD + BACEN Res. 4.893/2021
- pack-ae: UAE PDPL Federal Decree-Law 45/2021
- pack-ksa: KSA PDPL Royal Decree M/19/2021 + SAMA CSF 2017

## Continuous Compliance Evidence

### Lane: `oya-governance-compliance-evidence-recency`

Refuses merges if any evidence artifact older than 90 days is referenced as "current" without a refresh date stamp.

### Evidence emission

- `evidence/compliance/<framework>/<control>/<date>.json` — control evidence
- `microservices/cloud-k8s/evidence/multispectrum/<change_id>-<unix_ts>.json` — per-changeset evidence
- `microservices/cloud-k8s/evidence/cis-k8s-benchmark/<scan-date>.json` — CIS benchmark scan
- `microservices/cloud-k8s/evidence/nsa-k8s-hardening/<scan-date>.json` — NSA hardening scan

### Cadences

- Daily: CC4.x + CC7.x + A.8.15 + A.8.16; CIS K8s scan
- Weekly: CC8.x; A.5.27
- Monthly: CC3.x; A.5.7
- Quarterly: full matrix re-validated
- Annually: external auditor re-attestation

## Verification

- `cargo run -p oya-dev-cli -- gate validate compliance-evidence-recency` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate authority-cohesion` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cis-k8s-benchmark --microservice cloud-k8s` — exit 0.
- Annual SOC 2 Type 2: external auditor sign-off in `evidence/audits/soc2/<year>-type2-report.pdf`.
- Annual ISO 27001:2022 audit: analogous.

## References

- `microservices/cloud-k8s/threat-model.md`.
- `microservices/cloud-k8s/dpia.md`.
- `microservices/cloud-k8s/policy/{cluster-isolation, data-residency, tenant-scope, ci-scope, auditor-scope, public-read}.{md,cedar}`.
- `microservices/cloud-k8s/incident-response.md`.
- ADR-0028 (audit-chain); ADR-0117; ADR-0121; ADR-0123 (hyperscaler-maturity); ADR-0130; ADR-0131; ADR-0140 (Cedar).
- SOC 2 Type 2 TSC — `aicpa.org`.
- ISO 27001:2022 — `iso.org/standard/27001`.
- GDPR — `gdpr-info.eu`; EDPB — `edpb.europa.eu`.
- KR PIPA + ISMS-P — `pipc.go.kr` + `kisa.or.kr`.
- HIPAA — `hhs.gov/hipaa`.
- CIS Kubernetes Benchmark v1.9 — `cisecurity.org`.
- NSA/CISA Kubernetes Hardening Guide v1.2 — `nsa.gov`.
- KR-CSAP — `isms.kisa.or.kr`.
