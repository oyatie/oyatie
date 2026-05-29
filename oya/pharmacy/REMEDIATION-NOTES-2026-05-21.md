# Pharmacy Remediation Notes - 2026-05-21

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/pharmacy/ARCHITECTURE.md
- microservices/pharmacy/manifest.json

Counterpart-fact preservations:
- None.

Files renamed:
- None.
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 10.
- Trigger A matched: 0.
- Trigger B matched: 3.
- Trigger C matched: 4.
- Trigger D matched: 0.
- IPs unmatched: 5.

### IP changes
- `microservices/pharmacy/implementation-plans/IP-003-eprescribe-surescripts-epcs.md` — added DR posture, Sustainability emission.
- `microservices/pharmacy/implementation-plans/IP-004-drug-interaction-eight-engine.md` — added DR posture, Sustainability emission.
- `microservices/pharmacy/implementation-plans/IP-005-allergy-check-cross-class.md` — added Sustainability emission.
- `microservices/pharmacy/implementation-plans/IP-007-verification-tall-man-dual-verify.md` — added Sustainability emission.
- `microservices/pharmacy/implementation-plans/IP-010-reimbursement-340b-pbm.md` — added DR posture.

### Unmatched IPs
- `microservices/pharmacy/implementation-plans/IP-001-medication-catalog-fdb-ingest.md` — no trigger match; no doctrine section added.
- `microservices/pharmacy/implementation-plans/IP-002-formulary-pt-workflow.md` — no trigger match; no doctrine section added.
- `microservices/pharmacy/implementation-plans/IP-006-dose-check-renal-hepatic.md` — no trigger match; no doctrine section added.
- `microservices/pharmacy/implementation-plans/IP-008-compounding-usp-795-797-800.md` — no trigger match; no doctrine section added.
- `microservices/pharmacy/implementation-plans/IP-009-inventory-recall-cabinet-adapter.md` — no trigger match; no doctrine section added.

### Follow-up
- `microservices/pharmacy/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-12.
- Scope: PRD doctrine propagation for `pharmacy`; PRD values match the present `manifest.json` `dr` and `capacity_model` blocks.

### DR posture
- Values: RTO 900s, RPO 60s, active-active safety paths, failover_runbook `microservices/pharmacy/runbooks/pharmacy-dispense-failover.md`.
- ADR: ADR-0343; HIPAA/PCI/SOC2/ISO/KR-PIPA floors are satisfied by the pharmacy safety target.
- Alternative considered: use the looser PCI floor for PBM/payment paths; rejected because dispense and BCMA cannot inherit payment-system tolerance.
- Cost: requires active-active controlled-substance, DSCSA, and BCMA custody evidence.

### Capacity model
- Values: 0.55 vCPU, 1280 MiB RAM, 20 GB storage, 8 Postgres connections, 8 Valkey connections, 12 outbound HTTP connections; `per_workflow_run` scaling; Tier-2 placement; 2-28 pods per tenant cell.
- ADR: ADR-0340.
- Alternative considered: a single dispense-queue scale metric; rejected because the manifest declares dispense, verification, EPCS, BCMA, DSCSA, and reimbursement workflow runs as the scaling unit.
- Cost: requires workflow-run admission and capacity accounting across clinical and commercial pharmacy paths.

### Sustainability + cost attribution
- Values: audit rows carry `cost_usd_minor_units`, `co2_grams`, and `watt_hours`; carbon routing disabled for DDI/DAI, dose checks, BCMA, controlled substances, EPCS, and real-time payment/fraud paths.
- ADR: ADR-0344; ADR-0337 applies to pharmacy KPI streams written to the data-warehouse path.
- Alternative considered: carbon-route catalog and safety checks together; rejected because safety checks sit in medication-administration hot paths.
- Cost: adds per-dispense, per-catalog-ingest, and per-PBM attribution.

### API versioning
- Values: YYYY-MM-DD carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for Surescripts/NCPDP/PBM/cabinet/pump integrations, internal-mesh exemption.
- ADR: ADR-0342.
- Alternative considered: network-vendor native versioning only; rejected because tenant pinning must survive vendor adapter swaps.
- Cost: maintains compatibility matrices across clinical and commercial pharmacy networks.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.55 vCPU, 1280 MiB RAM, 20 GB storage, and per_workflow_run scaling match medication dispense, e-prescribe, BCMA, DSCSA, and reimbursement workflows.
- ADR: ADR-0340 capacity envelopes and ADR-0340 D-6 pod-runtime/cell-placement covariance.
- Rejected: Rejected Tier-3 placement because pod_runtime_tier=1 requires Tier-0, Tier-1, or Tier-2 cell placement under ADR-0340 D-6.
- Cost: Commits Pharmacy to Kata overhead and sealed audit/key recovery paths for medication-safety continuity.

### Block 2: dr
- Values: RTO 900s, RPO 60s, active-active true, backup substrates postgres_wal_g, valkey_cluster, object_storage_versioned, openbao_seal_unseal, audit_chain_merkle_seal.
- ADR: ADR-0343 recoverability floors, with compliance-pack floors treated as minimums.
- Rejected: Rejected 300s RTO because the available manifest evidence supports clinical continuity, but not a bedside-monitoring class failover path.
- Cost: Commits the service to runbook-backed failover drills and evidence capture at runbooks/pharmacy-dispense-failover.md.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; Pharmacy handles controlled-substance, EPCS, BCMA, inventory, and reimbursement records where tenant PHI and medication-safety state drive clinical action. It does not execute tenant code, but it is a regulated tenant data-plane service with key and audit dependencies, so Tier 1 plus Tier-2 cell placement is the valid doctrine pairing.
- ADR: ADR-0338 pod runtime tiering and ADR-0340 D-6 covariance.
- Rejected: Rejected Tier 0 because Pharmacy orchestrates regulated workflows but does not host tenant-customer code.
- Cost: Commits placement and scheduling to the declared runtime isolation class rather than cheapest generic app placement.

### Block 4: tenant_version_pinning
- Values: declared version 2026-05-21, default 2026-05-21, three-version support window, 180 day minimum support, per-tenant pinning enabled.
- ADR: ADR-0342 tenant/API version pinning and manifest schema public_surface_files contract map.
- Rejected: Rejected synthetic historical API dates because only current v1-equivalent contract files exist.
- Cost: Future contract changes need explicit version calendars and migration documents before tenant sunset.

### Block 5: consumes_upstream_oss
- Values: postgresql, valkey, cedar, openbao, opentofu.
- ADR: ADR-0345 OSS stewardship declarations, using registry dep_name strings from specs/oss-stewardship-registry.json.
- Rejected: Rejected Citus and Pulsar names because they are not registry dep_name values in the current stewardship registry.
- Cost: CVE response ownership and upgrade stewardship now attach to the declared upstream substrate set.

### Block 6: iac_module_invocations
- Values: aws-guest/tenant-namespace, aws-guest/postgres-wal-g, aws-guest/valkey-cluster, aws-guest/openbao-bindings, oci-guest/tenant-namespace, oci-guest/postgres-wal-g, oci-guest/always-free/tenant-namespace, on-prem/tenant-namespace, colo/tenant-namespace, oyatie-as-cloud-provider/per-cell-nodepool-kata.
- ADR: ADR-0339 shared IaC module invocation doctrine and manifest schema authority.
- Rejected: Rejected sovereign as an iac_module_invocations context because the manifest schema closes the context enum.
- Cost: Provider-specific IaC must remain a thin invocation layer over shared module primitives and version pins.
