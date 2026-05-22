# IP-019 Healthcare Integration SDK Client Generation

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-019-sdk-client-generation.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Owner teams: axis-healthcare-integration + platform-sdk + council-product
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local authorities: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/sdk-plan.md; microservices/healthcare-integration/contracts/openapi-v1.yaml; microservices/healthcare-integration/contracts/asyncapi-v1.yaml; microservices/healthcare-integration/contracts/healthcare-integration-v1.proto; microservices/healthcare-integration/capabilities/fhir-read.yaml; microservices/healthcare-integration/capabilities/hl7-route.yaml; microservices/healthcare-integration/capabilities/break-glass-authorize.yaml; microservices/healthcare-integration/capabilities/consent-sync.yaml; microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml; microservices/healthcare-integration/capabilities/patient-match-review.yaml
Benchmark displacement set: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Build SDK generation as a product-grade clinical interoperability client surface, not as a loose wrapper around generated OpenAPI methods.
- Preserve the PRD boundary: healthcare-integration owns regulated clinical interoperability, consent, break-glass, patient matching, and provenance without creating a vendor-named suite boundary.
- Treat generated clients as controlled clinical execution surfaces with tenant scope, principal identity, audience type, purpose, data class, idempotency, audit, policy, and DealSet settlement built into every call.
- Use ADR-0105 layer names when emitting package boundaries so generated code lands in the sdk layer and never backfills domain, usecase, worker, adapter, or rest logic.
- Use ADR-0131 to keep all SDK generation under the flat microservice ownership model instead of adding a healthcare suite folder.
- Use ADR-0242 and ADR-0244 to force tenant_id as a required constructor and per-request field, including read-only FHIR access.
- Use ADR-0243, ADR-0294, and the local Cedar policy fragments to make deny evidence a first-class SDK response type.
- Use ADR-0246 and ADR-0257 to keep ontology reads library-first and network-opt-in from SDK helper calls.
- Use ADR-0258 to version SDK packages from contract semantic versions instead of hand-maintained language package versions.
- Use ADR-0253-amendment to expose HTTP/3 fallback, strict TLS, ECH, and PQC capability negotiation as typed transport options.
- Use ADR-0263 to emit trace, metric, structured log, and audit-chain correlation identifiers from every generated operation.
- Use ADR-0296 to bind credential-sidecar references without placing PHI-bearing secrets in generated clients.
- Use ADR-0297 to make scraping, spoofing, bot, and replay refusal outcomes machine-readable for client operators.
- Use ADR-0314 to bind marketplace DealSet settlement metadata to client calls that trigger billable provider-network movement.
- Use ADR-0321 to keep benchmark coverage as tenant_class substance over Oyatie substrate rather than vendor parity theater.

## Current thinness being replaced
- The previous file repeated stamped lines that named Epic, Cerner, Allscripts, Veeva, and generic FHIR/HL7 connectors without explaining the SDK generation boundary.
- It did not distinguish SDK concerns from REST contract, AsyncAPI event, proto internal surface, catalog registration, SLO gating, or DPIA evidence.
- It did not name Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, or Health Catalyst.
- It did not specify generated-client behavior for PHI redaction, break-glass escalation, patient-match ambiguity, consent freshness, or provenance sealing.
- It did not bind generated client errors to Cedar deny evidence, audit-chain emission, or retry safety.
- It did not show how SDK generation displaces integration-engine lock-in while preserving regulated healthcare controls.

## Scope
- Generate SDK clients from microservices/healthcare-integration/contracts/openapi-v1.yaml for synchronous commands and reads.
- Generate event producer and consumer helpers from microservices/healthcare-integration/contracts/asyncapi-v1.yaml.
- Generate internal test harness stubs from microservices/healthcare-integration/contracts/healthcare-integration-v1.proto where synchronous internal calls need proto parity.
- Generate language targets only when the contract, policy, audit, and SLO evidence for that target can be kept in sync.
- Initial language targets are TypeScript for platform apps, Python for data migration operators, and Go for backend service integration.
- Each language target must expose the same operation names, error taxonomy, audit metadata, and policy evidence envelope.
- Each language target must be built from checked-in contracts, not from live service discovery.
- SDK generation includes examples, typed fixtures, and contract conformance tests.
- SDK generation excludes vendor connector implementation code; adapters remain in adapter and worker layers.
- SDK generation excludes PHI sample payloads; examples use redacted deterministic fixtures.
- SDK generation excludes credential storage; credentials are resolved through the sidecar binding path.
- SDK generation excludes direct database access, message broker shortcuts, or policy bypass flags.

## Capability bindings
- fhir-read binds generated read helpers to data_class=fhir_resource and purpose-scoped access.
- fhir-read clients must make _include, _revinclude, bulk export, and search parameter usage explicit in request builders.
- fhir-read clients must surface consent freshness before returning decoded resources.
- hl7-route binds generated route helpers to data_class=hl7_message and source-system scoped ACK handling.
- hl7-route clients must expose HL7 v2 ACK, NACK, retry, dead-letter, and replay handles without hiding queue state.
- break-glass-authorize binds generated emergency access helpers to data_class=break_glass_event.
- break-glass-authorize clients must require declared emergency reason, supervisor path, expiration, and post-event review marker.
- consent-sync binds generated consent helpers to clinical_consent state transitions and stale-consent refusal evidence.
- consent-sync clients must expose last-known consent source, current projection revision, and remediation workflow id.
- ehr-provenance-seal binds generated seal helpers to hash, signature, source system, transform id, and audit-chain pointer.
- ehr-provenance-seal clients must treat unsealed record movement as an error, not a warning.
- patient-match-review binds generated match helpers to ambiguous identity evidence and human review workflow links.
- patient-match-review clients must expose match confidence, blocked merge reason, steward assignment, and rollback bundle id.

## Generated client contract
- Every generated operation requires tenant_id.
- Every generated operation requires principal_id.
- Every generated operation requires audience_type.
- Every generated operation requires purpose.
- Every generated operation requires data_class.
- Every mutation requires idempotency_key.
- Every mutation requires traceparent or generated trace context.
- Every mutation returns audit_event_id when accepted.
- Every mutation returns policy_decision_id when evaluated.
- Every operation returns request_fingerprint for replay evidence.
- Every operation returns contract_version.
- Every operation returns sdk_generation_version.
- Every operation returns cell_id and residency_pack when available.
- Every error response carries stable code, human-safe summary, operator detail, retry class, and evidence pointers.
- Deny responses include Cedar fragment id, policy version, matched principal, matched resource, action, context hash, and redaction status.
- DealSet responses include settlement obligation id, provider network reference, billing classification, and hold status when relevant.
- PHI-bearing fields are redacted by default in logs and examples.
- Debug mode cannot print PHI payloads; it can print redacted shape, schema path, and hash.
- Generated clients must not add optional bypass arguments.
- Generated clients must not silently downgrade transport security.
- Generated clients must not auto-merge patient identities.
- Generated clients must not auto-approve break-glass review.
- Generated clients must not auto-resolve consent conflicts.
- Generated clients must not hide failed provenance sealing.

## Generator architecture
- Generator input is the checked-in OpenAPI, AsyncAPI, proto, capability YAML, and ADR binding list.
- Generator metadata records source file digests for every generated package.
- Generator output is deterministic by contract version and language target.
- Generator templates are owned by platform-sdk, but healthcare-integration supplies capability-specific policy and evidence overlays.
- Operation grouping follows capability names, not vendor names.
- The generated fhir namespace groups resource reads, search, export, provenance, consent, and tenant scope helpers.
- The generated hl7 namespace groups route, ACK, replay, transformation, queue, and source-system helpers.
- The generated emergency namespace groups break-glass authorization, post-access review, expiration, and supervisor workflows.
- The generated consent namespace groups sync, conflict, stale projection, pack overlay, and remediation helpers.
- The generated identity namespace groups patient-match review, merge block, duplicate evidence, and steward workflows.
- The generated audit namespace groups event lookup, evidence export, and trace correlation helpers.
- AsyncAPI helpers expose typed event builders and typed event readers for accepted, denied, replayed, sealed, consent-conflicted, and match-review-created events.
- Proto helpers expose internal fixture builders for service-to-service conformance tests.
- Package publishing must include SBOM, provenance, contract hash, and generation command manifest.
- Package publishing must not require a live healthcare endpoint.
- Package publishing must fail when local contract references are missing.

## Benchmark displacement
- Redox displacement: generated clients expose tenant, purpose, Cedar deny evidence, audit-chain ids, and DealSet settlement in the call contract instead of leaving integration behavior as external network middleware.
- Rhapsody displacement: generated clients keep route, transform, ACK, and replay behavior visible to product operators instead of burying them in engine-specific channels.
- InterSystems IRIS for Health displacement: generated clients preserve FHIR and HL7 access without requiring callers to adopt a database-centered interoperability runtime.
- Lyniate/Corepoint displacement: generated clients encode clinical routing and provenance as Oyatie contract primitives instead of locked point-to-point interface definitions.
- Mirth Connect displacement: generated clients provide typed tenant-scoped operation envelopes instead of script-channel conventions that are hard to audit across tenants.
- NextGate displacement: generated clients expose patient-match ambiguity and steward workflow state instead of making identity resolution a black-box service dependency.
- Health Catalyst displacement: generated clients provide governed extraction and evidence export without shifting ownership to analytics warehouse flows.
- Redox parity is insufficient unless the SDK proves Cedar-deny traceability and settlement evidence per request.
- Rhapsody parity is insufficient unless SDK consumers can rehearse route failure and replay without the engine UI.
- InterSystems parity is insufficient unless resource access is contract-first and independent of a proprietary persistence substrate.
- Lyniate/Corepoint parity is insufficient unless route definitions are reconstructable from repository contracts.
- Mirth parity is insufficient unless script-like transforms are replaced by typed, reviewed, replayable operations.
- NextGate parity is insufficient unless patient matching can be challenged, reviewed, and rolled back through generated helpers.
- Health Catalyst parity is insufficient unless analytics extraction remains a downstream consumer, not the clinical control plane.

## Implementation steps
- Step 1: Freeze contract inputs from openapi-v1.yaml, asyncapi-v1.yaml, healthcare-integration-v1.proto, and six capability YAML records.
- Step 2: Add generation metadata fields for contract_version, sdk_generation_version, source_digest, generated_at, and binding_adrs.
- Step 3: Generate TypeScript client shapes with required tenant, principal, audience, purpose, data_class, idempotency, trace, and audit fields.
- Step 4: Generate Python client shapes with the same operation envelopes and redaction defaults.
- Step 5: Generate Go client shapes with context propagation and explicit retry classes.
- Step 6: Generate event helper builders for accepted, denied, replayed, consent conflict, provenance gap, patient-match duplicate, and break-glass review events.
- Step 7: Generate conformance fixtures from proto shapes for internal service tests.
- Step 8: Add denied-response models that reference Cedar policy ids and local policy fragment paths.
- Step 9: Add transport options that default to strict TLS and document HTTP/3 fallback semantics.
- Step 10: Add credential-sidecar resolver interfaces that accept references, not secrets.
- Step 11: Add DealSet settlement metadata to billable provider-network operations.
- Step 12: Add pack overlay helpers for HIPAA-2024, SOC-2, ISO-27001, GDPR, KR-Medical-Devices, EU-MDR, and GxP.
- Step 13: Add generated examples using deterministic redacted patient and encounter fixtures.
- Step 14: Add generated negative examples for stale consent, missing tenant, missing purpose, unsealed provenance, and ambiguous patient match.
- Step 15: Add package publishing checks that fail on missing contract hash or ADR binding metadata.
- Step 16: Add SDK changelog entries tied to contract version changes.
- Step 17: Add verification evidence proving no generated file contains fixture PHI.
- Step 18: Add review gate for generated operation names so vendor names do not leak into the API surface.

## Tests and evidence
- Contract test: every OpenAPI operation appears in TypeScript, Python, and Go clients.
- Contract test: every AsyncAPI message appears in event helper builders.
- Contract test: every proto service shape used by internal sync calls has a fixture builder.
- Policy test: missing tenant_id fails before network dispatch.
- Policy test: missing principal_id fails before network dispatch.
- Policy test: missing purpose fails before network dispatch.
- Policy test: missing data_class fails before network dispatch.
- Policy test: missing idempotency_key fails for mutations.
- Policy test: denied Cedar responses retain policy_decision_id and redacted context hash.
- Audit test: accepted mutations expose audit_event_id.
- Audit test: replay helpers emit original_request_fingerprint.
- Redaction test: debug logging masks patient name, MRN, encounter id, insurance id, and free-text note fields.
- Transport test: HTTP/3 fallback is explicit and recorded in client telemetry.
- Transport test: strict TLS downgrade cannot be requested by application code.
- Credential test: sidecar reference is accepted and raw credential material is rejected.
- DealSet test: provider-network movement returns settlement obligation evidence.
- Capability test: break-glass helper requires reason, expiration, and post-access review destination.
- Capability test: patient-match helper refuses auto-merge without steward workflow id.
- Capability test: consent-sync helper refuses stale projection without remediation workflow id.
- Capability test: provenance seal helper returns failed seal as hard error.
- Benchmark test: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst are named in generated benchmark-displacement evidence.

## Rollback
- Roll back by restoring the prior SDK package version while keeping service contract versions unchanged.
- Roll back generated clients without rolling back OpenAPI, AsyncAPI, or proto contracts unless contract verification identifies the contract as the source.
- Roll back one language target independently when generation defects are language-specific.
- Block rollback that would remove tenant, purpose, Cedar, audit, or DealSet fields from generated operations.
- Preserve package provenance and contract digests for reverted versions.
- Emit rollback evidence to the healthcare-integration audit-chain event family.
- Notify owners of fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, and patient-match-review when generated clients are reverted.
- Keep stale generated packages unpublished when package signing fails.
- Keep local fixtures available for regression reproduction.
- Do not use rollback to reintroduce vendor-named wrappers.

## Evidence packet fields
- SDK evidence must record generator input contract hashes.
- SDK evidence must record capability YAML hashes.
- SDK evidence must record Binding ADR list.
- SDK evidence must record language target.
- SDK evidence must record package version.
- SDK evidence must record signing identity.
- SDK evidence must record SBOM reference.
- SDK evidence must record PHI redaction attestation.
- SDK evidence must record Cedar denial fixture coverage.
- SDK evidence must record transport fallback fixture coverage.
- SDK evidence must record credential-sidecar no-secret attestation.
- SDK evidence must record benchmark displacement proof rows.

## Acceptance criteria
- Each generated client operation includes tenant_id, principal_id, audience_type, purpose, data_class, trace context, contract_version, and sdk_generation_version.
- Every mutation includes idempotency_key and returns audit_event_id or deny evidence.
- Every language target exposes the same capability namespaces and error taxonomy.
- SDK generation cites the current Binding ADR set used by healthcare-integration IPs.
- SDK generation cites PRD.md, sdk-plan.md, contracts, and six capability records as source authorities.
- Generated examples are PHI-redacted and deterministic.
- Generated clients displace Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst on governed B2B control-plane evidence.
- No generated client introduces a suite folder, vendor-owned boundary, raw credential store, policy bypass, or automatic patient identity merge.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-019-sdk-client-generation.md:9` - Repo-local authorities: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/sdk-plan.md; microservices/healthcare-integration/contracts/o...; `microservices/healthcare-integration/IP-019-sdk-client-generation.md:38` - - Generate SDK clients from microservices/healthcare-integration/contracts/openapi-v1.yaml for synchronous commands and reads..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `ISO27001-2022` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-019-sdk-client-generation.md:24` - - Use ADR-0296 to bind credential-sidecar references without placing PHI-bearing secrets in generated clients.; `microservices/healthcare-integration/IP-019-sdk-client-generation.md:31` - - It did not distinguish SDK concerns from REST contract, AsyncAPI event, proto internal surface, catalog registration, SLO gating, or DPIA evidence..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-019-sdk-client-generation.md:34` - - It did not bind generated client errors to Cedar deny evidence, audit-chain emission, or retry safety..
