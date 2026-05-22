---
doc_class: Remediation-Notes
microservice: diagnostics
wave: 15M-RECONCILE
date: 2026-05-21
status: reconciled
owner: wave-15m-reconcile
canonical_sources:
  - ../imaging/PRD.md
  - ../imaging/REMEDIATION-NOTES-2026-05-21.md
  - ../../docs/decisions/ADR-0332-healthcare-domain-decomposition.md
  - ../../docs/decisions/ADR-0132-no-suite-policy.md
---

# Diagnostics Remediation Notes - 2026-05-21

## Reconciliation Log

Wave 15M-RECONCILE removed the concurrently-authored imaging bundle from diagnostics after the dedicated imaging microservice became authoritative.

## Removed From Diagnostics

- Bounded contexts: `ImagingOrder`, `ImagingResult`, `DICOMStudy`, and imaging-specific variants.
- Contracts: DICOMweb, DIMSE, PACS/VNA, FHIR `ImagingStudy`, imaging report, and radiology API surfaces.
- SLOs: `dicom-c-store-success.openslo.yaml` and `imaging-study-register-to-viewer-open.openslo.yaml`.
- Cedar policies: radiologist and imaging technologist policies.
- Local decisions/IPs: diagnostics DICOM substrate ADR and DICOM substrate implementation plan.
- IaC and OS support defaults that provisioned diagnostics-local image object custody.
- Counterpart vendors: GE Centricity, Philips IntelliSpace, Sectra PACS+VNA, Epic Radiant, and other imaging/PACS vendors.

## Remaining Diagnostics Scope

Diagnostics owns lab and pathology only:

- lab orders and lab results;
- pathology cases and sign-out;
- specimens and chain-of-custody;
- critical-result escalation;
- reference ranges and reflex testing;
- result authorization, interpretation, delivery, TAT, and QC.

## Allowed Imaging References

The only allowed diagnostics references to imaging are:

- supersession references to `microservices/imaging/`;
- cross-service image-correlation request/response handoffs;
- notes that explicitly say diagnostics does not own image artifacts, imaging reports, PACS/VNA, or DICOM.

## Capability YAML Check

No `microservices/diagnostics/capabilities/` directory or diagnostics capability YAMLs were present during this reconciliation. No capability YAML deletion was required.

## ADR / Plan Updates

- ADR-0332 now lists imaging as the eighth new healthcare microservice.
- ADR-0332 now describes total healthcare-domain count as eight new plus one narrowed existing, nine total.
- ADR-0332 §C includes the dedicated imaging scope, top-3 counterparts, and cross-service handoffs.
- ADR-0332 §D includes imaging handoffs for EMR orders, EMR report delivery, diagnostics correlation, billing, emergency, CDS, and healthcare-integration broker flows.
- `.omc/plans/healthcare-decomposition-plan-2026-05-21.md` adds imaging to Wave 15M-B and updates IP, compliance-pack, parallelism, and verification counts.

<!-- WAVE-15M-RECONCILE-COMPLETION
status: reconciled
date: 2026-05-21
owner: wave-15m-reconcile
diagnostics_scope: lab + pathology only
imaging_authority: microservices/imaging/PRD.md
removed_contexts: ImagingOrder, ImagingResult, DICOMStudy
removed_contracts: DICOMweb, DIMSE, PACS/VNA, FHIR ImagingStudy, imaging report APIs
removed_slos: dicom-c-store-success.openslo.yaml, imaging-study-register-to-viewer-open.openslo.yaml
removed_cedar: radiologist-can-read.cedar, technologist-can-acquire.cedar
capability_yaml_status: none present under microservices/diagnostics
adr_0332_count: 8 new healthcare microservices + 1 narrowed healthcare-integration = 9 total
commit_status: no commits
-->

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- No rewrite required; the service had no Redis vocabulary in the Wave 15-Valkey inventory.

Counterpart-fact preservations:
- None.

Files renamed:
- None.
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 9.
- Trigger A matched: 0.
- Trigger B matched: 1.
- Trigger C matched: 0.
- Trigger D matched: 0.
- IPs unmatched: 8.

### IP changes
- `microservices/diagnostics/implementation-plans/IP-009-critical-result-escalation.md` — added DR posture.

### Unmatched IPs
- `microservices/diagnostics/implementation-plans/IP-001-tenant-scope-kernel.md` — no trigger match; no doctrine section added.
- `microservices/diagnostics/implementation-plans/IP-002-cedar-default-deny.md` — no trigger match; no doctrine section added.
- `microservices/diagnostics/implementation-plans/IP-003-ontology-projection.md` — no trigger match; no doctrine section added.
- `microservices/diagnostics/implementation-plans/IP-004-workflow-template-library.md` — no trigger match; no doctrine section added.
- `microservices/diagnostics/implementation-plans/IP-005-rest-contract-surface.md` — no trigger match; no doctrine section added.
- `microservices/diagnostics/implementation-plans/IP-006-async-event-surface.md` — no trigger match; no doctrine section added.
- `microservices/diagnostics/implementation-plans/IP-008-reflex-engine.md` — no trigger match; no doctrine section added.
- `microservices/diagnostics/implementation-plans/IP-010-pack-overlay-and-migration.md` — no trigger match; no doctrine section added.

### Follow-up
- `microservices/diagnostics/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-12.
- Scope: PRD doctrine propagation for `diagnostics`; PRD values match the present `manifest.json` `dr` and `capacity_model` blocks.

### DR posture
- Values: RTO 3600s, RPO 300s, multi-region enabled, failover_runbook `microservices/diagnostics/runbooks/diagnostics-result-failover.md`.
- ADR: ADR-0343; HIPAA/SOC2/ISO/KR-PIPA floors are satisfied by the 3600s/300s lab target.
- Alternative considered: rely on EMR DR only; rejected because critical-result acknowledgement and specimen custody are diagnostics-owned safety records.
- Cost: needs a diagnostics-local failover runbook and duplicate critical-result route testing.

### Capacity model
- Values: 0.2 vCPU, 512 MiB RAM, 8 GB storage, 4 Postgres connections, 2 Valkey connections, 8 outbound HTTP connections; `per_message` scaling; Tier-3 placement; 2-24 pods per tenant cell.
- ADR: ADR-0340.
- Alternative considered: Tier-1 or Tier-2 clinical-data placement; rejected because the manifest declares Diagnostics as a first-party healthcare application aligned to `pod_runtime_tier=2`.
- Cost: commits the service to message backlog sizing instead of only HTTP request autoscaling.

### Sustainability + cost attribution
- Values: audit rows carry `cost_usd_minor_units`, `co2_grams`, and `watt_hours`; carbon routing disabled for STAT labs, critical values, pathology sign-out, and HIPAA emergency paths.
- ADR: ADR-0344.
- Alternative considered: make all analytics carbon-deferrable; rejected because result publication and sign-out are clinical safety paths.
- Cost: adds per-order and per-case attribution dimensions to finops reporting.

### API versioning
- Values: YYYY-MM-DD carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for lab integrations, internal-mesh exemption.
- ADR: ADR-0342.
- Alternative considered: FHIR-only versioning; rejected because lab/pathology OpenAPI, AsyncAPI, and proto3 contracts also cross tenant/vendor boundaries.
- Cost: maintains compatibility lanes for analyzer, reference-range, and report-delivery consumers.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.20 vCPU, 512 MiB RAM, and per_message scaling reflect lab/order/result event throughput rather than clinician session residency.
- ADR: ADR-0340 capacity envelopes and ADR-0340 D-6 pod-runtime/cell-placement covariance.
- Rejected: Rejected Tier-2 placement because Diagnostics is a product application and pod_runtime_tier=2 co-varies cleanly with Tier-3.
- Cost: Commits the service to message backlog sizing and object-store retention instead of only HTTP request autoscaling.

### Block 2: dr
- Values: RTO 3600s, RPO 300s, active-active true, backup substrates postgres_wal_g, object_storage_versioned, audit_chain_merkle_seal.
- ADR: ADR-0343 recoverability floors, with compliance-pack floors treated as minimums.
- Rejected: Rejected sub-minute RPO because result workflows can replay from sealed lab/event queues without losing the final charted result.
- Cost: Commits the service to runbook-backed failover drills and evidence capture at runbooks/diagnostics-result-failover.md.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; Diagnostics is a first-party lab and pathology application that stores and routes PHI results, but it does not own a shared substrate control plane or execute tenant-customer code. Tier 2 keeps it in the standard application isolation lane while HIPAA recovery obligations are handled by DR and audit-chain backups.
- ADR: ADR-0338 pod runtime tiering and ADR-0340 D-6 covariance.
- Rejected: Rejected Tier 1 because this service does not expose shared substrate credentials, keys, or tenant data-plane infrastructure.
- Cost: Commits placement and scheduling to the declared runtime isolation class rather than cheapest generic app placement.

### Block 4: tenant_version_pinning
- Values: declared version 2026-05-21, default 2026-05-21, three-version support window, 180 day minimum support, per-tenant pinning enabled.
- ADR: ADR-0342 tenant/API version pinning and manifest schema public_surface_files contract map.
- Rejected: Rejected synthetic historical API dates because only current openapi/asyncapi/proto contract files are present.
- Cost: Future contract changes need explicit version calendars and migration documents before tenant sunset.

### Block 5: consumes_upstream_oss
- Values: postgresql, cedar, openbao, opentofu.
- ADR: ADR-0345 OSS stewardship declarations, using registry dep_name strings from specs/oss-stewardship-registry.json.
- Rejected: Rejected adding Valkey without a service-local evidence path proving it is part of Diagnostics runtime state.
- Cost: CVE response ownership and upgrade stewardship now attach to the declared upstream substrate set.

### Block 6: iac_module_invocations
- Values: aws-guest/tenant-namespace, aws-guest/postgres-wal-g, aws-guest/object-storage-versioned, oci-guest/tenant-namespace, oci-guest/object-storage-versioned, oci-guest/always-free/tenant-namespace, on-prem/object-storage-versioned, colo/object-storage-versioned, oyatie-as-cloud-provider/shard-cell.
- ADR: ADR-0339 shared IaC module invocation doctrine and manifest schema authority.
- Rejected: Rejected sovereign as a manifest context because the schema has no sovereign context enum for iac_module_invocations.
- Cost: Provider-specific IaC must remain a thin invocation layer over shared module primitives and version pins.
