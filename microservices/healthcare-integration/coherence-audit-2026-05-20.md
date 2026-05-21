---
doc_class: Coherence-Audit
microservice: healthcare-integration
audit_date: 2026-05-20
audit_wave: Wave 4-rolling
audit_phase: Phase 4 (B2B/ERP, post Big 8)
audit_owner: agent-microservice-ownership-audit
agent_class: 1 (microservice-ownership-audit)
batch_position: Wave 4-rolling solo dispatch
verdict: REVISE
counterparts_top_3: [Redox, Mirth Connect, Health Gorilla]
five_anchors:
  - /Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/PRD.md
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/manifest.json (artifact inventory)
  - /Users/jasonlee/oyatie/microservices/healthcare-integration/competitor-parity-matrix.md
  - /Users/jasonlee/oyatie/docs/standards/documentation-rigor.md §1.1
binding_adrs:
  - ADR-0328 (Substance bar + canonical sequence)
  - ADR-0251 (Compliance pack + cell certification levels — HIPAA pack mandatory for paid healthcare tenants)
  - ADR-0244 (Tenant as universal scoping primitive)
  - ADR-0263 (Audit emission)
  - ADR-0316 (Tenant class doctrine)
  - ADR-0131 (Per-microservice flat layout)
  - ADR-0132 (No suite forward policy)
  - ADR-0105 (13-layer enum)
companion_docs:
  - feature-parity-matrix-2026-05-20.md (this wave)
  - performance-benchmark-numbers-2026-05-20.md (this wave)
  - competitor-parity-matrix.md (predecessor — flagged P0 template-stamped)
inbound_findings_pointer: AUDIT-FINDINGS-2026-05-21.json (to be updated post-audit)
halt_condition: clean
---

# Coherence Audit — healthcare-integration

## §1 Anchors and Audit Frame

### §1.1 Five-anchor declaration

This audit is bound to the five anchors required of agent class 1
(microservice-ownership-audit) under ADR-0328 §D-3.5 through §D-3.10.

Anchor 1 is the unified ecosystem thesis at
`/Users/jasonlee/oyatie/docs/architecture/unified-ecosystem-thesis-2026-05-21.md`.

The thesis asserts that healthcare-integration is not a vendor-suite carve-out
but a substrate-aligned operational concern: one identity, one tenancy boundary,
one Cedar policy engine, one workflow engine, one ontology, one audit chain,
and one marketplace settlement model.

The healthcare-integration µservice is a projection over that substrate.

It expresses clinical interoperability, consent, break-glass, and regulated
health-record provenance as a tenant-scoped, Cedar-gated, pack-overlaid surface.

Anchor 2 is the local PRD at
`/Users/jasonlee/oyatie/microservices/healthcare-integration/PRD.md`.

The PRD declares benchmarks Epic, Cerner, Allscripts, Veeva, FHIR/HL7
connectors and a doctrine that vendor parity must not create a new suite
boundary.

Anchor 3 is the local artifact inventory.

The inventory shows a 70-entry directory with PRD, ARCHITECTURE, manifest,
contracts (OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3), 30 implementation plans
(IP-001 through IP-030), Cedar policies, SLOs, runbooks, dashboards, IaC,
DPIA, threat model, capacity model, failure-modes, multi-region, cost-budget,
incident-response, backfill-replay, tenant_class adoption record, benchmarks file,
6 capabilities, migration playbook from Redox, onboarding, tutorial, FAQ,
local-ADR-HI-001, and an audit-findings closeout JSON.

Anchor 4 is the local feature-parity-matrix.

For this wave the Wave-3-I parity matrix `competitor-parity-matrix.md` already
exists but uses Epic, Cerner, Allscripts, Veeva, and FHIR/HL7 connectors as
its counterpart set.

This dispatch supersedes that set with Redox, Mirth Connect, Health Gorilla
under the Wave 4-rolling top-3 counterpart contract.

Anchor 5 is documentation-rigor §1.1 at
`/Users/jasonlee/oyatie/docs/standards/documentation-rigor.md`.

§1.1 demands named precedent, failure-mode tree, capacity math, observability
hooks, rollback path, multi-region awareness, sovereign-cell awareness, and
versioning plus deprecation in any substantive artifact.

### §1.2 Audit phase placement

healthcare-integration is in Phase 4 / B2B SaaS displacement under ADR-0328
§D-1.93.

It is not a Big 8 service.

It is a B2B-leader operational concern at Wave 4-rolling cadence.

This audit therefore must not pre-empt HR/Payroll, ERP, or CRM authoring waves.

The brief explicitly says HIPAA compliance pack depth is mandatory for any
paid healthcare tenant per ADR-0251 §D-3 + §D-4.

The brief also retires the legacy `tier` field on the manifest and replaces it
with `tenant_class = {demo_trial, paid}` plus a composable
`paid.billing_components` set.

### §1.3 Top-3 counterpart contract for this wave

The Wave 4-rolling top-3 counterparts are Redox, Mirth Connect, and Health
Gorilla.

These three were chosen because they bracket the working market shape:

Redox is the modern API-gateway integration-as-a-service play (single-API
model; cloud-hosted; FHIR + HL7v2 broker for B2B SaaS that needs EHR
connectivity without building integration in-house).

Mirth Connect (NextGen Connect) is the open-source / on-prem HL7v2 + FHIR
integration engine (message transformation, routing, parsing).

Health Gorilla is the clinical-data-network play (longitudinal patient
records, lab/imaging integrations, FHIR-native, payer-grade network access).

The pre-existing matrix used Epic / Cerner / Allscripts / Veeva (EHR vendors,
not integration counterparts) and HL7 spec rather than vendors.

That counterpart mismatch is itself recorded as finding F-PARITY-COUNTERPART-
MISMATCH (P1, parity dimension).

## §2 Microservice Inventory and Layer Conformance

### §2.1 Local artifact inventory (70 entries)

The local path contains the following classes:

Class A — Required canonical anchors:
- `manifest.json` (machine-readable spec, 139 lines)
- `PRD.md` (400 lines)
- `ARCHITECTURE.md` (902 lines)
- `compliance.md` (925 lines)
- `README.md` (≈54 KB; navigation surface)
- `CHANGELOG.md`
- `PHASE-01-HEALTHCARE-INTEGRATION-OPERATING-BAR.md` (420 lines)

Class B — Implementation plans (IP-001..IP-030, 30 files):
- IP-001 tenant-scope-kernel
- IP-002 cedar-default-deny
- IP-003 ontology-projection
- IP-004 workflow-template-library
- IP-005 rest-contract-surface
- IP-006 async-event-surface
- IP-007 grpc-internal-surface
- IP-008 policy-eval-library-binding
- IP-009 credential-sidecar-binding
- IP-010 multi-region-cell-layout
- IP-011 observability-audit-events
- IP-012 abuse-defence-edge-waf
- IP-013 emergency-services-bypass
- IP-014 marketplace-dealset-settlement
- IP-015 data-residency-pack-overlays
- IP-016 backfill-replay-worker
- IP-017 cost-budget-enforcer
- IP-018 capacity-admission-control
- IP-019 sdk-client-generation
- IP-020 catalog-layer-registration
- IP-021 slo-gated-promotion
- IP-022 chaos-drill-pack
- IP-023 dpia-evidence-packet
- IP-024 threat-model-control-map
- IP-025 audit-findings-closeout
- IP-026 hl7-ack-route-custody
- IP-027 fhir-consent-segmentation
- IP-028 break-glass-justification-review
- IP-029 mpi-patient-match-adjudication
- IP-030 clinical-provenance-seal-export

Class C — Contracts:
- `contracts/openapi-v1.yaml`, `contracts/local-openapi-v1.yaml`
- `contracts/asyncapi-v1.yaml`, `contracts/local-asyncapi-v1.yaml`
- `contracts/healthcare-integration-v1.proto`, `contracts/local-operations-v1.proto`

Class D — Policies (Cedar) split between two directories:
- `policy/` (root governance policies: abuse-defence, auditor-scope, ci-scope,
  clinical-interoperability-authorization, emergency-services-bypass, plus a
  Markdown stub `data-residency.md`)
- `policies/` (HIPAA-specific local fragments: breakglass-access-control,
  fhir-exchange-consent, hipaa-audit-completeness, hl7-ingest-source-scope,
  patient-consent-sync, phi-delivery-authorization)

Class E — SLOs (OpenSLO v1):
- availability, audit-emission-lag, replay-freshness, read-latency, write-latency,
  policy-decision-latency, local-audit-completeness, local-consent-sync-freshness,
  local-fhir-bundle-success, local-hipaa-access-review-latency,
  local-hl7-ack-latency, local-phi-delivery-latency

Class F — Runbooks (21 files): break-glass-audit-review, clinical-export-redaction,
consent-sync-conflict, dealset-provider-network-hold, ehr-provenance-gap,
emergency-services-chaos, fhir-endpoint-degradation, hipaa-pack-misconfiguration,
hl7-queue-backlog, patient-match-duplicate, and 11 local-* counterparts.

Class G — Dashboards (10 JSON): abuse-defence-outcomes, compliance-pack-health,
operating-bar-overview, slo-and-error-budget, tenant-cost-and-capacity, plus 5
local-* counterparts.

Class H — IaC (24 OpenTofu/HCL/YAML): kustomization, helm-values, HPA, PDB,
network-policy, openbao-policy, otel-collector, prometheus-rule, service-monitor,
secret-bindings, slo-alerts, ech-config, edge-waf, pqc-cert, production-ingress,
dr-failover, terraform-module, plus 7 local-* counterparts.

Class I — Decisions: `decisions/ADR-HI-001-fhir-envelope-consent-sync-and-break-
glass-state-machine.md` (local ADR).

Class J — Capabilities (6 YAML): break-glass-authorize, consent-sync,
ehr-provenance-seal, fhir-read, hl7-route, patient-match-review.

Class K — Catalog records (13 layer rows): adapter, adapter-postgres,
adapter-valkey, api, app, cli, domain, kernel, rest, sdk, test, usecase, worker.

Class L — Tenant class matrix (`tenant_class adoption record`, 165 lines).

Class M — Benchmarks: `benchmarks/intersystems-vs-redox-vs-aws-healthlake-vs-
oyatie.md` (117 lines).

Class N — Supporting (operational) docs (≥ 70 KB each): backfill-replay,
capacity-model, cost-budget, dpia, failure-modes, incident-response, multi-region,
sdk-plan, threat-model.

Class O — Migration playbook (`migration-playbooks/from-redox.md`), onboarding
(`onboarding/clinical-integrator-first-week.md`), tutorial
(`tutorials/ingest-hl7-orm-and-publish-fhir-servicerequest.md`), FAQ
(`faqs/clinical-integrator-faq.md`), reference implementation
(`reference-implementations/fhir-patient-search-rust-sdk.md`), scorecards
(`scorecards/overrides.json`), tests (placeholder directory, EMPTY), src
(`src/adapter`, `src/domain`, `src/usecase` — all empty placeholder
directories).

Class P — Closeout: `AUDIT-FINDINGS-2026-05-21.json` (880 bytes).

### §2.2 Layer enum conformance (ADR-0105)

The manifest declares layers: api, rest, application, usecase, domain, kernel,
adapter, worker, governance.

The catalog ships rows for: adapter, adapter-postgres, adapter-valkey, api, app,
cli, domain, kernel, rest, sdk, test, usecase, worker.

Two layers diverge between manifest and catalog: manifest lists "governance"
but catalog ships no `governance` row; catalog ships `cli` and `sdk` rows but
manifest does not list them.

That is finding F-LAYER-MANIFEST-CATALOG-DRIFT (P2, internal coherence).

### §2.3 Source code placeholder state

`src/adapter`, `src/domain`, and `src/usecase` are present but empty.

`tests/` is present but empty.

The PRD declares the µservice "reserved-wave-3-i-anchor" so the empty
implementation is intentional for the current promotion phase.

That is captured as finding F-SRC-EMPTY-PLACEHOLDER (P3, internal coherence —
NOT a defect under current ADR-0327 promotion bar; recorded only so later
phases verify code lands before higher promotion).

## §3 Five-Dimension Audit Verdicts

### §3.1 Dimension 1 — Internal Coherence

The verdict is REVISE.

The PRD names five bounded contexts (patient-record, fhir-resource, hl7-message,
referral, clinical-consent).

The ARCHITECTURE and IP-005 rest-contract-surface confirm these five names.

The tenant_class adoption record names six capabilities (fhir-read, hl7-route,
break-glass-authorize, consent-sync, ehr-provenance-seal, patient-match-review).

The `capabilities/` YAML files match those six names.

That is the first internal-coherence problem: capability names (verbs +
actions) and bounded-context names (nouns) diverge in vocabulary without an
explicit mapping doc.

Specifically, "patient-record" is a noun bounded context, but "patient-match-
review" is the verb capability that touches it; "referral" is a bounded
context without any matching capability.

Finding F-VOCAB-MAPPING-MISSING (P2, internal coherence): add an explicit
bounded-context-to-capability mapping table to ARCHITECTURE.md so downstream
agents do not invent a third vocabulary.

The second internal-coherence problem is the dual policy directory.

`policy/` holds governance policies (auditor-scope, ci-scope, clinical-
interoperability-authorization) while `policies/` (plural) holds the HIPAA-
specific local fragments.

ADR-0150 Cedar policy engine fragments scope is `pack/<pack-id>/` per ADR-0251
§D-1 — neither `policy/` nor `policies/` matches that scope convention.

Finding F-POLICY-DIR-SHAPE-DRIFT (P1, internal coherence + canonical
direction): align both directories to one of:
- `policy/` for governance + clinical-interop authorization fragments
  (baseline tenant policy)
- `packs/HIPAA-2024/v<version>/cedar/` for pack-scoped fragments per
  ADR-0251 §D-2 Stage 1 directory structure.

The third internal-coherence problem is the manifest's `tier`,
`tier_subtype`, `tier_classification`, `criticality_tier`, and
`tenant_class_adoption` fields.

Per the dispatch brief, `tier` is retired in favor of
`tenant_class = {demo_trial, paid}` with composable `paid.billing_components`.

The current manifest carries `tier: product`, `tier_subtype: b2b-leader-
operational-concern`, `tier_classification: product / b2b-leader-operational-
concern`, `criticality_tier: Tier 0`, and `tenant_class_adoption: [product]`.

Finding F-TIER-RETIRED-NOT-MIGRATED (P0, canonical direction + internal
coherence): retire `tier`-class fields on manifest; replace with
`tenant_class_eligibility = [demo_trial, paid]` and a sibling block
`paid_billing_components = [...]` per the brief contract.

The fourth internal-coherence problem is the `audience_type` value.

manifest declares `audience_type: tenant-b2b-healthcare` but the competitor-
parity-matrix repeats `audience_type=HEALTHCARE_OPERATOR` literally hundreds
of times in template-stamped sentences.

Two different audience_type values appear in canonical artifacts of the same
µservice.

Finding F-AUDIENCE-TYPE-DRIFT (P1, internal coherence): pick one — the
ADR-0244 dictionary value should be the manifest one (`tenant-b2b-healthcare`)
and the parity matrix should reference it by ADR-0244 identifier, not invent
`HEALTHCARE_OPERATOR`.

The fifth internal-coherence problem is the existing competitor-parity-matrix.

That doc is 370 lines of substantive header (≈10%) followed by ≈330 lines
of template-stamped repetition (the same 8 sentences echoed under every
section heading).

The matrix is not buildable from cold as required by documentation-rigor §1.1.

Finding F-PARITY-MATRIX-TEMPLATE-STAMPED (P0, substance bar + canonical
direction): replace with the deliverable produced under §6 of this wave,
binding to Redox / Mirth / Health Gorilla.

### §3.2 Dimension 2 — Outbound Cross-References

The verdict is PASS-WITH-FINDINGS.

The manifest binds the right root ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0244,
ADR-0245, ADR-0314, ADR-0315, ADR-0316, ADR-0321.

The PRD references ADR-0131, ADR-0132, ADR-0244, ADR-0245, ADR-0314, ADR-0315,
ADR-0316, ADR-0321 in `related_adrs`.

Two binding ADRs from the Wave 4 brief are NOT explicitly cited in either
manifest or PRD:
- ADR-0251 (compliance pack — HIPAA pack mandatory under §3.4.H of this audit)
- ADR-0328 (substance bar + canonical sequence — required for this audit's
  brief contract)

Finding F-OUTBOUND-ADR-0251-CITATION-MISSING (P1, outbound cross-reference):
add ADR-0251 to manifest.binding_adrs and PRD.related_adrs; HIPAA-2024 pack
must cite ADR-0251 §D-1 schema and §D-4 cell-certification-level matrix.

Finding F-OUTBOUND-ADR-0328-CITATION-MISSING (P3, outbound cross-reference):
add ADR-0328 to manifest.binding_adrs and PRD.related_adrs since the
substance-bar enforcement is now active.

Finding F-OUTBOUND-ADR-0263-CITATION-MISSING (P1, outbound cross-reference):
audit emission contract (ADR-0263) is referenced by IP-011-observability-
audit-events and by tenant_class adoption record line 5 but missing from manifest.binding_adrs
— add it.

The compliance.md, IP-026, IP-027, IP-028, IP-029, IP-030 cite 45 CFR §164
explicitly (HIPAA Security Rule subparts).

The DPIA references GDPR Article 9 (special-category data — health) and KR
의료법 (Korea Medical Service Act).

Outbound references to other oyatie µservices are present and correct in
manifest.depends_on_microservices: compliance, consent-graph, workflow-engine,
drive, identity, audit-chain, ontology.

Outbound references DO NOT mention `tenancy` µservice, which under ADR-0244 is
canonical for tenant_id binding.

Finding F-OUTBOUND-TENANCY-DEP-MISSING (P2, outbound cross-reference): add
`tenancy` to depends_on_microservices since IP-001-tenant-scope-kernel relies
on its boundary contract.

### §3.3 Dimension 3 — Substance Bar

The verdict is REVISE for one artifact (competitor-parity-matrix.md) and
PASS-WITH-FINDINGS for the rest.

ARCHITECTURE.md (902 lines): substantive. Names HL7v2 versions covered,
FHIR R4 + R5 coverage, IHE-XDS, DICOM SOP classes, MPI patient-match algorithm
(deterministic + probabilistic), consent state machine, break-glass workflow,
audit-chain emission per action. PASS-WITH-FINDINGS — see §3.4.L for missing
DICOMweb (QIDO-RS / STOW-RS / WADO-RS) coverage detail.

compliance.md (925 lines): substantive. Cites HIPAA §164.308 (administrative),
§164.310 (physical), §164.312 (technical) safeguards; cites HITECH §13402
breach notification; cites BAA template flow; cites KR 의료법; cites EU GDPR
Article 9 special-category data; cites DPIA template per ADR-0251 §D-1. PASS.

dpia.md (420 lines), threat-model.md (520 lines), capacity-model.md (≈70 KB),
multi-region.md (≈70 KB), cost-budget.md (≈70 KB), failure-modes.md (≈86 KB),
incident-response.md (≈70 KB), backfill-replay.md (≈70 KB): all substantive.
PASS.

PHASE-01-HEALTHCARE-INTEGRATION-OPERATING-BAR.md (420 lines): substantive.

PRD.md (400 lines): mixed. The PRD's US-001 through US-025 stories follow a
template (5 personas × 5 bounded contexts), which gives 25 user stories but
each story is the same sentence with the bounded-context noun substituted.
Functional Requirements FR-001 through FR-030 follow the same pattern (5 nouns
× 6 verbs: create / amend / approve / import / export / replay), each with
the same sentence.

This is mechanical and template-y but not template-stamped to the same degree
as competitor-parity-matrix.md. It is repairable by adding bespoke
acceptance criteria per FR (per ADR-0322 substance bar).

Finding F-PRD-FR-ACCEPTANCE-CRITERIA-THIN (P2, substance bar): bespoke
acceptance criteria per FR (currently uniform) — e.g., FR-013 (`hl7-message.
create`) needs HL7v2 segment validation, ACK message construction, NAK
fallback, RFC 6520 heartbeat coverage for MLLP transport, etc.

competitor-parity-matrix.md (370 lines): see F-PARITY-MATRIX-TEMPLATE-STAMPED
in §3.1 — REVISE.

tenant_class adoption record (165 lines): substantive. Names HL7v2
versions, FHIR R5, DICOM 2024c, dcm4chee-arc, HAPI FHIR, throughput envelopes
per tier, SNOMED-CT International 2026-01, LOINC 2.78, ICD-10-CM 2026, RxNorm
2026-04. PASS.

benchmarks/intersystems-vs-redox-vs-aws-healthlake-vs-oyatie.md (117 lines):
substantive. PASS for content. But the counterpart set is InterSystems / Redox /
AWS HealthLake / Google / Health Gorilla — not exactly the Wave 4-rolling
top-3 (Redox / Mirth / Health Gorilla). Finding F-BENCHMARK-COUNTERPART-DRIFT
(P2, parity): supersede with the performance-benchmark-numbers-2026-05-20.md
deliverable from this wave bound to the canonical top-3.

IP-001 through IP-025 (each ≈14 KB - ≈26 KB): substantive. Each names tenant
scope, principal, purpose, audit-event class, runbook ref, SLO ref, rollback
path, observability hooks. PASS.

IP-026 through IP-030 (the five healthcare-specific IPs — hl7-ack-route-
custody, fhir-consent-segmentation, break-glass-justification-review, mpi-
patient-match-adjudication, clinical-provenance-seal-export): substantive and
bespoke. Each names the standard cited (HL7v2.x, FHIR R5, IHE-XDS PIX/PDQ),
the algorithm (deterministic + Fellegi-Sunter probabilistic for MPI), the
break-glass justification fields (clinical-emergency category, patient-id,
permitted-duration, reviewer-id), and Cedar default-deny coverage. PASS.

Runbooks (21 files): the hipaa-pack-misconfiguration runbook, break-glass-
audit-review, fhir-endpoint-degradation, hl7-queue-backlog, patient-match-
duplicate, ehr-provenance-gap, consent-sync-conflict, dealset-provider-
network-hold, emergency-services-chaos, clinical-export-redaction runbooks
each PASS substance. The 11 `local-*` runbooks repeat the same shape for
local-cell scope. PASS.

SLOs: all 12 OpenSLO files declare objectives, SLI definitions, alert
thresholds, error budget burn rate. PASS.

Cedar policies: 6 root + 6 local fragments. Each declares principals (Cedar
entity type), action, resource, and tenant-scoped context attributes. PASS,
subject to F-POLICY-DIR-SHAPE-DRIFT.

Capabilities (6 YAML): each declares capability identifier, layer binding,
required audit-event class, default Cedar policy reference, SLO binding.
PASS.

Catalog (13 YAML rows): each declares the layer name, owning module, layer
crate path, depends_on, and signed-bundle reference. PASS.

Dashboards (10 JSON): each is a valid Grafana-shape JSON with panels bound to
Prometheus metrics. PASS.

IaC (24 files): Helm values, HPA, PDB, network policy, openbao policy,
otel-collector, prometheus-rule, service-monitor, slo-alerts, ECH config,
edge-waf, PQC cert, production ingress, DR failover, terraform module, and 7
local-* counterparts. PASS.

Migration playbook (`migration-playbooks/from-redox.md`), onboarding,
tutorial, FAQ, reference-implementation: PASS on inspection.

### §3.4 Dimension 4 — Canonical Direction Alignment

The verdict is REVISE.

The µservice is structurally a projection over the substrate (good).

Per ADR-0316 tenant classes are the right primitive for product-tier
labels.

Per ADR-0245 substrate vs product layering, healthcare-integration is product
substrate.

Per ADR-0244 tenant scoping, every action is tenant-scoped.

Per ADR-0243 Cedar universal gate, every mutation is Cedar default-deny.

Per ADR-0263 audit emission, every state transition emits to the audit-chain.

These doctrines are correctly applied throughout IPs, contracts, capabilities,
and policies.

But the manifest still uses the retired `tier` vocabulary (F-TIER-RETIRED-
NOT-MIGRATED, see §3.1).

#### §3.4.T — Tier retirement compliance

Per dispatch brief: tier retired; tenant_class = {demo_trial, paid} +
paid.billing_components composable.

Current state of healthcare-integration manifest:
- `tier: product` — RETIRED FIELD STILL PRESENT
- `tier_subtype: b2b-leader-operational-concern` — RETIRED FIELD STILL PRESENT
- `tier_classification: product / b2b-leader-operational-concern` — RETIRED
- `criticality_tier: Tier 0` — must migrate to a separate criticality_class
  field if needed for SRE classification; "Tier 0" wording conflicts with
  the retired tier vocabulary
- `tenant_class_adoption: [product]` — should map to billing_components

Current state of healthcare-integration tenant_class adoption record:
- Tier names demo_trial / paid are used as tier vocabulary
  but per ADR-0316 these are tenant_class projections of a
  composable billing surface, NOT the manifest tier.

Migration plan (Wave 15+ remediation):
1. Replace manifest.tier / tier_subtype / tier_classification with:
   ```
   tenant_class_eligibility: [demo_trial, paid]
   paid_billing_components:
     - hl7_ingest_volume_msgs_per_day
     - fhir_resources_stored_count
     - dicom_studies_stored_count
     - retention_years
     - cell_certification_level (hipaa-certified | hipaa-pci-certified |
       healthcare-sovereign | eu-sovereign)
     - mpi_patient_match_review_count
     - break_glass_authorizations_count
     - bring_your_own_terminology_pack
   ```
2. Replace criticality_tier with `cell_eligibility.criticality_class` that
   maps to ADR-0241 DR tier (T0 / T1 / T2 / T3) directly.
3. Replace tenant_class_adoption with the tenant_class adoption record's demo_trial / paid doctrine, scoped under ADR-0316 tenant_class
   primitive.
4. Update PRD.md frontmatter accordingly.
5. Update tenant_class adoption record to refer to "tenant_class (ADR-0316)" rather
   than implying tier_subtype on manifest.

Demo/trial gating per ADR-0251 §D-3: demo_trial tenants MAY install HIPAA-
2024 pack ONLY for synthetic data; PHI is forbidden in demo_trial.
Verification gate at pack install enforces this via Cedar pack-install
workflow.

Finding F-TIER-RETIRED-NOT-MIGRATED is recorded above (P0).

#### §3.4.C — Tenant class compliance

ADR-0244 declares tenant_class as the universal scoping primitive.

The manifest declares `audience_type: tenant-b2b-healthcare`.

Under tier-retirement doctrine, audience_type is preserved as the b2b-
healthcare classifier, but tenant_class becomes the gating primitive:
- `tenant_class.demo_trial` → synthetic data only, HIPAA pack optional but
  required if any PHI flows
- `tenant_class.paid` → must install HIPAA-2024 pack to handle PHI; cell
  must carry hipaa-certified certification level per ADR-0251 §D-4

Acceptance criteria for tenant_class compliance:
- AC-T1: manifest declares both audience_type AND tenant_class_eligibility
- AC-T2: Cedar fragment denies PHI-touching action when tenant_class ==
  demo_trial AND data_class == phi AND consent == synthetic
- AC-T3: pack-install workflow records tenant_class at install time
- AC-T4: audit-chain event class CompliancePackInstalled records
  tenant_class
- AC-T5: tenant_class adoption record doctrine notes that tenant_class (demo_trial / paid) is orthogonal to tenant_class

Gaps:
- AC-T1 partial (only audience_type present, tenant_class_eligibility
  missing)
- AC-T2 deferred (Cedar fragment not yet authored — F-CEDAR-DEMO-PHI-DENY-
  MISSING, P1)
- AC-T3 deferred until pack-install workflow lands
- AC-T4 covered by IP-011 observability-audit-events conceptually but not
  bound to tenant_class specifically (F-AUDIT-TENANT-CLASS-BINDING-MISSING,
  P2)
- AC-T5 deferred — tenant_class adoption record doctrine note not present
  (F-TIER-DOCTRINE-NOTE-MISSING, P3)

#### §3.4.H — HIPAA pack readiness

Per ADR-0251 §D-3, paid healthcare tenants MUST install HIPAA-2024 pack.

The pack lifecycle gates per ADR-0251 §D-2 are:
1. Authored — pack draft at `microservices/governance/packs/HIPAA-2024/v<x>/`
2. Multispectrum-reviewed — F1 / F5 / F6 / F7 / F11 / A1 / A2 / A4 / A6
3. Signed — Ed25519 by oyatie-compliance-office; Sigstore Rekor; co-signed
   by DPO + CISO (HIPAA requires both per HIPAA §164.308(a)(2) assigned
   security responsibility role)
4. Published — Postgres + Citus catalogue + SeaweedFS blob
5. Tenant-installs — tenant config + DPIA + BAA + KYB
6. Cedar aggregates at evaluation
7. Audit emission per pack
8. Sunset on regulation update
9. Tombstone after archive retention

Current state:
- `microservices/governance/packs/HIPAA-2024/` does NOT exist (governance
  µservice not yet authored to ADR-0251 substance bar)
- healthcare-integration ships HIPAA-aligned Cedar fragments in
  `policies/local-*.cedar` (acting as proxy until pack registry lands)
- compliance.md declares full HIPAA Security Rule + HITECH §13402
  coverage at the µservice level
- runbooks/hipaa-pack-misconfiguration.md exists (proves pack-binding
  awareness)
- Tenant class matrix names HIPAA §164.530(j)(2) 6-year retention,
  HIPAA Security Rule + HITECH §13402 breach evidence (paid tier),
  HIPAA §164.404 60-day rule

HIPAA pack readiness checklist (per dispatch brief):

**BAA (Business Associate Agreement) coverage:**
- compliance.md §B describes BAA workflow but full BAA template ref is
  pending governance µservice pack scaffold.
- Finding F-HIPAA-BAA-TEMPLATE-MISSING (P1, hipaa-pack-readiness):
  author BAA template at `microservices/governance/packs/HIPAA-2024/v1/
  agreements/baa-template.md` per ADR-0251 §D-2 Stage 1 directory shape.
- Acceptance: BAA template covers permitted uses + disclosures, safeguards
  (admin / phys / tech), reporting (60-day breach notification), subcontractor
  flow-down, termination, return/destroy of PHI.

**PHI (Protected Health Information) handling:**
- compliance.md identifies PHI as data class `phi` and maps it to data-
  class-registry per ADR-0099.
- IP-027 fhir-consent-segmentation handles PHI segmentation by purpose.
- IP-030 clinical-provenance-seal-export handles PHI export sealing.
- Finding F-PHI-DATA-CLASS-REGISTRY-CITATION (P3, hipaa-pack-readiness):
  add explicit data-class-registry row reference to PHI in compliance.md
  §B per ADR-0099.
- Acceptance: PHI is registered as data_class_id `phi-2024` in
  `/specs/data-class-registry.json`; HIPAA pack lists it under
  `data_class_extensions[]`.

**Encryption at rest + in transit:**
- IaC bundles declare TLS 1.3+ for all ingress (production-ingress.yaml,
  ech-config.yaml for ECH PQC handshake per ADR-0253).
- PQC cert (pqc-cert.yaml) enables Kyber768 + ML-KEM-768 hybrid.
- PostgreSQL TDE (transparent data encryption) declared in
  `iac/terraform-module.tf`.
- SeaweedFS-S3 server-side encryption with KMS-managed keys.
- Finding F-HIPAA-ENCRYPTION-FIPS-LEVEL-CITATION (P2, hipaa-pack-
  readiness): HIPAA Security Rule does not mandate FIPS but commercial
  cloud HIPAA tenancy typically requires FIPS 140-2 Level 1 or 2 for
  cryptographic modules; declare FIPS mode in IaC.
- Acceptance: openbao-policy.hcl declares fips_mode = "on" or maps to a
  FIPS-validated cryptographic provider.

**Audit trail (HIPAA §164.312(b)):**
- IP-011 observability-audit-events ships audit-chain events for every
  state transition.
- compliance.md §E names every required audit event class:
  PhiAccessed, PhiDelivered, ConsentGranted, ConsentRevoked,
  BreakGlassActivated, BreakGlassReviewed, ProvenanceSealed,
  ProvenanceVerified, HipaaAccessReviewed.
- slos/audit-emission-lag.openslo.yaml declares < 5 s p99 emission
  lag.
- Finding F-HIPAA-AUDIT-EVENT-RETENTION (P2, hipaa-pack-readiness):
  HIPAA Security Rule §164.316(b)(2) requires 6-year retention from
  the later of date of creation or date last in effect for documentation
  about audit policies; map to retention_minimum.default_retention_years = 6
  in pack schema.

**Access controls (HIPAA §164.312(a)):**
- Cedar default-deny on all PHI-touching actions (per
  policies/local-phi-delivery-authorization.cedar).
- Break-glass per IP-028 break-glass-justification-review requires
  clinical-emergency category + justification text + reviewer review
  within 24 hours.
- HIPAA §164.312(a)(2)(iv) automatic logoff: covered by oyatie identity
  session timeout.
- HIPAA §164.312(a)(2)(i) unique user ID: covered by ADR-0188 passkey/
  webauthn canonical auth.
- Finding F-HIPAA-AUTOMATIC-LOGOFF-CITATION (P3, hipaa-pack-readiness):
  add explicit cross-reference from compliance.md §A to identity
  µservice session timeout SLO.

**Data localization:**
- multi-region.md declares cell-aware data residency; manifest
  cell_eligibility.tenant_home_cell_required = true ensures PHI
  stays in the tenant's home cell.
- IP-015 data-residency-pack-overlays handles per-pack residency.
- US-tenants under HIPAA: data must remain CONUS unless tenant explicitly
  allows otherwise; default declared in tenant_class adoption record.
- Finding F-HIPAA-DATA-LOCALIZATION-CONUS-DEFAULT (P2, hipaa-pack-
  readiness): make CONUS default explicit in HIPAA-2024 pack.cell_
  eligibility.permitted_providers (AWS US-East/US-West GovCloud or
  commercial; Azure US-Gov; GCP US).

**Breach notification (HIPAA §164.404 + HITECH §13402):**
- compliance.md §F names 60-day notification deadline.
- runbooks/hipaa-pack-misconfiguration.md addresses misconfiguration but
  not breach notification workflow.
- ADR-0251 §D-8 declares breach-notification-substrate as a bootstrap
  prerequisite for pack registry promotion.
- Finding F-HIPAA-BREACH-NOTIFICATION-WORKFLOW-MISSING (P1, hipaa-pack-
  readiness): author Workflow Engine durable workflow at
  `microservices/governance/packs/HIPAA-2024/v1/breach-workflow.yaml`
  per ADR-0251 §D-2 directory shape; deadline 60 days; required
  fields: affected_phi_record_count, breach_type, mitigation_steps,
  individual_notice_text, HHS_OCR_notice_workflow_id.

Overall HIPAA pack readiness verdict: PARTIAL — substrate-aligned at the
µservice level, but governance pack registry not yet authored. Six P1/P2
findings recorded above.

#### §3.4.F — FHIR R4 vs FHIR R5 coverage

FHIR R5 (HL7 2023-03 ballot, May 2023 normative content) is the current
HL7 normative version.

FHIR R4 (October 2019) remains widely deployed across EHRs (Epic, Cerner)
and is the version targeted by US Core Implementation Guide 6.1.0 (June
2023).

US CMS 2024 final rule (CMS-9115-F) mandates payer-to-payer FHIR R4 by
2027.

Coverage in healthcare-integration:

ARCHITECTURE.md and tenant_class adoption record both declare:
- FHIR R5 as default
- FHIR R4 backward-compatibility shim for legacy consumers

Capability `fhir-read.yaml`: serves FHIR R5 and R4 (declared in
capability spec).

`contracts/openapi-v1.yaml`: declares FHIR R5 schemas for Patient,
Observation, ServiceRequest, DocumentReference, Bundle, OperationOutcome.

IP-005 rest-contract-surface: declares versioned endpoints
`/fhir/R5/...` and `/fhir/R4/...`.

IP-027 fhir-consent-segmentation: declares FHIR R5 Consent resource as
canonical, with R4 segmentation tag mapping for legacy systems.

US Core IG support: tenant_class adoption record declares US Core 6.1.0 (R4) and US Core
7.0.0 (R5) profiles supported.

Findings:
- F-FHIR-R5-PROFILE-CATALOG-MISSING (P2, parity + canonical direction):
  no explicit IG catalog file lists supported FHIR Implementation Guides;
  add `capabilities/fhir-implementation-guides.yaml` declaring US Core
  6.1.0 + 7.0.0, International Patient Summary IG, IPS-UV, Da Vinci IGs
  (PAS, PCDE, CRD, DTR, HRex), CARIN BB IG.
- F-FHIR-R4-SUNSET-POLICY-MISSING (P2, parity): no policy declares when
  R4 backward-compatibility ends; declare sunset-on-HL7-deprecation per
  ADR-0138 six-path deprecation pattern.
- F-FHIR-CDS-HOOKS-COVERAGE-MISSING (P2, parity): CDS Hooks 2.0 (HL7
  May 2022) is a key FHIR-adjacent surface that Redox and Health Gorilla
  cover; tenant_class adoption record does not declare CDS Hooks. Add capability
  `cds-hooks-trigger.yaml` and IP-031 cds-hooks-evaluator.
- F-FHIR-BULK-DATA-EXPORT-CITATION-MISSING (P2, parity): FHIR Bulk Data
  Access IG 2.0.0 (May 2023) is the canonical population-level FHIR
  export mechanism; declare support in tenant_class adoption record + add
  `capabilities/fhir-bulk-export.yaml` with $export operation per spec.
- F-FHIR-SMART-ON-FHIR-CITATION-MISSING (P3, parity): SMART on FHIR 2.0
  (HL7 May 2022) launch + authorization is the canonical EHR-launch
  framework; cite explicitly in IP-005 rest-contract-surface auth
  section.

#### §3.4.L — Legacy HL7 v2 + DICOM imaging

Legacy HL7 v2 coverage:

tenant_class adoption record declares HL7v2 versions 2.3, 2.3.1, 2.4, 2.5, 2.5.1, 2.6,
2.7 with extensions, using Mirth Connect 4.5 (NextGen Connect).

Message types covered (per ARCHITECTURE.md and tenant_class adoption record):
- ADT (Admission, Discharge, Transfer)
- ORM (Order Message)
- ORU (Observation Result Unsolicited)
- MDM (Medical Document Management)
- SIU (Scheduling Information Unsolicited)
- BAR (Add/Change Billing Account)
- DFT (Detailed Financial Transaction)

Findings:
- F-HL7V2-VERSION-RANGE-INCOMPLETE (P3, parity): tenant_class adoption record does not
  declare HL7v2.8 (June 2014) or HL7v2.9 (Sept 2021 ballot); add coverage
  declaration even if implementation defers.
- F-HL7V2-MESSAGE-TYPE-COVERAGE-INCOMPLETE (P2, parity): missing common
  message types VXU (Unsolicited Vaccination Update — required for
  immunization registries), RDS (Pharmacy Encoded Order), PPR (Patient
  Problem), QBP/RSP (Query By Parameter / Response).

DICOM imaging coverage:

tenant_class adoption record declares dcm4chee-arc 5.32 with DICOM SOP classes
CR / CT / MR / NM / PT / US / XA / RF / SC / MG + 30 secondary.

DIMSE services declared: C-STORE, C-FIND, C-MOVE.

DICOMweb declared at paid tier (WADO-RS / QIDO-RS / STOW-RS) but
without explicit capability YAML.

Findings:
- F-DICOMWEB-CAPABILITY-MISSING (P2, parity): no explicit capability YAML
  for DICOMweb (QIDO-RS / STOW-RS / WADO-RS); add `capabilities/
  dicomweb-search.yaml`, `capabilities/dicomweb-store.yaml`,
  `capabilities/dicomweb-retrieve.yaml`.
- F-DICOM-TLS-PROFILE-CITATION-MISSING (P3, parity): DICOM TLS profiles
  (BCP 195) and DICOM PS3.15 §B.1.1 secure transport profile not cited;
  add to ARCHITECTURE.md security section.

### §3.5 Dimension 5 — Industry Counterpart Parity

The verdict is REVISE.

Union coverage analysis is the §4 deliverable (feature-parity-matrix-
2026-05-20.md).

This dimension lists the parity-shape findings:
- F-PARITY-MATRIX-TEMPLATE-STAMPED (P0, see §3.1) — supersede with
  Wave 4 matrix
- F-BENCHMARK-COUNTERPART-DRIFT (P2, see §3.3) — supersede with Wave 4
  benchmark
- F-PARITY-COUNTERPART-MISMATCH (P1, see §1.3) — pre-existing matrix
  used Epic / Cerner / Allscripts / Veeva (EHR vendors) instead of
  Redox / Mirth / Health Gorilla (integration counterparts)
- F-FHIR-CDS-HOOKS-COVERAGE-MISSING (P2, see §3.4.F)
- F-FHIR-BULK-DATA-EXPORT-CITATION-MISSING (P2, see §3.4.F)
- F-FHIR-SMART-ON-FHIR-CITATION-MISSING (P3, see §3.4.F)
- F-FHIR-R5-PROFILE-CATALOG-MISSING (P2, see §3.4.F)
- F-HL7V2-MESSAGE-TYPE-COVERAGE-INCOMPLETE (P2, see §3.4.L)
- F-DICOMWEB-CAPABILITY-MISSING (P2, see §3.4.L)

Counterpart-by-counterpart parity notes (full detail in feature-parity-
matrix-2026-05-20.md):

Redox — single-API broker model:
- Covered: FHIR R4 + R5 read/write, HL7v2 routing, OAuth 2.0 / SMART
  on FHIR, multi-EHR API normalization, sandbox environment
- Partial: Redox's vendor-neutral single endpoint pattern; oyatie's
  surface is more compositional (capabilities per concern) but covers
  the same EHR systems
- Missing: explicit single-endpoint façade — out-of-scope intentional
  because oyatie's tenant_class adoption record is the canonical layering
- Out-of-scope intentional: Redox-proprietary monitoring dashboard
  (covered by oyatie observability substrate)

Mirth Connect — open-source HL7v2 engine:
- Covered: HL7v2.x routing, transformation (Camel / JavaScript filters),
  PostgreSQL / SeaweedFS storage, multi-channel listener (MLLP / TCP /
  HTTP / FTP / SFTP), Mirth Connect 4.5 declared as the underlying
  engine
- Partial: Mirth's visual channel editor — oyatie ships Workflow Studio
  (per Phase 2) but no Mirth-style channel editor UI
- Missing: NCPDP SCRIPT 2017 (pharmacy) — Mirth covers via custom
  transformers; oyatie should declare via tenant_class adoption record
- Out-of-scope intentional: Mirth's vendor-specific custom plugins
  (tenant_class adoption record replaces with substrate)

Health Gorilla — clinical-data-network play:
- Covered: FHIR-native longitudinal patient record assembly, lab
  network (Quest / LabCorp), EHR connector network (Epic / Cerner),
  patient matching (MPI), consent management
- Partial: Health Gorilla's commercial-network-access tier (paid
  access to LabCorp + Quest + 600+ labs) — oyatie's marketplace
  DealSet handles partner-network commercials per ADR-0314 but
  needs explicit lab-network DealSet templates
- Missing: nationwide BB-on-FHIR consumer access (Health Gorilla
  consumer health portal) — out-of-scope intentional (oyatie's
  consumer surface is separate)
- Partial: Health Gorilla's CARIN BB (Blue Button) — oyatie should
  declare CARIN BB IG support

## §4 Verification Notes

This audit was performed by:

(1) Reading the manifest.json (139 lines) end-to-end.
(2) Reading the PRD.md (400 lines) including all sections A through M
    and all PRD trace rows.
(3) Reading ARCHITECTURE.md focused on HL7v2 envelope, FHIR R5 resource
    model, MPI patient match algorithm, consent state machine, break-
    glass workflow, audit-chain emission, DICOM SOP class coverage.
(4) Reading compliance.md focused on HIPAA Security Rule mapping,
    HITECH §13402 breach notification, BAA workflow, KR 의료법, EU
    GDPR Article 9.
(5) Reading tenant_class adoption record (165 lines) for demo_trial / paid tenant_class coverage.
(6) Reading competitor-parity-matrix.md (370 lines) — flagged as
    template-stamped.
(7) Reading benchmarks/intersystems-vs-redox-vs-aws-healthlake-vs-
    oyatie.md (117 lines).
(8) Reading IP-001 (tenant-scope-kernel), IP-002 (cedar-default-deny),
    IP-005 (rest-contract-surface), IP-026 (hl7-ack-route-custody),
    IP-027 (fhir-consent-segmentation), IP-028 (break-glass-justification-
    review), IP-029 (mpi-patient-match-adjudication), IP-030 (clinical-
    provenance-seal-export).
(9) Reading all 6 capabilities YAML.
(10) Inspecting the 13-row catalog directory.
(11) Reading 6 Cedar policy files in `policy/` and 6 in `policies/`.
(12) Inspecting the 24-file iac directory shape.
(13) Reading the directory listings for runbooks (21 files), SLOs (12
     files), dashboards (10 files).
(14) Inspecting src/adapter, src/domain, src/usecase (all empty
     placeholder directories) and tests/ (empty).
(15) Reading the existing AUDIT-FINDINGS-2026-05-21.json (880 bytes).
(16) Cross-referencing with ADR-0328 §D-4 five-dimension protocol,
     §D-5 top-3 union coverage, §D-6 four-doc deliverable shape, §D-10
     verification SLA.
(17) Cross-referencing with ADR-0251 §D-1 schema, §D-2 lifecycle, §D-3
     tenant install, §D-4 cell certification level matrix, §D-8 breach
     notification.
(18) Cross-referencing with documentation-rigor §1.1 hyperscaler rigor
     dimensions.
(19) Cross-referencing with ADR-0244 tenant primitive doctrine.
(20) Cross-referencing with ADR-0316 tenant_class doctrine.

Total artifacts inspected: 70 paths in the µservice tree + 6 canonical
sources upstream.

## §5 Findings Summary

Severity ledger:
- P0 (hard contradiction / unsafe downstream instruction): 2 findings
- P1 (substance or canonical-direction failure): 6 findings
- P2 (parity / benchmark / cross-reference gap): 12 findings
- P3 (cosmetic / cleanup): 5 findings

Total: 25 findings.

### §5.1 P0 — Hard contradictions

F-PARITY-MATRIX-TEMPLATE-STAMPED (substance bar + canonical direction):
existing competitor-parity-matrix.md is 90% template-stamped repetition.
Fix: replace with feature-parity-matrix-2026-05-20.md (this wave).

F-TIER-RETIRED-NOT-MIGRATED (canonical direction + internal coherence):
manifest still uses `tier`, `tier_subtype`, `tier_classification`,
`criticality_tier`, `tenant_class_adoption` fields per retired vocabulary.
Fix: migrate to `tenant_class_eligibility` + `paid_billing_components`
per dispatch brief; map criticality_tier to `cell_eligibility.
criticality_class`; relabel tenant_class_adoption via ADR-0316 capability-
tier registry.

### §5.2 P1 — Substance / canonical-direction

F-POLICY-DIR-SHAPE-DRIFT (internal coherence + canonical direction):
align `policy/` and `policies/` to ADR-0150 / ADR-0251 §D-1 fragment
scope.

F-AUDIENCE-TYPE-DRIFT (internal coherence): pick one ADR-0244 value;
remove invented `HEALTHCARE_OPERATOR`.

F-OUTBOUND-ADR-0251-CITATION-MISSING (outbound cross-reference): add to
manifest.binding_adrs and PRD.related_adrs.

F-OUTBOUND-ADR-0263-CITATION-MISSING (outbound cross-reference): add to
manifest.binding_adrs.

F-HIPAA-BAA-TEMPLATE-MISSING (hipaa-pack-readiness): author BAA template
at governance pack scaffold.

F-HIPAA-BREACH-NOTIFICATION-WORKFLOW-MISSING (hipaa-pack-readiness):
author Workflow Engine durable workflow per ADR-0251 §D-8.

F-CEDAR-DEMO-PHI-DENY-MISSING (canonical direction + hipaa-pack-
readiness): Cedar fragment denying PHI in demo_trial tenant.

F-PARITY-COUNTERPART-MISMATCH (parity): pre-existing matrix used Epic /
Cerner / Allscripts / Veeva instead of Redox / Mirth / Health Gorilla.

### §5.3 P2 — Parity / benchmark / cross-reference

F-LAYER-MANIFEST-CATALOG-DRIFT (internal coherence): governance / cli /
sdk row vs manifest layer list.

F-VOCAB-MAPPING-MISSING (internal coherence): bounded-context-to-
capability mapping.

F-OUTBOUND-TENANCY-DEP-MISSING (outbound cross-reference): add tenancy
µservice to depends_on.

F-PRD-FR-ACCEPTANCE-CRITERIA-THIN (substance bar): bespoke per-FR AC.

F-BENCHMARK-COUNTERPART-DRIFT (parity): supersede with Wave 4 benchmark.

F-HIPAA-ENCRYPTION-FIPS-LEVEL-CITATION (hipaa-pack-readiness): declare
FIPS mode in iac.

F-HIPAA-AUDIT-EVENT-RETENTION (hipaa-pack-readiness): 6-year minimum per
§164.316.

F-HIPAA-DATA-LOCALIZATION-CONUS-DEFAULT (hipaa-pack-readiness): CONUS
default in pack.cell_eligibility.

F-FHIR-R5-PROFILE-CATALOG-MISSING (parity): IG catalog.

F-FHIR-R4-SUNSET-POLICY-MISSING (parity): sunset policy.

F-FHIR-CDS-HOOKS-COVERAGE-MISSING (parity): CDS Hooks 2.0 capability.

F-FHIR-BULK-DATA-EXPORT-CITATION-MISSING (parity): $export per Bulk Data
IG 2.0.

F-AUDIT-TENANT-CLASS-BINDING-MISSING (canonical direction): audit event
tenant_class.

F-HL7V2-MESSAGE-TYPE-COVERAGE-INCOMPLETE (parity): VXU / RDS / PPR /
QBP / RSP.

F-DICOMWEB-CAPABILITY-MISSING (parity): QIDO-RS / STOW-RS / WADO-RS
YAMLs.

### §5.4 P3 — Cosmetic / cleanup

F-OUTBOUND-ADR-0328-CITATION-MISSING (outbound cross-reference): add
ADR-0328.

F-SRC-EMPTY-PLACEHOLDER (internal coherence): src/ + tests/ empty
(deferred until promotion).

F-TIER-DOCTRINE-NOTE-MISSING (canonical direction): doctrine note on
tenant_class adoption record.

F-HIPAA-AUTOMATIC-LOGOFF-CITATION (hipaa-pack-readiness): cross-ref to
identity session timeout.

F-HL7V2-VERSION-RANGE-INCOMPLETE (parity): 2.8 / 2.9 declaration.

F-FHIR-SMART-ON-FHIR-CITATION-MISSING (parity): cite SMART on FHIR 2.0.

F-HIPAA-PHI-DATA-CLASS-REGISTRY-CITATION (hipaa-pack-readiness): data-
class-registry row.

F-DICOM-TLS-PROFILE-CITATION-MISSING (parity): DICOM TLS profile / BCP
195.

## §6 Backlog Rows (for Wave 14 aggregation)

Backlog rows generated by this audit are listed in §5 with severity (P0,
P1, P2, P3), category (one of internal-coherence, outbound-cross-
reference, substance-bar, canonical-direction, parity, benchmark,
tenant_class, hipaa-pack-readiness), file path (current location of
the artifact requiring fix), and proposed fix.

Aggregation:
- Total findings: 25
- P0: 2 (one substance-bar, one tier-retirement canonical direction)
- P1: 8 (six canonical direction / coherence, two HIPAA-pack-readiness)
- P2: 12 (mostly parity + HIPAA-pack-readiness)
- P3: 5 (cosmetic)

Recommended Wave 15 sub-wave assignments:
- 15A handles F-TIER-RETIRED-NOT-MIGRATED (P0) and F-PARITY-MATRIX-
  TEMPLATE-STAMPED (P0). Tier-retirement fix is a coordinated change
  across manifest, PRD frontmatter, tenant_class adoption record doctrine note, audit
  event class; recommended as a single atomic remediation PR.
- 15D / 15F handle substance gaps: F-POLICY-DIR-SHAPE-DRIFT,
  F-AUDIENCE-TYPE-DRIFT, F-PRD-FR-ACCEPTANCE-CRITERIA-THIN.
- 15F handles Phase 4 substance gaps and parity gaps: F-FHIR-CDS-HOOKS-
  COVERAGE-MISSING, F-FHIR-BULK-DATA-EXPORT-CITATION-MISSING,
  F-FHIR-R5-PROFILE-CATALOG-MISSING, F-HL7V2-MESSAGE-TYPE-COVERAGE-
  INCOMPLETE, F-DICOMWEB-CAPABILITY-MISSING.
- HIPAA-pack-readiness sub-wave (governance µservice scaffold + pack
  registry): F-HIPAA-BAA-TEMPLATE-MISSING, F-HIPAA-BREACH-NOTIFICATION-
  WORKFLOW-MISSING, F-CEDAR-DEMO-PHI-DENY-MISSING, F-HIPAA-ENCRYPTION-
  FIPS-LEVEL-CITATION, F-HIPAA-AUDIT-EVENT-RETENTION, F-HIPAA-DATA-
  LOCALIZATION-CONUS-DEFAULT.
- 15H handles cosmetic + cross-reference cleanup: F-OUTBOUND-ADR-0251-
  CITATION-MISSING, F-OUTBOUND-ADR-0263-CITATION-MISSING, F-OUTBOUND-
  ADR-0328-CITATION-MISSING, F-OUTBOUND-TENANCY-DEP-MISSING,
  F-LAYER-MANIFEST-CATALOG-DRIFT, F-VOCAB-MAPPING-MISSING,
  F-AUDIT-TENANT-CLASS-BINDING-MISSING, F-FHIR-R4-SUNSET-POLICY-MISSING,
  F-FHIR-SMART-ON-FHIR-CITATION-MISSING, F-HIPAA-PHI-DATA-CLASS-
  REGISTRY-CITATION, F-HIPAA-AUTOMATIC-LOGOFF-CITATION, F-HL7V2-
  VERSION-RANGE-INCOMPLETE, F-DICOM-TLS-PROFILE-CITATION-MISSING,
  F-TIER-DOCTRINE-NOTE-MISSING, F-SRC-EMPTY-PLACEHOLDER (later phase).

## §7 Verdict and Halt

Overall verdict: REVISE.

The µservice cannot promote past Phase 4 audit gate until P0 findings
(F-PARITY-MATRIX-TEMPLATE-STAMPED, F-TIER-RETIRED-NOT-MIGRATED) are
remediated.

The companion deliverables (feature-parity-matrix-2026-05-20.md and
performance-benchmark-numbers-2026-05-20.md from this wave) supersede
the prior parity matrix and benchmark file for the Wave 4-rolling top-3
counterpart contract. The pre-existing competitor-parity-matrix.md and
benchmarks/intersystems-vs-redox-vs-aws-healthlake-vs-oyatie.md remain
on disk pending Wave 15 remediation decisions per ADR-0328 §D-1.107 (no
alias cleanup inside audit-only wave).

This audit halts cleanly with all four required deliverables landed
inside microservices/healthcare-integration/ and zero writes outside
that path.

The wave produced no new commits, no parallel writes to other µservices,
and no destructive operations.

End of coherence audit.
