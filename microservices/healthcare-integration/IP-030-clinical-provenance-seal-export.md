# IP-030 Healthcare Integration clinical provenance seal export

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-030-clinical-provenance-seal-export.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Status: authoring-ready
Owner: axis-healthcare-integration
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml; microservices/healthcare-integration/runbooks/local-ehr-provenance-gap.md; microservices/healthcare-integration/runbooks/ehr-provenance-gap.md; microservices/healthcare-integration/runbooks/local-clinical-export-redaction.md; microservices/healthcare-integration/runbooks/clinical-export-redaction.md; microservices/healthcare-integration/contracts/openapi-v1.yaml; microservices/healthcare-integration/contracts/asyncapi-v1.yaml; microservices/healthcare-integration/dashboards/local-audit-completeness.json; microservices/healthcare-integration/slos/local-fhir-bundle-success.openslo.yaml
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Define an atomic implementation plan for sealed clinical provenance export.
- Make exported FHIR, HL7, consent, break-glass, and MPI evidence regulator-ready.
- Bind export to Cedar, tenant scope, audit-chain evidence, consent segmentation, and route custody.
- Preserve source provenance without exposing raw secrets or unredacted PHI outside authorized scope.
- Ensure seal verification is independent of vendor consoles.
- Attach DealSet settlement status where vendor exchange is commercial.
- Keep this IP documentation-only and limited to the assigned file.

## Export packet fields
- Field 001: tenant_id identifies the owner tenant.
- Field 002: principal_id identifies the export requester.
- Field 003: audience_type identifies auditor, regulator, tenant admin, healthcare operator, or approved worker.
- Field 004: purpose identifies audit, regulator-export, patient-access, incident-review, or remediation.
- Field 005: data_class identifies provenance_seal, fhir_resource, hl7_message, clinical_consent, or break_glass_event.
- Field 006: export_packet_id uniquely identifies the packet.
- Field 007: export_version identifies contract version under ADR-0258.
- Field 008: seal_algorithm identifies signing and hash algorithm.
- Field 009: seal_key_ref identifies sidecar-held signing key reference class.
- Field 010: source_system_id identifies the clinical source.
- Field 011: route_custody_id identifies HL7 ACK custody evidence.
- Field 012: consent_record_id identifies consent evidence.
- Field 013: break_glass_review_id identifies emergency access evidence.
- Field 014: patient_match_case_id identifies MPI adjudication evidence.
- Field 015: fhir_bundle_job_id identifies segmented bundle evidence.
- Field 016: redaction_manifest_id identifies redaction rules and output.
- Field 017: audit_event_ids identify ADR-0263 chain entries.
- Field 018: jurisdiction_code identifies residency and regulator overlay.
- Field 019: home_cell identifies export production locality.
- Field 020: dealset_reference identifies settlement state where applicable.

## Seal assembly states
- State 001: requested means export was requested and scoped.
- State 002: policy-evaluated means Cedar decision exists.
- State 003: evidence-collected means route, consent, MPI, break-glass, FHIR, and audit references are attached.
- State 004: redaction-planned means rule set is selected.
- State 005: redaction-applied means output is filtered.
- State 006: canonicalized means packet is normalized for hashing.
- State 007: sealed means hash and signature are produced.
- State 008: verified means independent verification succeeded.
- State 009: delivery-pending means authorized recipient is ready.
- State 010: delivered means recipient received packet.
- State 011: rejected means policy, redaction, verification, or delivery failed.
- State 012: quarantined means packet cannot be delivered but must be retained.
- State 013: superseded means a newer packet replaces it.
- State 014: expired means delivery window closed.
- State 015: appealed means tenant or subject challenges export.
- State 016: regulator-acknowledged means regulator receipt evidence is attached.
- State 017: settlement-held means vendor settlement is blocked pending seal acceptance.
- State 018: settlement-released means seal evidence released hold.
- State 019: archived means retention controls apply.
- State 020: deleted is forbidden for sealed export evidence before retention expiry.

## Export policy gates
- Gate 001: deny when tenant_id is absent.
- Gate 002: deny when principal_id is absent.
- Gate 003: deny when audience_type is not authorized for export.
- Gate 004: deny when purpose is not audit, regulator-export, patient-access, incident-review, or remediation.
- Gate 005: deny when data_class is outside packet scope.
- Gate 006: deny when consent evidence is missing for ordinary patient-access export.
- Gate 007: deny when break-glass review is open and pack blocks export.
- Gate 008: deny when MPI adjudication is unresolved.
- Gate 009: deny when route custody evidence is missing for HL7-derived data.
- Gate 010: deny when FHIR redaction manifest is missing.
- Gate 011: deny when source credential sidecar cannot prove source context.
- Gate 012: deny when signing key reference is unavailable.
- Gate 013: deny when audit-chain evidence cannot be written.
- Gate 014: deny when jurisdiction_code conflicts with recipient.
- Gate 015: deny when export_version has passed sunset under ADR-0258.
- Gate 016: deny when ontology projection version has passed sunset under ADR-0257.
- Gate 017: deny when Cedar fragment soak rules are unmet under ADR-0294.
- Gate 018: deny when DealSet hold blocks vendor release.
- Gate 019: deny when verification fails.
- Gate 020: emit refusal evidence for every deny.

## Redaction and canonicalization rules
- Rule 001: redaction uses deterministic rules tied to redaction_manifest_id.
- Rule 002: redaction manifest includes rule ids, input classes, output classes, and pack overlays.
- Rule 003: canonicalization sorts resources, events, and attachments deterministically.
- Rule 004: canonicalization excludes volatile delivery metadata from seal hash.
- Rule 005: canonicalization includes export_version and schema ids.
- Rule 006: seal hash includes canonical packet content.
- Rule 007: seal signature uses sidecar-held signing key.
- Rule 008: verification can run without source-system credential.
- Rule 009: verification fails closed when a referenced audit event is missing.
- Rule 010: verification fails closed when redaction manifest hash mismatches.
- Rule 011: verification fails closed when route custody hash mismatches.
- Rule 012: verification fails closed when consent_version mismatches.
- Rule 013: verification fails closed when break-glass review state is not allowed.
- Rule 014: verification fails closed when MPI decision is not final.
- Rule 015: delivery manifest includes recipient, purpose, pack, jurisdiction, and expiry.
- Rule 016: delivery manifest excludes raw patient identifiers where not required.
- Rule 017: regulator export includes evidence index.
- Rule 018: patient-access export includes understandable denial and redaction explanation.
- Rule 019: vendor export includes DealSet hold or release state.
- Rule 020: replay export produces a new packet that links the prior packet.

## Provenance Seal Export Benchmark Displacement
- Displacement claim: this IP measures competitors against verifiable clinical provenance export, not route logs, analytics lineage, or generic audit exports.
- Non-generic rule: a vendor comparison must name seal packet inputs, canonicalization, signature custody, redaction manifest, recipient class, and verification behavior.
- Redox displacement: Redox exchange evidence is displaced by independently verifiable provenance seal packets.
- Redox proof: export includes route, consent, FHIR, MPI, and audit references.
- Rhapsody displacement: interface-engine message logs are displaced by canonicalized sealed export.
- Rhapsody proof: ACK custody hash and route_version are seal inputs.
- InterSystems IRIS for Health displacement: platform repository provenance is displaced by flat service packet assembly and ADR-traceable contracts.
- InterSystems proof: packet fields cite healthcare-integration contracts and capability records.
- Lyniate/Corepoint displacement: channel audit exports are displaced by signed packet verification and local redaction runbooks.
- Lyniate/Corepoint proof: clinical export redaction runbooks own remediation.
- Mirth displacement: script output archives are displaced by deterministic canonicalization and sidecar-held signatures.
- Mirth proof: seal verification does not depend on channel scripts.
- NextGate displacement: MPI lineage is included but cannot seal unresolved identity.
- NextGate proof: patient_match_case_id must be final before export.
- Health Catalyst displacement: analytic lineage is displaced by operational provenance covering route, consent, identity, emergency access, and audit.
- Health Catalyst proof: analytics-ready data without seal verification is rejected.

## Failure modes
- Failure 001: missing consent evidence blocks ordinary patient export.
- Failure 002: unresolved MPI case blocks export.
- Failure 003: open break-glass review blocks export where pack requires closure.
- Failure 004: missing route custody blocks HL7-derived export.
- Failure 005: redaction manifest mismatch blocks delivery.
- Failure 006: signing key unavailable blocks seal.
- Failure 007: audit-chain outage blocks mutation and delivery.
- Failure 008: recipient jurisdiction conflict blocks delivery.
- Failure 009: contract version sunset blocks export.
- Failure 010: ontology version sunset blocks projection.
- Failure 011: verification failure quarantines packet.
- Failure 012: DealSet hold blocks vendor release.
- Failure 013: regulator acknowledgement missing keeps packet delivery-pending.
- Failure 014: appeal opens but does not delete evidence.
- Failure 015: attempted deletion before retention expiry is forbidden.

## Capacity and performance
- Capacity 001: export packet assembly partitions by tenant, home_cell, pack, recipient, and data_class.
- Capacity 002: large FHIR bundles use streaming canonicalization.
- Capacity 003: seal jobs are idempotent by export_packet_id and canonical hash.
- Capacity 004: verification jobs are idempotent by seal hash.
- Capacity 005: packet delivery retries are separated from packet assembly.
- Capacity 006: regulator export queues have deadline-based priority.
- Capacity 007: patient-access export queues have tenant-visible progress.
- Capacity 008: vendor settlement release waits on verification but does not block unrelated tenants.
- Capacity 009: metrics use packet class and reason codes, not raw patient identifiers.
- Capacity 010: trace spans separate evidence collection, redaction, canonicalization, signing, verification, and delivery.
- Capacity 011: SLO for FHIR bundle success remains referenced for bundle input quality.
- Capacity 012: audit completeness dashboard tracks missing evidence counts.
- Capacity 013: seal key rotation creates new packets only when content changes or policy requires reseal.
- Capacity 014: replay export links previous packet without mutating it.
- Capacity 015: storage cost is estimated by packet byte size, attachment count, retention class, and pack.

## Observability
- Event `oya.healthcare.integration.provenance.export.requested` records request.
- Event `oya.healthcare.integration.provenance.export.redacted` records redaction.
- Event `oya.healthcare.integration.provenance.export.sealed` records seal creation.
- Event `oya.healthcare.integration.provenance.export.verified` records verification.
- Event `oya.healthcare.integration.provenance.export.delivered` records delivery.
- Metric `healthcare_integration_provenance_export_total` dimensions: status, purpose, pack, cell.
- Metric `healthcare_integration_provenance_verification_failure_total` dimensions: reason_code, pack, cell.
- Metric `healthcare_integration_provenance_packet_bytes` dimensions: packet_class, pack, cell.
- Trace span `healthcare.provenance.export.assemble` wraps evidence, redaction, canonicalization, signing, verification, and delivery.
- Log schema includes export_packet_id, seal_hash, redaction_manifest_id, decision_id, audit_event_id, and workflow_run_id.
- Dashboard reference: dashboards/local-audit-completeness.json.
- Runbook reference: runbooks/local-ehr-provenance-gap.md.
- Runbook reference: runbooks/ehr-provenance-gap.md.
- Runbook reference: runbooks/local-clinical-export-redaction.md.
- Runbook reference: runbooks/clinical-export-redaction.md.

## Implementation steps
- Step 001: Add export packet value object.
- Step 002: Add provenance seal aggregate.
- Step 003: Add evidence collector usecase.
- Step 004: Add deterministic redaction manifest model.
- Step 005: Add canonicalization worker.
- Step 006: Add sidecar signing adapter.
- Step 007: Add independent verification usecase.
- Step 008: Add delivery worker with recipient policy.
- Step 009: Add DealSet hold and release integration.
- Step 010: Add audit-chain events for request, redaction, seal, verify, deliver, reject, and quarantine.
- Step 011: Add OpenAPI examples for export request, verify, and download.
- Step 012: Add AsyncAPI events for export state transitions.
- Step 013: Add property tests for canonicalization determinism.
- Step 014: Add replay tests for superseded packet linkage.
- Step 015: Add benchmark displacement evidence to review packet.

## Tests and evidence
- Test 001: line count for this IP is at least 200.
- Test 002: ADR scan finds the full binding ADR list.
- Test 003: benchmark scan finds all seven named competitors.
- Test 004: local reference scan finds ehr-provenance-seal.yaml.
- Test 005: local reference scan finds provenance and redaction runbooks.
- Test 006: local reference scan finds contract references.
- Test 007: local reference scan finds local-audit-completeness.json.
- Test 008: review confirms unresolved MPI, consent, route custody, or break-glass review blocks export where required.
- Test 009: review confirms ADR-0321 was not edited.
- Test 010: review confirms no oya vcs verify, done, or promote was run.

## Rollback
- Rollback 001: disable new export_version for affected tenant only.
- Rollback 002: retain previous sealed packets.
- Rollback 003: quarantine failed packets instead of deleting them.
- Rollback 004: retain original audit event ids.
- Rollback 005: keep DealSet holds until verification passes.
- Rollback 006: restore prior Cedar fragment only after soak-window rules permit.
- Rollback 007: notify tenant admin when delivery status changes.
- Rollback 008: rerun canonicalization idempotently after fix.
- Rollback 009: create superseding packet rather than mutating previous packet.
- Rollback 010: export regulator remediation packet for affected exports.

## Acceptance criteria
- AC01: Every export packet carries tenant, principal, audience, purpose, data class, packet id, version, seal algorithm, key reference, and audit ids.
- AC02: Consent, route custody, MPI, FHIR, break-glass, and audit references are included when relevant.
- AC03: Redaction and canonicalization are deterministic.
- AC04: Seal verification can run independently of vendor consoles.
- AC05: Unresolved identity, consent, route, or emergency-review evidence blocks export where required.
- AC06: DealSet holds are respected for vendor release.
- AC07: Prior packets are superseded, never rewritten.
- AC08: Metrics avoid raw patient identifiers.
- AC09: All seven named benchmarks are explicitly displaced.
- AC10: This plan remains scoped to the assigned IP file.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-030-clinical-provenance-seal-export.md:10` - Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabili....
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-030-clinical-provenance-seal-export.md:17` - - Preserve source provenance without exposing raw secrets or unredacted PHI outside authorized scope.; `microservices/healthcare-integration/IP-030-clinical-provenance-seal-export.md:156` - - Capacity 011: SLO for FHIR bundle success remains referenced for bundle input quality..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-030-clinical-provenance-seal-export.md:160` - - Capacity 015: storage cost is estimated by packet byte size, attachment count, retention class, and pack..
