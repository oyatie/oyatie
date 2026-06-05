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
| OpenSSF Best Practices Badge | OSS-style best practices (advanced badge target) | OpenSSF |
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
| CC8.1 | Change management | every change via PR + ~50-lane set; ADR successor-IP for material changes |
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
| A.5.8 | Information security in project management | per-PR ~50-lane set |
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
| A.8.25 | Secure development lifecycle | SLSA L3 + per-PR ~50-lane set |
| A.8.26 | Application security requirements | OWASP ASVS v4 mapping |
| A.8.27 | Secure system architecture + engineering principles | clean-arch per `feedback_clean_architecture_requirements.md` |
| A.8.28 | Secure coding | rustfmt + clippy -D warnings + `cargo deny check` |
| A.8.30 | Outsourced development | not applicable; agentic dev team is in-house |
| A.8.31 | Separation of development, test + production environments | per-pack overlays; staging vs production refs per ADR-0139 |
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
| 2026-Q2 | all 6 axes (baseline) | findings table populated from the quarterly Buck2/Prow governance scan output on first quarter close; baseline-quarter row updates to count + breakdown post-run | filed under `microservices/governance/IP-M01-AUDIT-*-NNN.md` |

## Verification

- `buck2 build //:quality-lane-registry-authority-check # lane=compliance --microservice governance` — exit 0.
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

---



## §day-one-cert-readiness
This anchor is closed for `governance` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `governance` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +12 more.
- Example: `audit-refresh` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `audit-refresh` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `governance` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`, `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`; +17 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `audit-refresh` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.governance.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `governance` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `audit-refresh` touches those data classes.
- Signal sources: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +11 more.
- Example event class: `oya.governance.audit.refresh.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §ml-model-lifecycle
This anchor is closed for `governance` against documentation-rigor.md §3.2.6.E: model inventory, retrain cadence and promotion gates.

### Service-specific answer
- Local ML posture: `False` for direct model use; inherited detection/intelligence models still require versioned consumption evidence.
- Model inventory key: `manifest.json:ml_models` or the Intelligence audience tag `governance.audit-refresh` if models are substrate-hosted.
- Promotion gates: offline eval, bias/fairness report, drift threshold, SLO budget, rollback model id, and human approval for high-risk/adverse-action paths.
- Retraining cadence is model-specific; high-risk models require documented data cut, feature schema, holdout set, and pack-specific legal review.
- Example: `audit-refresh` model output is never the sole authority for a legal/financial/employment/minor-impacting decision; Cedar and human-review policies remain in control.
- Deprecated model versions sunset under ADR-0258 with traffic split, canary, rollback, and post-promotion audit.
- Model cards include intended use, non-use, data provenance, performance by segment, failure modes, and owner.
- Services without local models keep this as a negative declaration so future agents cannot silently add ML without the lifecycle gate.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: NIST AI RMF model-governance lifecycle is the reference pattern for the control shape described here.
- Precedent 2: Google Model Cards is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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

## §detection-fairness-audit
This anchor is closed for `governance` against documentation-rigor.md §3.2.6.E: fairness metrics, thresholds and disaggregated false-positive audit.

### Service-specific answer
- Fairness audit applies to `governance` risk/detection decisions that affect access, ranking, safety, money, employment, health, or protected classes.
- Metrics: false-positive rate ratio, false-negative rate ratio, calibration by segment, equalized-odds gap, appeal overturn rate, and challenge-friction rate.
- Thresholds: no protected segment exceeds 1.25x baseline false-positive rate without documented mitigation and human review.
- Segments are derived from lawful, minimized attributes; `governance` never stores protected attributes solely to make a product feature easier.
- Example: `audit-refresh` abuse/risk score challenge rate is compared across locale, accessibility profile, age tier, and jurisdiction pack.
- Audit cadence: every model/rule promotion, quarterly for active high-risk detectors, and after any SEV involving false positives.
- Fairness reports are retained in audit evidence; raw protected-attribute joins remain in restricted analytics cells.
- If the service has no ML, deterministic rules still get false-positive and appeal-rate monitoring.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: Microsoft Fairlearn audit pattern is the reference pattern for the control shape described here.
- Precedent 2: NIST AI RMF measurement function is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `governance` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.governance.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `audit-refresh` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `audit-refresh` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `governance` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`, `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`; +2 more.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `governance.aggregation_indexer` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `governance` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +13 more.
- Example: `audit-refresh` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.governance` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/governance/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.
- Example: `audit-refresh` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `governance` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`, `microservices/governance/iac/build/Dockerfile.distroless-rust`, `microservices/governance/iac/helm/_oya-helpers/Chart.yaml`, `microservices/governance/iac/helm/_oya-helpers/templates/_helpers.tpl`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `audit-refresh` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `governance` is in annual full-scope pentest and every major `audit-refresh` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`, `microservices/governance/iac/build/Dockerfile.distroless-rust`, `microservices/governance/iac/helm/_oya-helpers/Chart.yaml`, `microservices/governance/iac/helm/_oya-helpers/templates/_helpers.tpl`; +16 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `governance` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `governance` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `audit-refresh` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `governance` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/governance/catalog/oya-check-data-class.yaml`, `microservices/governance/catalog/oya-check-lean-a1.yaml`, `microservices/governance/catalog/oya-check-lean-a2.yaml`, `microservices/governance/catalog/oya-check-license-policy.yaml`, `microservices/governance/catalog/oya-check-supply-chain.yaml`, `microservices/governance/catalog/oya-governance-aggregation-indexer-adapter.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `audit-refresh` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `governance` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `audit-refresh` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `audit-refresh` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `governance` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `audit-refresh` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `governance`; owner `axis-governance`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `aggregation-indexer`, `bundled-check-crate`, `evidence-emitter`, `lane-runtime`, `policy-engine`.
- Capability records cited: `microservices/governance/capabilities/audit-refresh.yaml`, `microservices/governance/capabilities/finding-query.yaml`, `microservices/governance/capabilities/lane-execute.yaml`.
- API surfaces cited: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar/policy artifacts cited: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- SLO and dashboard evidence: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +4 more.
- Runbook/IaC evidence: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +14 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/governance/contracts/asyncapi/governance-events.yaml`, `microservices/governance/contracts/openapi/governance.yaml`, `microservices/governance/contracts/proto/governance.proto`.
- Cedar binding: `microservices/governance/policy/auditor-scope.cedar`, `microservices/governance/policy/cedar-canonical-imports.cedar`, `microservices/governance/policy/ci-scope.cedar`, `microservices/governance/policy/data-residency.md`, `microservices/governance/policy/lane-execution.md`, `microservices/governance/policy/public-read.cedar`; +1 more.
- State/event binding: `governance.aggregation_indexer`, `governance.bundled_check_crate`, `governance.evidence_emitter`, `governance.lane_runtime`, `governance.policy_engine`.
- Capability binding: `audit-refresh`, `finding-query`, `lane-execute`.
- SLO binding: `microservices/governance/slos/aspirational-enforcement-correctness.openslo.yaml`, `microservices/governance/slos/check-crate-availability.openslo.yaml`, `microservices/governance/slos/conformance-evidence-freshness.openslo.yaml`, `microservices/governance/slos/envoy-wasm-filter-latency-p99.openslo.yaml`, `microservices/governance/slos/gate-validate-latency.openslo.yaml`, `microservices/governance/slos/per-lane-runtime-budget.openslo.yaml`; +1 more.
- Runbook binding: `microservices/governance/runbooks/aggregation-rebuild.md`, `microservices/governance/runbooks/envoy-wasm-filter-rollback.md`, `microservices/governance/runbooks/evidence-replay.md`, `microservices/governance/runbooks/industry-baseline-refresh.md`, `microservices/governance/runbooks/lane-bypass-emergency.md`, `microservices/governance/runbooks/lane-failure-triage.md`; +2 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `governance`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `governance`.
- `policy-engine` supplies the signed Cedar corpus while `governance` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `governance` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `governance`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `governance` applies the most restrictive policy and emits a degraded-mode audit event.
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
