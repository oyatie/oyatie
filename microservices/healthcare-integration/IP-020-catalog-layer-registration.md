# IP-020 Healthcare Integration Catalog Layer Registration

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-020-catalog-layer-registration.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Owner teams: axis-healthcare-integration + catalog-platform + council-architecture
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local authorities: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/manifest.json; microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-api.yaml; microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-app.yaml; microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-domain.yaml; microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-kernel.yaml; microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-rest.yaml; microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-sdk.yaml; microservices/healthcare-integration/catalog/oya-healthcare-integration-clinical-interoperability-worker.yaml
Benchmark displacement set: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Register healthcare-integration catalog layers as enforceable Oyatie service ownership records, not as decorative Backstage-style cards.
- Preserve the flat microservice boundary from ADR-0131 and the PRD: clinical interoperability belongs under microservices/healthcare-integration, not under vendor suites or product-label folders.
- Make ADR-0105 layer slugs explicit for api, app, domain, kernel, rest, sdk, worker, adapter, usecase, cli, test, postgres, and valkey records.
- Bind every catalog row to the clinical-interoperability business capability and to a concrete healthcare capability record.
- Connect catalog registration to tenant doctrine from ADR-0242 and tenant scope from ADR-0244.
- Connect catalog registration to Cedar as universal gate from ADR-0243.
- Connect catalog registration to library-first policy and ontology routes from ADR-0246 and ADR-0257.
- Connect catalog registration to API versioning from ADR-0258.
- Connect catalog registration to observability emission from ADR-0263.
- Connect catalog registration to credential sidecar ownership from ADR-0296.
- Connect catalog registration to abuse-defence baseline from ADR-0297.
- Connect catalog registration to marketplace DealSet settlement from ADR-0314.
- Connect catalog registration to B2B leader coverage from ADR-0321.
- Ensure a catalog consumer can identify who owns FHIR reads, HL7 routing, break-glass authorization, consent sync, EHR provenance sealing, and patient-match review.
- Ensure a catalog consumer can identify contract sources, policy sources, SLO sources, runbook sources, dashboard sources, and rollback evidence sources.

## Current thinness being replaced
- The previous file repeated stamped capability lines without naming actual catalog records.
- It did not distinguish the API catalog row from REST, SDK, worker, adapter, domain, kernel, and persistence rows.
- It did not state ownership, dependencies, evidence links, or promotion gates.
- It did not state how catalog rows displace integration-engine inventories in Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, or Health Catalyst.
- It did not bind catalog records to PRD.md, manifest.json, capability YAML, contracts, policies, SLOs, dashboards, or runbooks.
- It did not prevent vendor-named catalog aliases from becoming service boundaries.

## Registration scope
- Catalog registration covers the healthcare-integration service and its clinical-interoperability bounded context.
- Catalog registration covers the api layer record for externally visible healthcare-integration service ownership.
- Catalog registration covers the app layer record for operator-facing app workflows.
- Catalog registration covers the domain layer record for clinical interoperability entities.
- Catalog registration covers the kernel layer record for invariant enforcement.
- Catalog registration covers the rest layer record for OpenAPI-backed REST entry points.
- Catalog registration covers the sdk layer record for generated client ownership.
- Catalog registration covers the worker layer record for async import, replay, and provider-network movement.
- Catalog registration covers the adapter layer record for EHR, HL7, FHIR, identity, and warehouse connectors.
- Catalog registration covers the usecase layer record for application-service orchestration.
- Catalog registration covers the cli layer record for operator remediation tools.
- Catalog registration covers the test layer record for contract, policy, replay, and migration fixtures.
- Catalog registration covers postgres and valkey records only as infrastructure dependencies, not as domain ownership.
- Catalog registration excludes vendor products as catalog owners.
- Catalog registration excludes suite folders.
- Catalog registration excludes anonymous connector buckets.
- Catalog registration excludes direct human-only inventory pages without machine-readable row data.
- Catalog registration excludes any row that cannot cite a local repo authority.

## Required catalog fields
- service: healthcare-integration.
- bounded_context: clinical-interoperability.
- layer: one ADR-0105 layer slug.
- owner_team: an accountable Oyatie team, not a vendor.
- lifecycle_state: planned, active, deprecated, or retired.
- source_contracts: local OpenAPI, AsyncAPI, proto, or capability YAML paths.
- source_policies: local Cedar or policy documentation paths.
- source_slos: local OpenSLO paths.
- source_runbooks: local runbook paths.
- source_dashboards: local dashboard JSON paths.
- source_adrs: current Binding ADR set.
- tenant_scope: required fields from capability YAML.
- data_classes: fhir_resource, hl7_message, clinical_consent, break_glass_event, patient_match_evidence, and provenance_seal.
- capability_refs: fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, patient-match-review.
- marketplace_settlement: DealSet when provider-network or commercial obligations are triggered.
- policy_mode: caller_side_library_first.
- ontology_mode: library_first.
- credential_mode: sidecar_reference_only.
- transport_mode: strict TLS with HTTP/3 fallback evidence.
- abuse_defence: anti-bot, anti-spoof, anti-scrape, replay controls.
- audit_chain: required.
- rollback_ref: runbook and evidence export path.
- promotion_gate: SLO and policy verification links.
- benchmark_displacement: named competitor displacement rows.

## Layer-specific registration
- api row owns service-level discoverability and links to manifest.json and PRD.md.
- api row must declare that healthcare-integration is not a vendor suite.
- app row owns operator flows for source discovery, transform preview, permit request, workflow run, projected-object inspection, audit verification, and rollback export.
- domain row owns clinical objects: patient-record, fhir-resource, hl7-message, referral, clinical-consent, break-glass-event, provenance-seal, and match-review-case.
- kernel row owns invariants for tenant scope, Cedar authorization, consent freshness, provenance sealing, and identity ambiguity.
- rest row owns externally reachable OpenAPI operations and HTTP transport obligations.
- sdk row owns generated client packages and SDK package provenance.
- worker row owns async import, replay, queue handling, dead-letter remediation, provider-network holds, and evidence bundle generation.
- adapter row owns source-specific connector logic without making vendors catalog owners.
- usecase row owns orchestration between policies, workflows, ontology projection, and audit-chain writes.
- cli row owns operator remediation commands for replay, consent conflict, export redaction, and patient-match duplicate review.
- test row owns contract fixtures, policy fixtures, replay fixtures, and benchmark displacement fixtures.
- postgres row owns relational persistence dependency metadata only.
- valkey row owns cache and queue dependency metadata only.

## Capability-to-catalog mapping
- fhir-read maps to rest, sdk, domain, kernel, policy, audit, and SLO catalog evidence.
- fhir-read must link to contracts/openapi-v1.yaml and slos/local-fhir-bundle-success.openslo.yaml when local SLO evidence is used.
- hl7-route maps to worker, adapter, domain, kernel, policy, audit, and SLO catalog evidence.
- hl7-route must link to slos/local-hl7-ack-latency.openslo.yaml and runbooks/local-hl7-ack-latency-burn.md for queue health.
- break-glass-authorize maps to app, rest, domain, kernel, policy, audit, and runbook evidence.
- break-glass-authorize must link to policies/local-breakglass-access-control.cedar and runbooks/local-breakglass-audit-review.md when local evidence is used.
- consent-sync maps to worker, domain, kernel, policy, audit, SLO, and runbook evidence.
- consent-sync must link to slos/local-consent-sync-freshness.openslo.yaml and runbooks/local-consent-sync-lag.md.
- ehr-provenance-seal maps to domain, kernel, audit, worker, dashboard, and runbook evidence.
- ehr-provenance-seal must link to runbooks/local-ehr-provenance-gap.md and dashboards/local-audit-completeness.json.
- patient-match-review maps to app, domain, kernel, worker, audit, and runbook evidence.
- patient-match-review must link to runbooks/local-patient-match-duplicate.md and human steward workflows.

## Benchmark displacement
- Redox displacement: catalog rows expose every service owner, policy source, SLO, and audit path instead of treating interoperability as a vendor-hosted connection inventory.
- Rhapsody displacement: catalog rows make route ownership, ACK latency, and replay runbooks discoverable without an engine console.
- InterSystems IRIS for Health displacement: catalog rows keep FHIR, HL7, and persistence dependency ownership separate so the database/runtime does not become the interoperability product boundary.
- Lyniate/Corepoint displacement: catalog rows turn interface definitions into ADR-bound service-layer records with local contracts and rollback paths.
- Mirth Connect displacement: catalog rows replace channel-script tribal knowledge with governed layer records, policy evidence, and replay fixtures.
- NextGate displacement: catalog rows make patient identity review ownership explicit and separate from black-box identity matching.
- Health Catalyst displacement: catalog rows keep analytics extraction as downstream consumption, not the owner of clinical interoperability controls.
- Redox-like connector catalogs are insufficient unless every row carries tenant, policy, audit, and DealSet evidence.
- Rhapsody-like engine catalogs are insufficient unless operator remediation runbooks are linked from service records.
- InterSystems-like platform catalogs are insufficient unless infrastructure rows cannot own domain concepts.
- Lyniate/Corepoint-like interface inventories are insufficient unless row changes pass Oyatie ADR and SLO gates.
- Mirth-like channel lists are insufficient unless script behavior is converted to typed contract references.
- NextGate-like match inventories are insufficient unless review and rollback workflow ids are first-class.
- Health Catalyst-like pipeline catalogs are insufficient unless data extraction does not override clinical consent controls.

## Implementation steps
- Step 1: Read manifest.json and all existing catalog YAML records for healthcare-integration.
- Step 2: Normalize every catalog row to service=healthcare-integration and bounded_context=clinical-interoperability.
- Step 3: Verify every row uses an ADR-0105 layer slug.
- Step 4: Add source_adrs with the current Binding ADR set.
- Step 5: Add capability_refs for fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, and patient-match-review.
- Step 6: Add source_contracts to rest, sdk, worker, and test rows.
- Step 7: Add source_policies to kernel, domain, usecase, rest, worker, and app rows.
- Step 8: Add source_slos to app, rest, worker, and test rows.
- Step 9: Add source_runbooks to app, worker, cli, and test rows.
- Step 10: Add source_dashboards to app, worker, api, and operations-facing rows.
- Step 11: Add tenant_scope required fields to capability-bearing rows.
- Step 12: Add policy_mode=caller_side_library_first where Cedar evaluation is referenced.
- Step 13: Add ontology_mode=library_first where ontology projections are referenced.
- Step 14: Add credential_mode=sidecar_reference_only where connector or credential resolution is referenced.
- Step 15: Add marketplace_settlement=DealSet for provider-network and commercial movement rows.
- Step 16: Add benchmark_displacement rows for the seven named competitors.
- Step 17: Add row-level promotion_gate references for policy, SLO, and audit verification.
- Step 18: Reject catalog aliases that contain vendor names as owner or bounded context.
- Step 19: Reject catalog rows with missing local repo references.
- Step 20: Record row digests for admission evidence.

## Tests and evidence
- Catalog test: every healthcare-integration catalog row has service=healthcare-integration.
- Catalog test: every catalog row has an ADR-0105 layer slug.
- Catalog test: no catalog owner is Redox, Rhapsody, InterSystems, Lyniate, Corepoint, Mirth, NextGate, Health Catalyst, Epic, Cerner, Allscripts, or Veeva.
- Catalog test: every capability-bearing row cites at least one capability YAML path.
- Catalog test: every externally visible row cites PRD.md and manifest.json.
- Catalog test: rest row cites openapi-v1.yaml.
- Catalog test: sdk row cites sdk-plan.md and openapi-v1.yaml.
- Catalog test: worker row cites asyncapi-v1.yaml and at least one runbook.
- Catalog test: policy-bearing rows cite Cedar or policy docs.
- Catalog test: SLO-bearing rows cite local OpenSLO files.
- Catalog test: dashboard-bearing rows cite dashboard JSON files.
- Catalog test: row-level source_adrs match the current Binding ADR set.
- Catalog test: tenant_scope required fields include tenant_id and principal_id.
- Catalog test: DealSet appears where provider-network settlement applies.
- Catalog test: benchmark displacement text names all seven required competitors.
- Catalog test: no row creates a suite folder or anonymous connector bucket.
- Catalog test: no infrastructure dependency row owns clinical domain concepts.
- Catalog test: rollback references exist for rows with runtime behavior.
- Catalog test: catalog row hashes change when source references change.

## Rollback
- Roll back by reverting only incorrect catalog row metadata.
- Do not roll back PRD.md, manifest.json, contracts, capability YAML, policies, SLOs, dashboards, or runbooks as part of this IP.
- Keep deprecated row ids reserved so old evidence bundles remain traceable.
- Mark bad rows deprecated before replacement when downstream references already exist.
- Remove vendor-named aliases instead of preserving them as deprecated rows.
- Preserve audit evidence for row addition, update, deprecation, and replacement.
- Keep admission blocked if rollback would orphan a runtime layer.
- Keep admission blocked if rollback would remove source_adrs.
- Keep admission blocked if rollback would remove tenant scope from a capability-bearing row.
- Keep admission blocked if rollback would make infrastructure own clinical domain semantics.

## Catalog evidence fields
- Catalog evidence must record row id.
- Catalog evidence must record layer slug.
- Catalog evidence must record owner team.
- Catalog evidence must record capability refs.
- Catalog evidence must record source contract refs.
- Catalog evidence must record source policy refs.
- Catalog evidence must record SLO refs.
- Catalog evidence must record runbook refs.
- Catalog evidence must record dashboard refs.
- Catalog evidence must record benchmark displacement rows.
- Catalog evidence must record row digest.
- Catalog evidence must record deprecation successor when applicable.

## Acceptance criteria
- Catalog rows cover every ADR-0105 layer already represented under microservices/healthcare-integration/catalog.
- Catalog rows cite PRD.md, manifest.json, relevant contracts, capability records, policies, SLOs, dashboards, and runbooks.
- Catalog rows name the current Binding ADR set used by healthcare-integration IPs.
- Catalog rows preserve flat microservice ownership and reject vendor-suite ownership.
- Catalog rows make policy, audit, SLO, rollback, credential, ontology, and DealSet evidence discoverable.
- Catalog registration displaces Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst with repo-owned B2B control-plane records.
- No catalog row hides clinical route, consent, break-glass, provenance, or patient-match ownership behind a generic connector label.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-020-catalog-layer-registration.md:101` - - fhir-read must link to contracts/openapi-v1.yaml and slos/local-fhir-bundle-success.openslo.yaml when local SLO evidence is used.; `microservices/healthcare-integration/IP-020-catalog-layer-registration.md:157` - - Catalog test: rest row cites openapi-v1.yaml..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-020-catalog-layer-registration.md:27` - - Ensure a catalog consumer can identify contract sources, policy sources, SLO sources, runbook sources, dashboard sources, and rollback evidence sources.; `microservices/healthcare-integration/IP-020-catalog-layer-registration.md:34` - - It did not bind catalog records to PRD.md, manifest.json, capability YAML, contracts, policies, SLOs, dashboards, or runbooks..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-020-catalog-layer-registration.md:21` - - Connect catalog registration to observability emission from ADR-0263..
