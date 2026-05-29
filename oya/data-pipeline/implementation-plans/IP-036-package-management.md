# IP-036 Data Pipeline package management finalization

Service: data-pipeline
Implementation plan: IP-036
Wave: 15A-DATA-PIPELINE-FINALIZER
Date: 2026-05-21
Scope path: microservices/data-pipeline/implementation-plans/IP-036-package-management.md
Audit source: microservices/data-pipeline/coherence-audit-2026-05-20.md
Audit finding: Section 3.9.2 names package management as missing.
Parity source: microservices/data-pipeline/feature-parity-matrix-2026-05-20.md
Primary ADR: microservices/data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md

## Scope
- Add package-management as a transform sub-context.
- Make transforms, connector packages, semantic metrics, materialization templates, exposure templates, compliance extensions, runbooks, and datasets installable and lockfile-pinned.
- Provide dbt deps and dbt Hub parity while binding registry source to Oyatie marketplace and tenant-local artifacts.
- Make every package install Cedar-gated, DealSet-aware where commercial, and replay-reproducible.
- Make Foundry package authoring possible under operator approval for restricted package categories.
- Preserve no-tier doctrine: package access is governed by tenant_class, DealSet, pack overlay, and billing components.
- Close the audit and feature matrix package-management gap.
- Feed IP-037 CDK publishing and IP-033 semantic metric packages.
- Preserve ADR-MS-001 lineage and replay evidence for package-driven runs.
- No files outside microservices/data-pipeline/ are required for this plan.

## Interfaces
- REST command `POST /data-pipeline/actions/package.publish`.
- REST command `POST /data-pipeline/actions/package.install`.
- REST command `POST /data-pipeline/actions/package.uninstall`.
- REST command `POST /data-pipeline/actions/package.pin`.
- REST command `POST /data-pipeline/actions/package.update`.
- REST command `POST /data-pipeline/actions/package.verify-signature`.
- REST query `GET /data-pipeline/packages/{package_id}/lockfile`.
- gRPC service `PackageRegistryControl`.
- Contract `contracts/package-registry-v1.yaml`.
- Capability records `capabilities/package-publish.yaml`, `package-install.yaml`, and `package-pin.yaml`.
- Cedar fragments `policies/local-package-install-scope.cedar`, `local-package-publish-scope.cedar`, and `local-package-pin-scope.cedar`.
- SLO projection `slos/local-package-install-latency.openslo.yaml`.
- Runbooks `runbooks/package-install-conflict.md` and `package-signature-verification-failure.md`.

## Data Flow
- Author publishes package_manifest_binding with package_id, package_version, category, dependency list, signature, and source_kind.
- Cedar validates author audience, tenant scope, DealSet state, pack overlay, and restricted-category approval.
- Resolver computes deterministic transitive dependency set.
- Lockfile fingerprint is generated from package ids, versions, source hashes, and signatures.
- Tenant installs package by lockfile, not by mutable version range.
- Runtime verifies lockfile before package use.
- CDK packages from IP-037 publish as connector_package.
- Semantic packages feed IP-033 metric registry.
- Materialization template packages feed IP-035.
- Exposure template packages feed IP-034.
- Dataset packages emit DealSet usage from IP-014.
- Package uninstall preserves lockfile for replay evidence.

## Cedar Policy
- Deny package.publish without tenant_package_steward or approved Foundry package_author.
- Deny package.publish if package signature is missing or invalid.
- Deny dataset_package publish without DealSet.
- Deny connector_package marketplace publish without operator approval.
- Deny compliance_pack_extension_package install when underlying pack inactive.
- Deny package.install when dependency is deprecated beyond grace window.
- Deny package.install when tenant_class lacks package capacity grant.
- Deny package.pin when actor lacks package steward authority.
- Deny package.update when lockfile diff exceeds safe threshold without operator review.
- Deny runtime package use when lockfile fingerprint drift is detected.
- Deny cross-tenant namespace publish.
- Deny package operation during audit-chain outage.

## Event Shapes
- `oya.data.pipeline.package.published` carries tenant_id, tenant_class, package_id, package_version, package_category, source_kind.
- `oya.data.pipeline.package.signature_verified` carries signature_chain_id, verification_result, registry_source, policy_decision_id.
- `oya.data.pipeline.package.lockfile_resolved` carries lockfile_fingerprint, dependency_count, resolver_version, conflict_count.
- `oya.data.pipeline.package.installed` carries package_id, package_version, lockfile_fingerprint, marketplace_dealset_id, tenant_local_artifact_uri.
- `oya.data.pipeline.package.pinned` carries pin_reason, pinned_by, lockfile_fingerprint, replay_scope.
- `oya.data.pipeline.package.updated` carries prior_lockfile, next_lockfile, safe_diff_result.
- `oya.data.pipeline.package.uninstalled` carries removed_at, retained_for_replay, replacement_package_id.
- Every event includes audit_event_id, cedar_decision_id, traceparent, home_cell, and foundry_lane when relevant.

## SLO Targets
- Reuse `availability.openslo.yaml` target 0.999 for package registry availability.
- Reuse `write-latency.openslo.yaml` target 0.999 for package mutation commands.
- Reuse `read-latency.openslo.yaml` target 0.999 for package and lockfile reads.
- Reuse `policy-decision-latency.openslo.yaml` target 0.999 for install authorization.
- Reuse `audit-emission-lag.openslo.yaml` target 0.999 for package events.
- Reuse `local-ingest-freshness.openslo.yaml` target 0.995 for dataset packages.
- Reuse `local-transform-latency.openslo.yaml` target 0.99 for transform packages.
- Reuse `local-lineage-capture.openslo.yaml` target 0.999 for package-driven lineage.
- Reuse `local-schema-drift-latency.openslo.yaml` target 0.999 for connector package drift.
- Reuse `local-quality-null-rate.openslo.yaml` target 0.999 for package quality gates.
- Reuse `replay-freshness.openslo.yaml` target 0.999 for package replay reproducibility.
- Reuse `local-deadletter-rate.openslo.yaml` target 0.995 for failed package jobs.
- Add `local-package-install-latency.openslo.yaml`: p95 tenant-local 30s, marketplace single-tenant 60s, marketplace multi-tenant 120s.

## Failure Modes
- Dependency resolver conflict refuses install and links package-install-conflict runbook.
- Signature verification failure refuses publish or install.
- Marketplace DealSet lapses after install; package is marked dealset_invalid but retained for replay.
- Lockfile drift at runtime refuses pipeline run.
- Restricted category lacks operator approval and remains staging.
- Deprecated dependency past grace window blocks install.
- Pack overlay change makes compliance extension illegal and pauses package use.
- Cedar outage fails closed.
- Audit-chain outage holds operation.
- Foundry-authored package exceeds rate limit and is deferred.
- Registry source unavailable serves pinned packages only.
- Package uninstall without replacement triggers IP-034 exposure impact.

## Migration
- Add package-management to manifest bounded_sub_contexts under transform.
- Register existing connector, transform, semantic, and runbook artifacts as tenant-local packages only when they are intended for reuse.
- Do not convert all markdown into packages; package only reusable executable or governed artifacts.
- Root IP-036 remains historical evidence; this file is the implementation-plans handoff.
- Add tenant_class to all package events.
- Replace package tier wording with tenant_class capacity and DealSet billing.
- Introduce tenant-local registry before marketplace-backed registry.
- Introduce restricted Foundry publish after human approval flow exists.
- Preserve all package uninstalls as replay-visible events.
- Keep old version pins immutable.
- No foreign service writes are needed.
- Marketplace integration uses contracts only.

## Dependencies
- IP-001 tenant scope kernel supplies package TenantScope.
- IP-002 Cedar default deny gates publish and install.
- IP-003 ontology projection may consume dataset packages.
- IP-004 workflow templates may consume package templates.
- IP-005 REST surface publishes package endpoints.
- IP-006 async event surface publishes package events.
- IP-007 gRPC surface publishes registry control.
- IP-008 policy eval binding evaluates package Cedar.
- IP-009 credential sidecar protects registry credentials.
- IP-010 multi-region layout constrains registry home_cell.
- IP-011 audit events records package operations.
- IP-012 abuse defence protects public package paths.
- IP-013 emergency bypass cannot bypass package policy.
- IP-014 DealSet settlement licenses marketplace packages.
- IP-015 residency overlays constrain package content.
- IP-016 backfill replay uses lockfiles.
- IP-017 cost budget enforcer meters package jobs.
- IP-018 capacity admission controls package install concurrency.
- IP-019 SDK generation exposes package clients.
- IP-020 catalog registration catalogs package domain.
- IP-021 SLO promotion blocks rollout on package burn.
- IP-022 chaos drills test resolver and signature failures.
- IP-023 DPIA records dataset package privacy.
- IP-024 threat map covers dependency and supply-chain risk.
- IP-025 audit closeout proves package finding closure.
- IP-026 drift quarantine blocks unsafe connector package.
- IP-027 lineage reconciliation consumes package lineage facets.
- IP-028 dead-letter custody replays package job failures.
- IP-029 transform cost attribution records package transform cost.
- IP-030 watermark governance gates connector package CDC.

## ADR-MS-001 Binding
- Package-driven transforms and connectors inherit lineage-first and replay custody.
- Historical replay reads the package lockfile active at the original run.
- Package changes are append-only events, not mutable hidden state.
- Package metrics avoid raw tenant identifiers.
- Policy and audit evidence precede package runtime use.
- Cost attribution includes package id and version.

## Acceptance Gates
- Gate 1: package-management appears under transform bounded_sub_contexts.
- Gate 2: all eight package categories have domain tests.
- Gate 3: lockfile resolution is deterministic.
- Gate 4: signature verification is mandatory.
- Gate 5: marketplace dataset and connector packages require DealSet.
- Gate 6: Foundry publish is human-approved for restricted categories.
- Gate 7: runtime lockfile drift denies execution.
- Gate 8: package registry contract is published.
- Gate 9: all 12 existing OpenSLOs are cited in promotion checklist.
- Gate 10: local-package-install-latency SLO is filed.
- Gate 11: IP-001 through IP-030 references remain intact in this plan.
- Gate 12: remediation notes mark audit package-management gap closed by this IP.


## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-036-package-management.md:37` - - SLO projection `slos/local-package-install-latency.openslo.yaml`.; `microservices/data-pipeline/implementation-plans/IP-036-package-management.md:78` - ## SLO Targets.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/implementation-plans/IP-036-package-management.md:83` - - Reuse `audit-emission-lag.openslo.yaml` target 0.999 for package events.; `microservices/data-pipeline/implementation-plans/IP-036-package-management.md:138` - - IP-017 cost budget enforcer meters package jobs..
