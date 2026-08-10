# EMR — Electronic Medical Record / Electronic Health Record

> **Status:** Wave 15M-B authored 2026-05-21. Authoring of the foundational healthcare µservice; companion µservices (`diagnostics`, `pharmacy`, `emergency`, `clinical-decision-support`, `care-management`) author independently.

## Mission

EMR is the clinical Record-Of-Truth for oyatie's healthcare offering. It owns 15 bounded contexts — Patient, Encounter, Problem, Medication, Allergy, Vital, Note, Order, Result, CareTeam, OrderSet, Documentation, BillingCode, PatientEducation, PortalSession — and exposes a FHIR R5 (default) + R4 (compatibility) API surface plus a tenant-class-aware patient + caregiver portal.

Every other healthcare µservice in the oyatie portfolio reads from or writes to EMR. EMR is the spine; everything else is a peer module.

## Counterparts the EMR µservice positions against

### Primary

- **Epic Systems** — Epic Hyperspace + EpicCare (inpatient) + Epic Spring (ambulatory) + MyChart (patient portal) + Beaker (lab) + Willow (pharmacy) + Cupid (cardiology) + Stork (OB) + Radiant (imaging). Epic is the US inpatient + large-IDN incumbent (~38% market share). Recognized clinical-UX leader; closed clinical-content store; vendor lock-in via Care Everywhere proprietary exchange.
- **Oracle Health Cerner** (post-2022 acquisition) — Cerner Millennium + PowerChart + CommunityWorks + HealtheLife. US #2 (~25%). Extensive interface-engine heritage; consolidating into Oracle's broader Healthcare Cloud.
- **athenahealth** — athenaClinicals + athenaCommunicator + athenaCollector + Population Health Insights. Cloud-first ambulatory leader (~12% of ambulatory market). Disrupted the ambulatory segment by bundling EHR with RCM (revenue cycle management) on a same-platform substrate.

### Secondary

- **Allscripts (Veradigm)** — Sunrise EHR (inpatient) + Paragon + TouchWorks (ambulatory) + Practice Fusion (small-practice). Mid-market and small-practice presence.
- **Meditech** — Meditech Expanse + Patient + Web Ambulatory. Community-hospital strength.
- **eClinicalWorks** — eClinicalWorks V11 + healow (patient app) + eCW Scribe (AI documentation). Small-to-mid ambulatory.
- **NextGen Healthcare** — NextGen Enterprise + NextGen Mobile + NextGen Office. Specialty-clinic focus.
- **Greenway Health** — Intergy + Prime Suite. Ambulatory.

## What EMR ships

| Capability | Scope | Counterpart parity |
|---|---|---|
| Patient demographics + MPI link | FR-PAT-001..006 | Epic + Cerner + athena |
| Clinical encounters (8 types) | FR-ENC-007..010 | Epic + Cerner + athena |
| Problem list (SNOMED + ICD-10) | FR-PROB-011..014 | Epic + Cerner + athena |
| Medication list + reconciliation | FR-MED-015..020 | Epic + Cerner + athena |
| Allergy + intolerance list | FR-ALG-021..023 | Epic + Cerner + athena |
| Vital signs + streaming | FR-VIT-024..026 | Epic + Cerner; surpasses athena in continuous monitoring |
| Clinical notes + dictation + amendments | FR-NOTE-027..033 | Epic; surpasses Cerner in CRDT-conflict-free autosave |
| CPOE (order entry) | FR-ORD-034..043 | Epic + Cerner; surpasses athena in inpatient CPOE |
| Results review + ack | FR-RES-044..047 | Epic + Cerner + athena |
| Care team | FR-CT-048..050 | Epic + Cerner + athena |
| Order sets + protocols + A/B | FR-OS-051..054 | Epic; surpasses Cerner + athena in A/B-eligible governance |
| Documentation templates + smart-phrases | FR-DOC-055..057 | Epic SmartTexts + Cerner DragonForms parity |
| Billing-code capture (CPT/HCPCS/ICD-10) | FR-BIL-058..060 | athena; parity with Epic Resolute + Cerner RCM |
| Patient education (multi-language) | FR-PED-061..062 | Epic + Cerner + athena |
| Patient portal session (mobile + web) | FR-PORT-063..066 | MyChart-parity at minimum; mobile-first design preference per ADR-MS-003 |
| FHIR R5 + R4 surface | OpenAPI contracts | All counterparts ship R4; oyatie ships R5 ahead of competitors |
| HIPAA-grade audit trail | Cross-cutting | All counterparts; oyatie tamper-evident-sealed |
| Break-glass with mandatory review | Cross-cutting | Epic + Cerner; oyatie review-saga durable |
| Proxy access (caregiver/guardian) | FR-PORT-064 | Epic MyChart + Cerner HealtheLife |

See `competitor-parity-matrix.md` for the full 120-row UNION-coverage matrix.

## Architecture at a glance

EMR is a 15-bounded-context hexagonal stack, one per BC, with the 12-layer enum per ADR-0105 materialized as:

```
oya-emr-<bc>-kernel
  ↑
oya-emr-<bc>-domain
  ↑
oya-emr-<bc>-usecase
  ↑
oya-emr-<bc>-application
  ↑
oya-emr-<bc>-{api, events, grpc}     # adapter trio per ADR-0145
  ↑
oya-emr-rest                          # collated REST server
  ↑
oya-emr-app                           # service binary
```

Adapters for persistence (`oya-emr-<bc>-adapter-postgres`, `oya-emr-vital-adapter-timescale`, `oya-emr-portal-session-adapter-valkey`) and out-of-process peer clients (`oya-emr-adapter-client-<peer>`) implement the kernel ports.

Workers handle BCMA ingestion, vital streaming, results consumption, audit emission, bulk-export, legal-hold workflows.

The full architecture is in `ARCHITECTURE.md`.

## Bounded Contexts

| BC | Crate-stem | Notes |
|---|---|---|
| `patient` | `oya-emr-patient-*` | Demographics + MPI link; merge / unmerge with 30d reversibility |
| `encounter` | `oya-emr-encounter-*` | 8 encounter types; admission → discharge state machine |
| `problem` | `oya-emr-problem-*` | SNOMED CT + ICD-10-CM dual-coded; amendable per CMS |
| `medication` | `oya-emr-medication-*` | RxNorm + NDC; EPCS for Schedule II; PDMP query |
| `allergy` | `oya-emr-allergy-*` | RxNorm + UNII + SNOMED; refute-with-history |
| `vital` | `oya-emr-vital-*` | TimescaleDB hypertable; high-frequency device streaming |
| `note` | `oya-emr-note-*` | CRDT autosave; signed-note immutability; cosign workflow |
| `order` | `oya-emr-order-*` | CPOE for med/lab/imaging/consult/diet/activity/nursing |
| `result` | `oya-emr-result-*` | LOINC-coded; subscribe-model; critical-value acknowledgment |
| `care-team` | `oya-emr-care-team-*` | Effective-date-ranged assignments |
| `order-set` | `oya-emr-order-set-*` | Authored + versioned + A/B-eligible + retire-with-grandfather |
| `documentation` | `oya-emr-documentation-*` | Templates + smart-phrases + dot-phrases |
| `billing-code` | `oya-emr-billing-code-*` | CPT/HCPCS/ICD-10-CM/ICD-10-PCS; physician attest |
| `patient-education` | `oya-emr-patient-education-*` | Multi-language content registry |
| `portal-session` | `oya-emr-portal-session-*` | Mobile-first; passkey; proxy-grant saga |

## Cross-µservice handoffs

EMR consumes from and dispatches to:

```
diagnostics                   ←→  lab + imaging orders dispatch; results return
pharmacy                       ←→  ePrescribing dispatch; MAR events return
emergency                      ←→  ED-to-inpatient handoff
clinical-decision-support      →   CDS Hooks 2.0 at every prescribe + order + view
care-management                ←→  episode-of-care discharge; care-plan update consume
healthcare-integration         ←→  HL7 v2 parsed FHIR ingest; outbound external EHR exchange
cloud-iam                      ←   caregiver + patient + proxy auth
policy-engine                  ←   Cedar evaluation on every PHI access
audit-chain                    →   every state-change + PHI-access event
consent-graph                  ←   patient consent state at FHIR-read time
workflow-engine                ←   durable sagas (proxy-grant, break-glass-review, legal-hold)
cloud-billing                  →   billing-code capture via AsyncAPI
cloud-kms                      ←   per-tenant KEK wrap
cloud-storage                  ←   blob attachments for notes, images, voice-dictations
observability                  →   per-SLO metrics + traces
```

## Tenant classes

| tenant_class | Required cell-cert-level | Retention default | EPCS | Notes |
|---|---|---|---|---|
| b2b-healthcare-provider | hipaa-certified | 7y | yes | US baseline |
| b2b-healthcare-network | hipaa-certified | 7y; many dept 10y | yes | US IDN |
| b2b-academic-medical-center | hipaa-pci-certified / sovereign | 7y; many dept 10y | yes | Sovereign-cell common |
| b2b-community-clinic | hipaa-certified | 7y | yes | Smaller deploy |
| b2b-ambulatory-surgery-center | hipaa-certified | 7y | yes | ASC-specific workflow |
| b2b-telehealth-platform | hipaa-certified | 7y | optional | Edge-deploy common |
| b2b-rural-health-clinic | hipaa-certified | 7y | optional | FQHC overlap |
| b2b-federally-qualified-health-center | hipaa-certified | 7y | optional | HRSA-eligible |
| b2b-skilled-nursing-facility | hipaa-certified | 7y | optional | LTC overlap |
| b2b-home-health-agency | hipaa-certified | 7y | optional | OASIS submission |
| KR-private-hospital | healthcare-sovereign-kr | 10y (의료법 §22) | n/a | HIRA reimbursement codes |
| EU-private-hospital | healthcare-sovereign-eu | per member state | n/a | EU sovereignty |

Other tenant classes (b2c-individual, b2b-non-healthcare) are INELIGIBLE for EMR.

## Compliance packs

- **Required:** `HIPAA-2024`.
- **Recommended:** `SOC2-T2-2024`, `ISO-27001-2022`, `EU-GDPR-2018-baseline`, `KR-PIPA-2023-amendment`, `KR-MEDICAL-LAW-2024`, `FDA-21CFR-PART11-2024`, `FERPA-2024` (for school-based clinics).

A tenant whose `compliance_packs[]` does not include `HIPAA-2024` cannot install the EMR µservice.

## Deployment contexts

EMR supports all 6 oyatie deployment contexts:

- `oyatie-public-cloud` — multi-tenant SaaS on oyatie's hosted infrastructure
- `guest-on-aws` — single-tenant on the tenant's AWS account (BAA-eligible)
- `guest-on-oci` — single-tenant on the tenant's OCI account (BAA-eligible)
- `on-prem` — air-gap-capable; sovereign healthcare networks
- `colo` — on customer-owned hardware in customer-owned data center
- `oyatie-as-cloud-provider` — oyatie sells IaaS to the tenant, hosts EMR

For US tenants, `guest-on-aws` + `on-prem` are typical; for EU, `colo` + `on-prem` dominate; for KR, `on-prem` is mandatory under 의료법 medical-record-residency rules.

## Performance targets

- Chart-open p99 ≤ 800ms
- Order entry p99 ≤ 200ms
- FHIR read p99 ≤ 150ms
- FHIR write p99 ≤ 300ms
- Search p99 ≤ 400ms
- Note save p99 ≤ 250ms
- Per-cell throughput ≥ 50,000 QPS
- Per-cell concurrent clinicians ≥ 25,000
- Per-cell concurrent portal users ≥ 100,000

See `slos/*.openslo.yaml` for canonical SLO definitions.

## Supported OSes (Tier-1)

13 Tier-1 OSes per ADR-0328 §D-17:

`talos-1.9`, `rhel-9`, `oracle-linux-9`, `suse-15-sp6`, `ubuntu-24.04-lts`, `debian-12`, `rocky-9`, `almalinux-9`, `centos-stream-9`, `amazon-linux-2023`, `flatcar-stable`, `photon-os-5.0`, `macos-26-apple-silicon-m5-plus` (dev only).

Arch matrix: `linux/amd64`, `linux/arm64`, `darwin/arm64`, plus Tier-2 `linux/ppc64le`, `linux/s390x`.

## Audit + retention

- Every PHI read/write emits an audit event into `audit-chain`.
- Default retention 7 years (HIPAA-2024).
- KR 의료법 overlay: 10 years for KR tenants.
- Legal hold supersedes retention.
- Audit chain is tamper-evident-sealed (Merkle + Ed25519).

See `iac/*/` for per-context deployment manifests.

## Quick links

- [PRD](./PRD.md) — full product requirements (822 lines)
- [ARCHITECTURE](./ARCHITECTURE.md) — 12-layer mapping + cross-µservice handoffs (612+ lines)
- [manifest.json](./manifest.json) — declarative metadata
- [competitor-parity-matrix](./competitor-parity-matrix.md) — Epic / Cerner / athena UNION coverage
- [decisions/ADR-MS-001](./decisions/ADR-MS-001-bounded-contexts.md) — bounded context decomposition
- [decisions/ADR-MS-002](./decisions/ADR-MS-002-fhir-r5-default.md) — FHIR R5 as default
- [decisions/ADR-MS-003](./decisions/ADR-MS-003-mobile-first-portal.md) — mobile-first patient portal
- [contracts/openapi-emr-v1.yaml](./contracts/openapi-emr-v1.yaml) — REST surface (FHIR R5)
- [contracts/asyncapi-emr-v1.yaml](./contracts/asyncapi-emr-v1.yaml) — clinical events
- [contracts/proto/emr.proto](./contracts/proto/emr.proto) — gRPC inter-µservice
- [policies/](./policies/) — 7 Cedar policies (physician prescribe, nurse document, patient view, break-glass, audit, default-deny, EPCS)
- [slos/](./slos/) — 11 OpenSLO files
- [iac/](./iac/) — 6 deployment contexts
- [implementation-plans/](./implementation-plans/) — IP-001..IP-010
- [supported-oses.json](./supported-oses.json) — full OS matrix

## Owning team + RACI

- **Responsible:** axis-emr
- **Accountable:** council-clinical + council-product
- **Consulted:** council-architecture, council-security, council-legal, council-privacy
- **Informed:** axis-diagnostics, axis-pharmacy, axis-emergency, axis-clinical-decision-support, axis-care-management, axis-healthcare-integration

## Status — Wave 15M-B

EMR µservice scaffold is authored 2026-05-21 as part of Wave 15M-B. The 14 deliverables are present at the substance bar set in `manifest.json#wave_15m_b_substance_floor`. Peer healthcare µservices (`diagnostics`, `pharmacy`, `emergency`, `clinical-decision-support`, `care-management`) author independently in parallel waves.

No commits issued per execution rules — orchestrator handles VCS gating.

## How EMR fits the oyatie portfolio

EMR is one of the **B2B healthcare-vertical product µservices** in the oyatie portfolio. It is unambiguously a product µservice (not a substrate µservice) per ADR-0245 substrate-vs-product layering. EMR depends on substrate µservices for tenant isolation, IAM, KMS, policy evaluation, audit emission, and observability. EMR exposes domain-shaped APIs to peer product µservices (`diagnostics`, `pharmacy`, `emergency`, `clinical-decision-support`, `care-management`, `healthcare-integration`).

The healthcare portfolio decomposition (per ADR-0332 in flight):

```
healthcare-integration       — HL7 v2 + IHE + Carequality / TEFCA QHIN bridges; MPI substrate
emr (this µservice)          — clinical Record-Of-Truth; 15 BCs
diagnostics                  — labs (LIS) + imaging (RIS-lite) order + result lifecycle
pharmacy                     — ePrescribing + MAR; NCPDP SCRIPT wire
emergency                    — ED-specific workflow; ESI triage; door-to-doc tracking
clinical-decision-support    — CDS Hooks 2.0; BPA-equivalent rules engine
care-management              — longitudinal care plan; episode-of-care
genomics (deferred)          — separate µservice if scoped
anesthesia (deferred)        — separate µservice (OQ-1 in PRD)
surgical (deferred)          — separate µservice (Wave 18)
behavioral-health (deferred) — pack overlay or separate µservice (Wave 19)
```

Each is a single-concern flat µservice per ADR-0131 + ADR-0132; no umbrella "Healthcare Suite" wraps them.

## Versioning + release pointers

EMR follows the oyatie per-µservice release pointer pattern (`release/emr/<env>`):

- `release/emr/dev` — continuous deployment from `main`.
- `release/emr/staging` — promoted on SLO-gate green per ADR-0139.
- `release/emr/production` — promoted from staging after burn-in window.

Per ADR-0131, the deprecated tree-wide `staging` + `production` refs do not apply to EMR.

## Contributing

EMR work is gated by:

- ADR-0131 per-microservice flat layout (artifacts under `app/emr/*`).
- ADR-0132 single-concern (no bundling into a "Healthcare Suite").
- Multispectrum review v2.4.0 (architectural, security, regulatory, supply-chain facets mandatory).
- A reviewer-agent APPROVE plus CI green is required before merge per `feedback_self_merge_via_contract_path`.

The owning team (`axis-emr`) RACI-owns every artifact under `app/emr/`. Peer µservice changes that affect EMR's REST or gRPC surface land via cross-µservice IPs.

## License


## Contact

- Primary contact: `axis-emr@oyatie.health`
- On-call escalation: `axis-emr-oncall@oyatie.health`
- Security disclosure: `security@oyatie.health` (per `docs/SECURITY.md` repo-wide)
- Privacy / DSAR queries: per-tenant DPO (forwarded to oyatie `privacy@oyatie.health`)

## FAQ (frequently-asked-questions distilled from tenant onboarding interviews)

### Q: Why not buy Epic-Made-Cheaper-And-Open-Source?

There is no "Epic-Made-Cheaper-And-Open-Source." The closed-source competition (Epic / Cerner) is the market reality. OpenEMR + OpenMRS + LibreHealth exist but lack the depth of Epic-class clinical workflow, the regulatory pack-overlay infrastructure, and the cellular topology that oyatie EMR ships natively. oyatie EMR is positioned to take share at the boundary where regulated-tenants demand vendor neutrality + sovereignty.

### Q: How does oyatie EMR compete with Epic on clinical-UX, where Epic has 25 years of refinement?

Three angles. (1) Mobile-first portal (ADR-MS-003) — Epic's MyChart is a refined web-first product retrofitted to mobile; oyatie EMR is mobile-native. (2) FHIR R5 default + R4 compatibility (ADR-MS-002) — Epic ships R4 default; R5 is a 2025-2026 readiness story still. (3) Tenant-class-aware overlays — Epic licenses Hyperspace + Spring + Community + FQHC variants separately; oyatie EMR ships one canonical base + tenant-class overlay. The clinical-UX gap to Epic in inpatient is the hardest; we close it by pilot-IDN co-design + recruiting senior clinical-informaticists who came from Epic.

### Q: What happens to a tenant if oyatie EMR has a critical bug that affects clinical safety?

(a) Per ADR-0263 + observability, SLO burn-rate alerts catch latency / availability regressions within minutes; (b) per `app/emr/incident-response.md` (referenced runbook to be authored Wave-2), tenant-DPO + tenant-CISO are alerted by automated workflow; (c) per ADR-0251 §D-8 breach-notification machinery, regulator-deadline-aware notification workflow fires if patient-safety material. The Q-3 risk register entries above capture mitigations.

### Q: Does oyatie EMR have a CE Mark / FDA 510(k) / KFDA classification?

Some EHR + CDS combinations require regulatory device classification (e.g., FDA Software-as-a-Medical-Device). oyatie EMR core (chart, demographics, encounter, problem, medication, allergy, vital, note, order, result, billing, care-team, education, portal) is positioned outside the SaMD scope. The clinical-decision-support µservice may, if it ships predictive models, fall under FDA HTI-1 PDSI scope; that ADR is owned by the CDS µservice, not EMR.

### Q: Is there a self-hosted free / community tier?

Not for EMR. EMR's PHI / patient-safety bar makes a casual self-hosted tier irresponsible (unverified deployments + uncertified cells + missing audit chain). For dev + evaluation, oyatie provides a sandbox tenant with synthetic data (no real PHI) under `oyatie-public-cloud` with rate-limited access.

### Q: How long does a typical Epic → oyatie migration take?

3-18 months depending on tenant size. Small-clinic athena → oyatie: 3 months. Community hospital Cerner → oyatie: 6-9 months. Large IDN Epic → oyatie: 12-18 months. The migration window includes dual-write strangler + clinician training + go-live ramp.

### Q: Does oyatie EMR ship a "Hyperspace-killer" clinical UX or just an API?

Both. The REST + gRPC + AsyncAPI surfaces are tenant-API-grade. The clinician UI ships separately (Wave-2+ scope) and is informed by the mobile-first portal ADR plus a clinician-desktop application target for tier-1 hospitals. Tenant-IDNs may also choose to bring their own UI atop the FHIR R5 API surface.

### Q: Can a tenant move EMR data OUT of oyatie?

Yes — that's the structural advantage over Epic. `/fhir/$export` FHIR Bulk Data v2.0 endpoint ships out the full tenant chart in standard NDJSON. C-CDA per encounter is exportable on demand. The tenant-data-portability runbook (Wave-2) walks an outbound migration through ETL + reconciliation.

### Q: What's the typical EMR billing rate (oyatie-as-cloud-provider mode)?


### Q: Is there an audit / compliance report customers can receive?

Yes — per ADR-0251 §D-2, each installed pack drives a regulator-evidence packet that oyatie publishes per the pack's `regulator_evidence_cadence` (HIPAA: annual; SOC2-T2: continuous + annual attestation). Tenants receive their packet on cadence.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0709-general-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
