---
doc_class: ComplianceSpec
title: Compliance Control-to-Framework Mapping
microservice: cloud-k8s
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-privacy + ops-compliance + ops-security
deciders: council-privacy, ops-security, axis-cloud, council-architecture, ops-compliance
related_adrs: [ADR-0028, ADR-0117, ADR-0121, ADR-0123, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
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
- ADR-0028 (audit-chain); ADR-0117; ADR-0121; ADR-0123 (hyperscaler-maturity); ADR-0139; ADR-0131; ADR-0140 (Cedar).
- SOC 2 Type 2 TSC — `aicpa.org`.
- ISO 27001:2022 — `iso.org/standard/27001`.
- GDPR — `gdpr-info.eu`; EDPB — `edpb.europa.eu`.
- KR PIPA + ISMS-P — `pipc.go.kr` + `kisa.or.kr`.
- HIPAA — `hhs.gov/hipaa`.
- CIS Kubernetes Benchmark v1.9 — `cisecurity.org`.
- NSA/CISA Kubernetes Hardening Guide v1.2 — `nsa.gov`.
- KR-CSAP — `isms.kisa.or.kr`.

---



## §day-one-cert-readiness
This anchor is closed for `cloud-k8s` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `cloud-k8s` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +12 more.
- Example: `cluster-bootstrap` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `cluster-bootstrap` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `cloud-k8s` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`, `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`; +18 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `cluster-bootstrap` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.cloud-k8s.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `cloud-k8s` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `cluster-bootstrap` touches those data classes.
- Signal sources: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`; +9 more.
- Example event class: `oya.cloud.k8s.cluster.bootstrap.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `cloud-k8s` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.cloud-k8s.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `cluster-bootstrap` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `cluster-bootstrap` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `cloud-k8s` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`, `cloud_k8s.cloud_k8s`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `cloud_k8s.cloud_k8s` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `cloud-k8s` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`; +12 more.
- Example: `cluster-bootstrap` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.cloud-k8s` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/cloud-k8s/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.
- Example: `cluster-bootstrap` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `cloud-k8s` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`, `microservices/cloud-k8s/iac/helm/cni-cilium/Chart.yaml`, `microservices/cloud-k8s/iac/helm/cni-cilium/values.yaml`, `microservices/cloud-k8s/iac/helm/envoy-gateway/Chart.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `cluster-bootstrap` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `cloud-k8s` is in annual full-scope pentest and every major `cluster-bootstrap` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`, `microservices/cloud-k8s/iac/helm/cni-cilium/Chart.yaml`, `microservices/cloud-k8s/iac/helm/cni-cilium/values.yaml`, `microservices/cloud-k8s/iac/helm/envoy-gateway/Chart.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `cloud-k8s` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `cloud-k8s` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `cluster-bootstrap` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `cloud-k8s` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-adapter-containerd.yaml`, `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-adapter-kubeadm.yaml`, `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-adapter.yaml`, `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-api.yaml`, `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-app.yaml`, `microservices/cloud-k8s/catalog/oya-cloud-k8s-cluster-bootstrap-domain.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `cluster-bootstrap` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `cloud-k8s` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `cluster-bootstrap` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `cluster-bootstrap` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `cloud-k8s` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `cloud_k8s.cloud_k8s`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `cluster-bootstrap` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `cloud-k8s`; owner `axis-cloud-k8s`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `cloud-k8s`.
- Capability records cited: `microservices/cloud-k8s/capabilities/cluster-bootstrap.yaml`, `microservices/cloud-k8s/capabilities/network-policy-apply.yaml`, `microservices/cloud-k8s/capabilities/node-lifecycle.yaml`.
- API surfaces cited: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar/policy artifacts cited: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`; +3 more.
- Runbook/IaC evidence: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +15 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/cloud-k8s/contracts/asyncapi/cloud-k8s-events.yaml`, `microservices/cloud-k8s/contracts/openapi/cloud-k8s.yaml`, `microservices/cloud-k8s/contracts/proto/cloud-k8s.proto`.
- Cedar binding: `microservices/cloud-k8s/policy/auditor-scope.cedar`, `microservices/cloud-k8s/policy/ci-scope.cedar`, `microservices/cloud-k8s/policy/cluster-isolation.md`, `microservices/cloud-k8s/policy/data-residency.md`, `microservices/cloud-k8s/policy/public-read.cedar`, `microservices/cloud-k8s/policy/tenant-scope.cedar`.
- State/event binding: `cloud_k8s.cloud_k8s`.
- Capability binding: `cluster-bootstrap`, `network-policy-apply`, `node-lifecycle`.
- SLO binding: `microservices/cloud-k8s/slos/cis-benchmark-conformance.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-api-availability.openslo.yaml`, `microservices/cloud-k8s/slos/cluster-cni-availability.openslo.yaml`, `microservices/cloud-k8s/slos/node-readiness-correctness.openslo.yaml`, `microservices/cloud-k8s/slos/pod-scheduling-latency.openslo.yaml`, `microservices/cloud-k8s/slos/service-mesh-availability.openslo.yaml`.
- Runbook binding: `microservices/cloud-k8s/runbooks/control-plane-restore.md`, `microservices/cloud-k8s/runbooks/csi-rebuild.md`, `microservices/cloud-k8s/runbooks/envoy-sni-debug.md`, `microservices/cloud-k8s/runbooks/etcd-quorum-recovery.md`, `microservices/cloud-k8s/runbooks/ingress-ddos-throttle.md`, `microservices/cloud-k8s/runbooks/istio-mtls-rotation.md`; +3 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `cloud-k8s`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `cloud-k8s`.
- `policy-engine` supplies the signed Cedar corpus while `cloud-k8s` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `cloud-k8s` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `cloud-k8s`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `cloud-k8s` applies the most restrictive policy and emits a degraded-mode audit event.
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

