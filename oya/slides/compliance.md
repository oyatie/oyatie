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

---



## §day-one-cert-readiness
This anchor is closed for `slides` against ADR-0250 §D-1: certification-ready evidence roster and audit scope.

### Service-specific answer
- Certification scope for `slides` covers packs `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Evidence collector classes: policy decision log, audit event seal, SLO burn-rate report, contract-schema validation, dependency/SBOM attestation, and runbook drill record.
- Primary evidence files: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +14 more.
- Example: `T0-suggest` readiness requires a signed audit event, an OpenAPI/AsyncAPI schema, an SLO target, and a pack-specific retention statement before launch.
- Retrofit is forbidden: controls land before certification audit, and audit artifacts are generated continuously rather than assembled after an incident.
- SOC 2 maps to access, change, logging, and incident controls; ISO 27001 maps to Annex A domains; regional packs add local regulator timing.
- Day-one means the µservice can enter a certification audit without architecture changes, not that the external certificate is already issued.
- Missing evidence is treated as REVISE and listed as a structural issue below.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: AWS Artifact evidence portal is the reference pattern for the control shape described here.
- Precedent 2: Google Assured Workloads control mapping is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against ADR-0251 §D-2: pack activation, overlays and per-pack Cedar deltas.

### Service-specific answer
- Active/expected pack roster: `kr`, `eu`, `us`, `us-healthcare`, `jp`, `sg`; +5 more.
- Pack overlays modify Cedar fragments `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar` without changing domain code.
- Data classes under pack control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Higher-restriction-wins: if GDPR conflicts with another pack, the stricter storage, transfer, notice, or access rule applies until legal workflow resolves it.
- CN-PIPL-2021 is activated on CN `jurisdiction_code`; KR packs pin data to KR cells; EU sovereign packs prevent non-EU failover unless explicitly allowed.
- Example: `T0-suggest` under KR pack uses KR cell routing, KR breach-notification timers, and pack-local audit retention.
- Pack activation is tenancy-owned, consumed by this µservice through Ontology/tenant projection, then enforced by Cedar and storage routing.
- No ad-hoc pack ids are introduced here; all ids must resolve through the central compliance-pack registry.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: AWS Control Tower guardrails is the reference pattern for the control shape described here.
- Precedent 2: Microsoft Purview Compliance Manager is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against ADR-0292 §D-1: minor-user refusal, teen tier and age-verification handling.

### Service-specific answer
- Minor exposure for `slides` is derived from audience `B2C_CONSUMER + B2B_TENANT` and data classes `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- Under-13 COPPA path refuses non-exempt consumer processing unless a child-safety or crisis exception applies; refusal emits an audit event.
- Ages 14-17 use KOSA-style high-privacy defaults, no dark patterns, reduced recommendation/engagement pressure, and guardian flows where lawful.
- EU under-18 flows require age verification token where the pack mandates it; no raw age document is retained by this µservice unless explicitly scoped.
- Example: `T0-suggest` checks `principal.age_class` before any personalization, payment, public-sharing, messaging, or recommendation-affecting mutation.
- Crisis-hotline and mandatory-reporting exceptions bypass friction while retaining audit and post-hoc accountability.
- Metrics track refusal count, teen-tier activation, age-token verification failure, and false-positive appeal outcomes with no raw minor identifier labels.
- If this µservice is not consumer-facing, this section records the inherited deny-by-default stance for accidental minor-targeted use.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: Apple Screen Time/Family controls is the reference pattern for the control shape described here.
- Precedent 2: Google Family Link teen safety pattern is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against ADR-0284 §D-1: platform_owner indirection and hard-coded brand-string audit.

### Service-specific answer
- Runtime platform-owner string is configured as `platform_owner.display_name`; `slides` does not hard-code user-visible owner names in API or UI output.
- Internal principal names may retain `oyatie.*` because ADR-0242 treats `oyatie` as the platform tenant, not as user-visible branding.
- Surfaces audited for display strings: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`, `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`; +16 more.
- API responses expose owner references as opaque ids or config-resolved display names; logs keep stable tenant/platform ids for auditability.
- Example: `T0-suggest` error text says `platform owner` or config-resolved name, while audit event principal remains `oyatie.slides.runtime`.
- Grep-audit evidence records exceptions: principal slugs, ADR citations, internal package names, and provenance fields are allowed when not user-visible.
- White-label tenants can override tenant-facing support links without changing compliance evidence, Cedar principals, or audit event taxonomy.
- This closes ADR-0284 without erasing canonical internal identity semantics.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: Salesforce My Domain tenant branding is the reference pattern for the control shape described here.
- Precedent 2: Google Workspace tenant branding is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.6.A: detection-event categories and routing topology.

### Service-specific answer
- `slides` emits detection signals through ADR-0263 audit pipeline, not an ungoverned side channel.
- Detection families applicable here: policy violation, insider risk, account-takeover, content/transaction abuse where `T0-suggest` touches those data classes.
- Signal sources: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`; +13 more.
- Example event class: `oya.slides.t0.suggest.risk_signal_emitted` with risk score, reason code, and tenant-safe dimensions.
- Routing topology: µservice audit event -> observability collector -> detection substrate -> investigation workflow when threshold and policy allow.
- False positives feed back through investigation labels; thresholds are versioned and auditable.
- Detection never becomes secret policy: users/tenants get explanation and appeal where law or product doctrine requires it.
- If model-driven scoring is absent, deterministic rules still emit detection events and declare no local ML model in the lifecycle section.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: AWS GuardDuty/Security Hub findings is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle detection pipeline is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against ADR-0310 §D-1: detection-to-investigation evidence handoff and case binding.

### Service-specific answer
- Investigation handoff starts from a signed detection event emitted by `slides` and ends in a case record with immutable evidence pointers.
- Cedar permit `oyatie.slides.investigation.open` gates who may create, read, export, or close a case.
- Evidence pack includes audit event id, policy decision hash, affected `T0-suggest` resource ids, tenant id, data classes, and SLO/degraded-mode context.
- Investigator access is read-only by default, time-boxed, purpose-bound, and visible in tenant/admin transparency where law permits.
- Case routing binds to workflow-engine for orchestration and audit-chain for seal verification; no investigation artifact is stored only in chat/email.
- Example: a suspicious `T0-suggest` mutation opens a case only after risk threshold and Cedar permit both pass; low-confidence signals remain queued for aggregation.
- Closure records final disposition, remediation, appeal outcome, regulator notifications, and model/rule feedback labels.
- Retention follows highest active compliance pack and legal hold state.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: AWS Detective investigation graph is the reference pattern for the control shape described here.
- Precedent 2: Google Chronicle SOAR case handoff is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 8: privileged access, break-glass and UEBA controls.

### Service-specific answer
- Operators of `slides` have no standing unredacted tenant-data access; JIT elevation uses identity step-up and Cedar approval.
- Break-glass access requires reason, scope, expiry, reviewer where required, and post-hoc audit review.
- Sensitive surfaces: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`, `slides.unknown`.
- UEBA signals include unusual export volume, after-hours privileged reads, cross-cell access, pack-boundary reads, and repeated denied Cedar decisions.
- Example: reading `slides.unknown` outside declared incident purpose creates a high-risk insider signal and routes to investigation.
- Privileged-access review cadence is monthly for Tier 0/1, quarterly otherwise, and after every SEV/security incident.
- Logs redact data but retain enough metadata to prove access purpose, approver, principal, and affected data class.
- Emergency break-glass optimizes for safety but never skips audit sealing.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview Insider Risk Management is the reference pattern for the control shape described here.
- Precedent 2: Google BeyondCorp zero-trust access is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 9: threat feed sources, freshness and degraded-mode policy.

### Service-specific answer
- `slides` consumes central threat intelligence for IP/domain reputation, credential stuffing, bot fingerprints, sanctions/abuse lists where applicable, and malicious package indicators.
- Feed freshness SLOs: ≤1h for IP/domain/bot reputation, ≤24h for credential corpus, immediate for emergency blocklists and compromised provider credentials.
- Enforcement points: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`; +12 more.
- Example: `T0-suggest` with malicious IP reputation receives stricter quota/challenge; high-risk legal/financial flows can halt pending investigation.
- Feed outage degraded mode raises sensitivity only on suspicious paths and never adds default friction to clean traffic.
- Feed source, version, checksum, and last refresh timestamp are emitted in audit evidence.
- Sanctions and law-enforcement feeds are pack-aware and never applied outside their legal scope without policy review.
- False positives can be appealed and fed back into allow-list/threshold tuning.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: Mandiant threat intelligence feeds is the reference pattern for the control shape described here.
- Precedent 2: AWS GuardDuty managed threat lists is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 16: signing, encryption, ECH and PQC key rotation cadence.

### Service-specific answer
- Signing keys for `oya.slides` audit events rotate at ≤90 days or immediately on suspected compromise.
- OpenBao dynamic credentials rotate at TTL ≤60s for provider/API secrets unless sidecar keeps the raw secret isolated.
- Encryption/data keys rotate at ≤1 year or pack-specific shorter cadence; ECH keys rotate at ≤90 days; PQC cert chains follow signing-key cadence.
- Secret paths use `${openbao:secret/<tenant_id>/slides/<key-class>}` and never embed raw tenant ids in metrics labels.
- Runbook evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.
- Example: `T0-suggest` credential rotation drains in-flight requests with old key id, validates new key id, then retires old leases after audit-chain seal.
- Rotation failure alerts within 5 minutes for Tier 0/1 and within 15 minutes otherwise.
- Rollback uses previous active key version only inside the documented grace window and emits an exception event.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: AWS KMS automatic rotation is the reference pattern for the control shape described here.
- Precedent 2: Google Cloud KMS key versions is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 20: algorithm roster, deprecation triggers and migration windows.

### Service-specific answer
- `slides` uses algorithm policy from sidecar/config; domain code never hard-codes cipher or signature choices.
- Current floor: TLS 1.3, AEAD-only suites, X25519, hybrid X25519MLKEM768 where supported, Ed25519 plus ML-DSA-65 for new platform-rooted chains.
- Forbidden: SHA-1, MD5, RSA-1024/2048 for new signatures, static DH, CBC-only TLS, self-signed production certs, and bespoke crypto.
- Affected surfaces: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`, `microservices/slides/iac/helm/Chart.yaml`, `microservices/slides/iac/helm/templates/deployment.yaml`, `microservices/slides/iac/helm/templates/hpa.yaml`; +9 more.
- Migration trigger: NIST/IETF/browser deprecation notice, active exploit, pack regulator requirement, or platform crypto policy update.
- Migration window: 90 days for normal deprecation, 24h emergency block for actively exploited algorithms, with compatibility fallback only when safe.
- Example: `T0-suggest` accepts classical TLS during PQC migration but prefers hybrid when both peers support it and records negotiated group in telemetry.
- Agility verification checks config, cert chain, dependency inventory, and runtime negotiated parameters.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: Cloudflare post-quantum TLS rollout is the reference pattern for the control shape described here.
- Precedent 2: Google Chrome hybrid post-quantum TLS is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 12: pentest scope, bounty intake and remediation SLO.

### Service-specific answer
- `slides` is in annual full-scope pentest and every major `T0-suggest` launch adds targeted test scope before production promotion.
- In-scope assets: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`, `microservices/slides/iac/helm/Chart.yaml`, `microservices/slides/iac/helm/templates/deployment.yaml`, `microservices/slides/iac/helm/templates/hpa.yaml`; +15 more.
- Bug bounty intake accepts auth, tenant-isolation, policy bypass, data exposure, abuse-defence false positive/negative, supply-chain, and crypto findings.
- Critical findings block promotion; remediation SLO is 24h containment, 7d fix for critical/high, 30d medium unless regulator pack is stricter.
- Example: a researcher bypassing `slides` tenant scoping gets safe-harbor handling and an investigation case, not abuse-defence friction by default.
- Retest evidence includes reproduction, patch commit, regression test, policy diff, and audit event proving closure.
- Findings are linked to scorecards and risk register; repeated classes feed the prevention backlog.
- Emergency-services/critical-path paths are pentested with safety bypass rules active, not disabled.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: Google Vulnerability Reward Program is the reference pattern for the control shape described here.
- Precedent 2: HackerOne managed bounty programs is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 13: inherited data-center, cell and physical-access controls.

### Service-specific answer
- `slides` inherits facility controls from `cell`, `cloud-iac`, and the active provider cell; no direct human facility access is owned by this µservice.
- Cell eligibility `not declared in manifest; bound here to the conservative platform default` determines whether Tier 0/1 hardened node pools and stronger physical attestation apply.
- Physical controls include badge/biometric access, cage/rack separation, CCTV, visitor logging, media destruction, environmental controls, and annual attestation review.
- Pack-specific facility evidence is referenced for HIPAA, PCI, FedRAMP/IL5, KR, EU sovereign, and CN sovereign where active.
- Example: `T0-suggest` in a regulated cell can only schedule onto node pools with matching facility attestation and residency tag.
- Facility incident response routes through cell/cloud-iac runbooks and still emits µservice impact evidence.
- If on-prem deployment is used, the facility attestation must be attached before this µservice can claim certification-ready status.
- No facility claim here overrides missing provider attestation.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: AWS data-center layered physical security is the reference pattern for the control shape described here.
- Precedent 2: Google data-center physical security controls is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 19: SBOM, signed artifacts, dependency pinning and provenance.

### Service-specific answer
- `slides` dependency inventory spans crates/catalog, containers, Helm/Kustomize/OpenTofu, Cedar fragments, contracts, and generated SDKs.
- Inventory artifacts: `microservices/slides/catalog/oya-slides-acl-adapter-postgres.yaml`, `microservices/slides/catalog/oya-slides-acl-domain.yaml`, `microservices/slides/catalog/oya-slides-acl-kernel.yaml`, `microservices/slides/catalog/oya-slides-ai-content-generation-domain.yaml`, `microservices/slides/catalog/oya-slides-broadcast-mode-adapter-livekit.yaml`, `microservices/slides/catalog/oya-slides-image-adapter-clamav.yaml`; +21 more.
- Every build emits SBOM, provenance, source commit, builder identity, dependency digests, and signature/transparency-log pointers.
- Dependencies are pinned to exact versions/digests; unpinned charts/images/crates block promotion.
- Example: `T0-suggest` image promotion requires cosign signature, SLSA provenance, vulnerability scan, license check, and matching manifest/catalog record.
- Critical CVEs trigger containment within 24h; vulnerable optional adapters can be disabled by Cedar/feature flag while core remains available.
- Supplier risk includes PSP/API vendors, model providers, cloud services, package registries, and CI/CD providers.
- Reproducibility check compares built artifact digest against provenance before deployment.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: SLSA provenance framework is the reference pattern for the control shape described here.
- Precedent 2: Sigstore Cosign/Fulcio/Rekor chain is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.5: applicable safety/security/policy edge cases and fallbacks.

### Service-specific answer
- Applicable rows for `slides` include account recovery, mistaken mutation, regional outage, regulator deadline, audit access, and delegated-agent authority where relevant.
- Safety invariant: `T0-suggest` never creates human harm through unnecessary friction, lost recovery path, or silent data loss.
- Security invariant: bypasses require attestation, audit, revocation, and scoped duration; no broad allow-list or fail-open behavior.
- Policy invariant: highest active compliance pack controls retention, transfer, notice, appeal, and regulator timing.
- Example: `T0-suggest` during regional outage preserves audit evidence locally and blocks cross-border transfer if pack policy forbids DR failover.
- Edge-case tests must prove behavior for network partition, key compromise, stale pack activation, audit pipeline backpressure, and byzantine caller.
- Every edge-case row cites runbook, Cedar policy, audit event, and CI lane evidence.
- Missing path becomes REVISE; this section is the operator/auditor map, not a future TODO.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: AWS Well-Architected resilience review is the reference pattern for the control shape described here.
- Precedent 2: Google SRE emergency rollback practice is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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
This anchor is closed for `slides` against documentation-rigor.md §3.2.4 Domain 14: data classes, retention, encryption and transfer restrictions.

### Service-specific answer
- Data classes processed: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.
- State/event surfaces carrying classification: `slides.unknown`.
- Every ingested field has data class, purpose, retention, residency, encryption, disclosure, and DSR behavior declared before storage.
- Classification changes are migrations: they require audit evidence, backfill/replay plan, and pack-specific review.
- Example: `T0-suggest` labels identifiers as `PII_IDENTIFYING`, operational logs as `AUDIT`, and aggregate metrics as `INTERNAL_ONLY` unless manifest narrows them.
- Misclassification detection emits an incident signal, quarantines affected records where possible, and blocks export until corrected.
- Cross-border transfer checks use `jurisdiction_code`, `home_cell`, pack roster, and data class together; no single field decides alone.
- Public data must still be explicitly classified; absence of classification is never treated as public.

### Concrete inventory used
- Service: `slides`; owner `axis-slides`; tier `product`; audience `B2C_CONSUMER + B2B_TENANT`.
- Bounded contexts used for this answer: `unknown`.
- Capability records cited: `microservices/slides/capabilities/T0-suggest.yaml`, `microservices/slides/capabilities/T1-assist.yaml`, `microservices/slides/capabilities/T2-auto.yaml`.
- API surfaces cited: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar/policy artifacts cited: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- SLO and dashboard evidence: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +7 more.
- Runbook/IaC evidence: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +13 more.
- Data classes declared for this control: `INTERNAL_ONLY`, `AUDIT`, `PII_QUASI`.

### Primitive and API binding
- API surface binding: `microservices/slides/contracts/asyncapi/slides-events.yaml`, `microservices/slides/contracts/openapi/slides.yaml`, `microservices/slides/contracts/proto/slides.proto`.
- Cedar binding: `microservices/slides/policy/auditor-scope.cedar`, `microservices/slides/policy/ci-scope.cedar`, `microservices/slides/policy/data-residency.md`, `microservices/slides/policy/editor-isolation.md`, `microservices/slides/policy/public-read.cedar`, `microservices/slides/policy/tenant-scope.cedar`.
- State/event binding: `slides.unknown`.
- Capability binding: `T0-suggest`, `T1-assist`, `T2-auto`.
- SLO binding: `microservices/slides/slos/broadcast-mode-availability.openslo.yaml`, `microservices/slides/slos/collab-cursor-sync-latency.openslo.yaml`, `microservices/slides/slos/crdt-merge-no-silent-loss.openslo.yaml`, `microservices/slides/slos/deck-open-latency.openslo.yaml`, `microservices/slides/slos/export-mp4-latency.openslo.yaml`, `microservices/slides/slos/export-pdf-latency.openslo.yaml`; +4 more.
- Runbook binding: `microservices/slides/runbooks/animation-engine-rollback.md`, `microservices/slides/runbooks/attachment-restore.md`, `microservices/slides/runbooks/broadcast-mode-degraded.md`, `microservices/slides/runbooks/collab-conflict-resolution-crdt.md`, `microservices/slides/runbooks/export-pipeline-failure-pptx.md`, `microservices/slides/runbooks/share-acl-drift.md`; +1 more.

### Cross-service links
- `tenancy` provides tenant lifecycle, `tenant_id`, `audience_type`, pack activation, and provider-BYOK mode consumed by `slides`.
- `identity` provides `principal_id`, SVID/auth context, step-up state, and age/audience claims consumed by `slides`.
- `policy-engine` supplies the signed Cedar corpus while `slides` performs caller-side library evaluation.
- `observability` and `audit-chain` receive signed audit events and SLO evidence for this anchor.
- `cloud-secrets` supplies OpenBao references for `slides` credential and signing-key isolation.
- `cell` and `cloud-iac` enforce the runtime cell, ingress, ECH/PQC, and facility posture for `slides`.

### Hyperscaler precedents
- Precedent 1: Microsoft Purview data classification is the reference pattern for the control shape described here.
- Precedent 2: AWS Macie sensitive-data discovery is the second reference pattern used to avoid a single-vendor cargo-cult design.
- The adaptation keeps the hyperscaler property that control evidence is observable, versioned, reversible, and tenant-scoped.
- The adaptation rejects hidden tribal knowledge: a cold reader can trace service, policy, storage, runtime, and audit evidence from this section.

### Failure modes and rollback
- Failure mode: stale tenant or pack projection. Behavior: `slides` applies the most restrictive policy and emits a degraded-mode audit event.
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

