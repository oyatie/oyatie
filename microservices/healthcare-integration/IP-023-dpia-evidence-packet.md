# IP-023 Healthcare Integration DPIA Evidence Packet

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-023-dpia-evidence-packet.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Owner teams: axis-healthcare-integration + privacy-engineering + council-risk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local authorities: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/dpia.md; microservices/healthcare-integration/compliance.md; microservices/healthcare-integration/policy/data-residency.md; microservices/healthcare-integration/policies/local-fhir-exchange-consent.cedar; microservices/healthcare-integration/policies/local-patient-consent-sync.cedar; microservices/healthcare-integration/policies/local-hipaa-audit-completeness.cedar; microservices/healthcare-integration/runbooks/local-clinical-export-redaction.md; microservices/healthcare-integration/runbooks/local-hipaa-access-review-delay.md; microservices/healthcare-integration/dashboards/compliance-pack-health.json
Benchmark displacement set: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Build a DPIA evidence packet for healthcare-integration that privacy, security, clinical operations, and auditors can inspect without vendor-console dependency.
- Prove that FHIR reads, HL7 routing, consent sync, break-glass authorization, provenance sealing, and patient-match review have explicit privacy controls.
- Treat DPIA as operational evidence bound to contracts, policies, SLOs, dashboards, runbooks, capability records, and ADRs.
- Preserve ADR-0131 flat microservice ownership and avoid a privacy-suite folder.
- Preserve ADR-0242 and ADR-0244 tenant doctrine by making tenant scope a DPIA evidence dimension.
- Preserve ADR-0243 Cedar authorization by including permit and deny evidence.
- Preserve ADR-0246 and ADR-0257 library-first policy and ontology paths by documenting network-opt-in privacy implications.
- Preserve ADR-0258 versioning by tying evidence to contract versions.
- Preserve ADR-0263 observability by requiring metric, trace, log, and audit evidence.
- Preserve ADR-0296 credential-sidecar controls by excluding raw credentials from the packet.
- Preserve ADR-0297 abuse-defence by documenting replay, spoof, scrape, and bot privacy risks.
- Preserve ADR-0314 DealSet settlement obligations when clinical provider-network movement has commercial and privacy implications.
- Preserve ADR-0321 B2B leader coverage by making healthcare benchmarks privacy-comparable and evidence-bound.

## Current thinness being replaced
- The previous file repeated stamped capability lines and did not describe a DPIA packet.
- It did not list data categories, processing purposes, lawful basis analogues, minimization controls, retention controls, residency controls, or data-subject/regulator evidence.
- It did not bind to dpia.md, compliance.md, data-residency.md, local Cedar policies, export redaction runbooks, or compliance dashboards.
- It did not distinguish FHIR PHI, HL7 messages, clinical consent, break-glass events, provenance seals, and patient-match evidence.
- It did not explain how Oyatie displaces Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, or Health Catalyst for privacy evidence.
- It did not state what evidence blocks release.

## Packet scope
- Packet covers PHI and regulated clinical data processed by healthcare-integration.
- Packet covers fhir_resource data class.
- Packet covers hl7_message data class.
- Packet covers clinical_consent data class.
- Packet covers break_glass_event data class.
- Packet covers provenance_seal evidence.
- Packet covers patient_match_evidence.
- Packet covers source-system identifiers.
- Packet covers tenant, principal, audience, purpose, and data_class dimensions.
- Packet covers import, export, read, route, replay, sync, seal, review, and rollback flows.
- Packet covers HIPAA-2024 pack evidence.
- Packet covers SOC-2 pack evidence.
- Packet covers ISO-27001 pack evidence.
- Packet covers GDPR pack evidence.
- Packet covers KR-Medical-Devices pack evidence.
- Packet covers EU-MDR pack evidence.
- Packet covers GxP pack evidence.
- Packet excludes raw PHI samples.
- Packet excludes raw secrets.
- Packet excludes vendor screenshots as primary evidence.
- Packet excludes clinical decision-making claims outside interoperability controls.

## Evidence inventory
- PRD evidence proves the product problem, target users, functional requirements, non-functional requirements, compliance impact, and out-of-scope boundaries.
- DPIA evidence proves data categories, processing flows, privacy risks, mitigations, residual risk, and review cadence.
- Compliance evidence proves pack overlays and regulatory control mapping.
- Data-residency evidence proves region, cell, residency pack, and export restrictions.
- OpenAPI evidence proves externally callable operations and required fields.
- AsyncAPI evidence proves emitted and consumed event shapes.
- Proto evidence proves internal service shape where synchronous calls exist.
- Capability YAML evidence proves tenant required fields and binding ADRs.
- Cedar policy evidence proves default-deny, consent, break-glass, audit completeness, and PHI delivery controls.
- SLO evidence proves latency, freshness, completeness, and review timeliness.
- Dashboard evidence proves operational visibility.
- Runbook evidence proves remediation and regulator-response procedures.
- Audit-chain evidence proves accepted, denied, replayed, sealed, exported, and reviewed transitions.
- SDK evidence proves client surfaces enforce tenant, purpose, and redaction defaults.
- Catalog evidence proves service ownership and layer accountability.

## Data categories
- Patient demographic identifiers are regulated and require minimization.
- Medical record identifiers are regulated and require hashing or redaction outside payload handling.
- Encounter identifiers are regulated and require purpose-bound access.
- FHIR resources are PHI-bearing unless explicitly redacted.
- HL7 messages are PHI-bearing and source-system sensitive.
- Clinical consent records are regulatory control data and may themselves be sensitive.
- Break-glass events are sensitive access-control records.
- Provenance seals are integrity evidence and can reveal source-system topology.
- Patient-match evidence is identity-risk data and must not be exposed broadly.
- Referral data is regulated clinical workflow data.
- Audit event ids are control evidence and must be retained.
- Policy decision ids are control evidence and must be retained.
- Trace ids are operational evidence and must avoid PHI-bearing attributes.
- DealSet settlement ids are commercial evidence and can reveal provider-network relationships.
- Credential-sidecar references are security metadata and must not contain secrets.
- Source-system ids can be sensitive when linked to providers.
- Pack overlay ids are compliance control evidence.
- Cell and residency ids are operational and regulatory evidence.

## Processing purposes
- FHIR read purpose is clinical interoperability.
- HL7 route purpose is clinical message delivery.
- Consent sync purpose is authorization and privacy-state propagation.
- Break-glass purpose is emergency access under review.
- Provenance seal purpose is record integrity and auditability.
- Patient-match review purpose is identity ambiguity resolution.
- Import purpose is tenant-approved migration or source synchronization.
- Export purpose is regulated clinical exchange or customer-controlled portability.
- Replay purpose is recovery, audit, and migration correction.
- Redaction purpose is privacy-preserving evidence export.
- Audit purpose is accountability and regulatory inspection.
- DealSet purpose is marketplace settlement for provider-network obligations.
- Abuse-defence purpose is protecting PHI and regulated endpoints.
- Credential resolution purpose is secure source-system access.

## Control mapping
- Tenant isolation maps to ADR-0242, ADR-0244, capability tenantScope, and every API request envelope.
- Cedar default-deny maps to ADR-0243 and local policy fragments.
- Consent enforcement maps to local-fhir-exchange-consent.cedar and local-patient-consent-sync.cedar.
- Audit completeness maps to local-hipaa-audit-completeness.cedar and local-audit-completeness.openslo.yaml.
- Break-glass review maps to local-breakglass-access-control.cedar and local-hipaa-access-review-latency.openslo.yaml.
- PHI delivery control maps to local-phi-delivery-authorization.cedar and local-phi-delivery-latency.openslo.yaml.
- Export redaction maps to runbooks/local-clinical-export-redaction.md.
- Residency control maps to policy/data-residency.md and pack overlays.
- Credential protection maps to ADR-0296 and sidecar references.
- Abuse protection maps to ADR-0297 and abuse-defence dashboards.
- Provenance integrity maps to ehr-provenance-seal capability and provenance gap runbooks.
- Patient identity protection maps to patient-match-review capability and duplicate review runbooks.
- Marketplace settlement privacy maps to ADR-0314 and DealSet hold evidence.
- Transport privacy maps to ADR-0253-amendment.
- Observability minimization maps to ADR-0263 and redacted telemetry requirements.

## Packet assembly rules
- Evidence packet must include a table of data classes.
- Evidence packet must include processing purpose per data class.
- Evidence packet must include lawful-basis analogue or customer-control basis per purpose.
- Evidence packet must include minimization control per data class.
- Evidence packet must include retention control per data class.
- Evidence packet must include residency control per data class.
- Evidence packet must include access control per data class.
- Evidence packet must include audit evidence per data class.
- Evidence packet must include export and deletion behavior per data class where supported.
- Evidence packet must include break-glass exception handling.
- Evidence packet must include consent conflict handling.
- Evidence packet must include patient-match ambiguity handling.
- Evidence packet must include provenance failure handling.
- Evidence packet must include abuse-defence privacy risks.
- Evidence packet must include credential handling.
- Evidence packet must include DealSet settlement privacy note.
- Evidence packet must include residual risk.
- Evidence packet must include owner and review cadence.
- Evidence packet must include benchmark displacement.
- Evidence packet must include no-PHI-fixture attestation.

## Benchmark displacement
- Redox displacement: DPIA evidence is repo-local, tenant-scoped, policy-bound, and exportable instead of dependent on managed-network attestations.
- Rhapsody displacement: privacy evidence covers route behavior, replay, and operator action rather than engine configuration alone.
- InterSystems IRIS for Health displacement: privacy evidence is not bound to a database/runtime posture; it follows each capability and data class.
- Lyniate/Corepoint displacement: interface privacy risk is controlled through ADR-bound contracts and local policies, not point-to-point interface documentation.
- Mirth Connect displacement: script/channel privacy behavior is replaced by typed controls, redaction tests, and audit-chain evidence.
- NextGate displacement: patient identity privacy is explicitly governed through review, ambiguity, and rollback evidence.
- Health Catalyst displacement: analytics privacy evidence is downstream; clinical consent and provenance remain the control source.
- Redox parity is insufficient unless Oyatie can export Cedar, tenant, consent, and audit evidence per request.
- Rhapsody parity is insufficient unless replay and transformation privacy are packetized.
- InterSystems parity is insufficient unless persistence is decoupled from DPIA control ownership.
- Lyniate/Corepoint parity is insufficient unless each interface maps to a tenant and data class.
- Mirth parity is insufficient unless scripts cannot leak PHI in logs.
- NextGate parity is insufficient unless identity-match evidence has access restrictions.
- Health Catalyst parity is insufficient unless extraction cannot bypass consent and provenance seals.

## Implementation steps
- Step 1: Enumerate data classes from PRD.md and capability YAML files.
- Step 2: Map each data class to processing purposes.
- Step 3: Map each processing purpose to contract operations and events.
- Step 4: Map each data class to Cedar controls.
- Step 5: Map each data class to SLO and dashboard evidence.
- Step 6: Map each data class to retention and residency controls.
- Step 7: Map each data class to redaction behavior.
- Step 8: Map each data class to export and replay behavior.
- Step 9: Map each data class to audit-chain events.
- Step 10: Map break-glass flows to review and expiration evidence.
- Step 11: Map consent conflicts to remediation runbooks.
- Step 12: Map patient-match ambiguity to steward workflows.
- Step 13: Map provenance failures to seal gap runbooks.
- Step 14: Map credential-sidecar behavior to no-secret evidence.
- Step 15: Map DealSet settlement to commercial privacy risk notes.
- Step 16: Map abuse-defence risks to replay, spoof, scrape, and bot controls.
- Step 17: Add residual risk and owner review cadence.
- Step 18: Add benchmark displacement evidence for seven competitors.
- Step 19: Verify all packet references are repo-local.
- Step 20: Verify no raw PHI appears in examples or evidence.

## Tests and evidence
- DPIA test: every capability has a data-class entry.
- DPIA test: every data class has purpose, minimization, retention, residency, access, and audit controls.
- DPIA test: every PHI-bearing class has redaction evidence.
- DPIA test: every authorization-sensitive class has Cedar evidence.
- DPIA test: consent enforcement cites local consent policies.
- DPIA test: audit completeness cites local audit policy and SLO evidence.
- DPIA test: break-glass review cites local access-control policy and review runbook.
- DPIA test: export redaction cites local-clinical-export-redaction.md.
- DPIA test: residency evidence cites policy/data-residency.md.
- DPIA test: credential evidence contains references, not secrets.
- DPIA test: DealSet evidence contains settlement ids, not PHI payloads.
- DPIA test: patient-match evidence is access-restricted.
- DPIA test: provenance evidence includes seal and audit-chain correlation.
- DPIA test: benchmark displacement names all seven required competitors.
- DPIA test: no vendor screenshot is primary evidence.
- DPIA test: no raw PHI fixture is present.
- DPIA test: residual risk is recorded with owner and review date.
- DPIA test: packet cites current Binding ADRs.

## Rollback
- Roll back packet publication when raw PHI is detected.
- Roll back packet publication when raw secrets are detected.
- Roll back packet publication when a data class lacks purpose mapping.
- Roll back packet publication when a PHI-bearing class lacks redaction evidence.
- Roll back packet publication when Cedar evidence is missing for controlled access.
- Roll back packet publication when audit completeness evidence is missing.
- Roll back packet publication when residency evidence is missing.
- Roll back packet publication when break-glass review evidence is missing.
- Roll back packet publication when patient-match evidence access is unclear.
- Roll back packet publication when DealSet evidence leaks commercial or PHI context beyond allowed fields.
- Preserve failed packet artifact hashes for privacy review.
- Preserve reviewer comments and remediation owners.
- Do not roll back underlying policies or contracts unless they are proven incorrect.

## Acceptance criteria
- DPIA evidence packet covers all healthcare-integration capabilities and data classes.
- Packet cites PRD.md, dpia.md, compliance.md, data-residency.md, local policies, runbooks, dashboards, and current Binding ADRs.
- Packet proves tenant scope, Cedar control, redaction, residency, retention, audit, credential, DealSet, and residual-risk handling.
- Packet displaces Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst through repo-owned privacy evidence.
- Packet contains no raw PHI, raw secrets, vendor-console-only primary proof, suite ownership, or ADR-0321 edits.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `ISO27001-2022` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-023-dpia-evidence-packet.md:15` - - Treat DPIA as operational evidence bound to contracts, policies, SLOs, dashboards, runbooks, capability records, and ADRs.; `microservices/healthcare-integration/IP-023-dpia-evidence-packet.md:31` - - It did not distinguish FHIR PHI, HL7 messages, clinical consent, break-glass events, provenance seals, and patient-match evidence..
