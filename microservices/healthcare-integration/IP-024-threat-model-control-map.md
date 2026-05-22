# IP-024 Healthcare Integration Threat Model Control Map

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-024-threat-model-control-map.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Owner teams: axis-healthcare-integration + security-architecture + council-risk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local authorities: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/threat-model.md; microservices/healthcare-integration/failure-modes.md; microservices/healthcare-integration/incident-response.md; microservices/healthcare-integration/policy/abuse-defence.cedar; microservices/healthcare-integration/policies/local-phi-delivery-authorization.cedar; microservices/healthcare-integration/policies/local-hl7-ingest-source-scope.cedar; microservices/healthcare-integration/policies/local-breakglass-access-control.cedar; microservices/healthcare-integration/iac/local-network-policy.yaml; microservices/healthcare-integration/iac/local-openbao-policy.hcl
Benchmark displacement set: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Convert healthcare-integration threat modeling into an explicit control map for clinical interoperability risks.
- Map each threat to assets, entry points, trust boundaries, abuse paths, preventive controls, detective controls, response controls, and evidence.
- Keep threat ownership under the flat healthcare-integration microservice and current Binding ADR set.
- Treat FHIR resources, HL7 messages, clinical consent, break-glass events, provenance seals, patient-match evidence, credentials, DealSet settlement, and audit-chain events as protected assets.
- Use ADR-0243 and local Cedar policies as primary authorization controls.
- Use ADR-0297 and abuse-defence.cedar for anti-bot, anti-spoof, anti-scrape, and replay controls.
- Use ADR-0296 and local OpenBao policy for credential-sidecar threat controls.
- Use ADR-0253-amendment and local network policy for transport and edge threat controls.
- Use ADR-0263 for detection evidence.
- Use ADR-0294 for anomaly rollback and soak controls.
- Use ADR-0314 for marketplace settlement threat controls.
- Use ADR-0321 to prove B2B healthcare leader coverage through threat displacement, not vendor-label parity.

## Current thinness being replaced
- The previous file repeated stamped capability lines without a threat model.
- It did not identify protected assets, actors, trust boundaries, attack paths, or control classes.
- It did not map local policies, IAC, runbooks, dashboards, contracts, or SLOs to threats.
- It did not address clinical-specific abuse such as consent bypass, HL7 source spoofing, break-glass misuse, patient-match poisoning, provenance stripping, or PHI scraping.
- It did not state benchmark displacement against Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, or Health Catalyst.
- It did not define acceptance evidence or rollback for control-map defects.

## Protected assets
- Asset A01: FHIR resource payloads.
- Asset A02: HL7 message payloads.
- Asset A03: Clinical consent state.
- Asset A04: Break-glass authorization records.
- Asset A05: EHR provenance seals.
- Asset A06: Patient-match evidence.
- Asset A07: Source-system identifiers.
- Asset A08: Tenant and principal identifiers.
- Asset A09: Audit-chain events.
- Asset A10: Cedar policy decisions.
- Asset A11: Credential-sidecar references.
- Asset A12: DealSet settlement obligations.
- Asset A13: Workflow run ids.
- Asset A14: Replay request fingerprints.
- Asset A15: Dashboard and SLO evidence.
- Asset A16: Export bundles.
- Asset A17: Redacted clinical fixtures.
- Asset A18: Pack overlay configuration.
- Asset A19: Network policy and ingress configuration.
- Asset A20: Catalog ownership records.

## Actors
- Actor R01: Tenant healthcare operator with legitimate access.
- Actor R02: Tenant administrator with pack activation rights.
- Actor R03: Clinical steward reviewing patient-match ambiguity.
- Actor R04: Emergency operator requesting break-glass access.
- Actor R05: Auditor inspecting evidence.
- Actor R06: Integration adapter service.
- Actor R07: Worker processing async import, route, replay, and export jobs.
- Actor R08: SDK client embedded in another Oyatie service.
- Actor R09: External EHR or source system.
- Actor R10: Malicious tenant user.
- Actor R11: Compromised source-system credential.
- Actor R12: Bot or scraper targeting FHIR search.
- Actor R13: Spoofed HL7 sender.
- Actor R14: Insider attempting unauthorized break-glass.
- Actor R15: Analytics consumer attempting consent bypass.
- Actor R16: Marketplace participant with settlement pressure.
- Actor R17: Operator under incident pressure.
- Actor R18: Supply-chain actor altering generated SDK or catalog metadata.

## Trust boundaries
- Boundary B01: Public REST and SDK ingress.
- Boundary B02: Async event ingress and egress.
- Boundary B03: Internal proto service boundary.
- Boundary B04: Cedar policy evaluation boundary.
- Boundary B05: Ontology projection boundary.
- Boundary B06: Credential-sidecar boundary.
- Boundary B07: EHR adapter boundary.
- Boundary B08: HL7 source-system boundary.
- Boundary B09: FHIR source-system boundary.
- Boundary B10: Worker queue boundary.
- Boundary B11: Audit-chain sink boundary.
- Boundary B12: Dashboard and observability boundary.
- Boundary B13: DealSet settlement boundary.
- Boundary B14: Export bundle boundary.
- Boundary B15: Break-glass workflow boundary.
- Boundary B16: Patient-match steward workflow boundary.
- Boundary B17: Pack overlay and residency boundary.
- Boundary B18: Network ingress and service mesh boundary.

## Threat scenarios
- T01: Missing tenant scope on FHIR read.
- T02: Principal spoofing through SDK or REST ingress.
- T03: HL7 source-system spoofing.
- T04: Replay of previously accepted PHI delivery request.
- T05: FHIR search scraping across high-cardinality parameters.
- T06: Consent stale read returns PHI.
- T07: Consent conflict is auto-resolved incorrectly.
- T08: Break-glass access is granted without emergency reason.
- T09: Break-glass access remains active after expiration.
- T10: Post-break-glass review is skipped.
- T11: Provenance seal is stripped during export.
- T12: Provenance seal mismatch is downgraded to warning.
- T13: Patient-match poisoning creates false merge pressure.
- T14: Patient-match ambiguity is auto-merged.
- T15: Credential-sidecar reference leaks raw secret.
- T16: Credential rotation race sends data to wrong source.
- T17: DealSet settlement pressure bypasses consent or audit.
- T18: Analytics extraction bypasses clinical control plane.
- T19: HTTP/3 fallback hides TLS downgrade.
- T20: PQC or ECH negotiation failure is unclassified.
- T21: Worker dead-letter replay leaks PHI to logs.
- T22: Audit-chain sink outage loses accountability.
- T23: Dashboard dimension loss hides tenant impact.
- T24: SDK generation supply-chain tampering removes required fields.
- T25: Catalog row misownership routes incident to wrong team.
- T26: Residency pack mismatch exports PHI across forbidden boundary.
- T27: Abuse-defence false negative allows scraping.
- T28: Abuse-defence false positive blocks emergency access without review path.
- T29: Adapter transform corrupts clinical meaning.
- T30: Runbook drift causes operator to use stale rollback path.

## Preventive controls
- C-P01: tenant_id required by contracts and SDKs.
- C-P02: principal_id required by contracts and SDKs.
- C-P03: audience_type and purpose required by capability records.
- C-P04: data_class required for PHI-bearing operations.
- C-P05: Cedar default-deny policy evaluation for controlled actions.
- C-P06: local-hl7-ingest-source-scope.cedar for source-system scoping.
- C-P07: local-fhir-exchange-consent.cedar for FHIR consent enforcement.
- C-P08: local-patient-consent-sync.cedar for consent state control.
- C-P09: local-breakglass-access-control.cedar for emergency access.
- C-P10: local-phi-delivery-authorization.cedar for PHI delivery.
- C-P11: abuse-defence.cedar for bot, spoof, scrape, and replay controls.
- C-P12: credential sidecar references instead of raw secrets.
- C-P13: local-openbao-policy.hcl for secret boundary policy.
- C-P14: local-network-policy.yaml for service network boundaries.
- C-P15: strict TLS, HTTP/3 fallback classification, ECH, and PQC controls.
- C-P16: provenance seal required before EHR movement.
- C-P17: patient-match human review before merge-adjacent actions.
- C-P18: DealSet settlement hold cannot bypass policy.
- C-P19: pack overlay residency enforcement.
- C-P20: generated SDK required-field checks.
- C-P21: catalog owner validation.
- C-P22: no raw PHI fixtures.
- C-P23: idempotency keys and replay fingerprints.
- C-P24: export redaction runbook gates.

## Detective controls
- C-D01: audit-chain event for accepted FHIR read.
- C-D02: audit-chain event for denied FHIR read.
- C-D03: audit-chain event for HL7 route accepted.
- C-D04: audit-chain event for HL7 route denied.
- C-D05: audit-chain event for consent conflict.
- C-D06: audit-chain event for break-glass requested.
- C-D07: audit-chain event for break-glass expired.
- C-D08: audit-chain event for post-access review.
- C-D09: audit-chain event for provenance seal failure.
- C-D10: audit-chain event for patient-match duplicate.
- C-D11: audit-chain event for export bundle generation.
- C-D12: local-slo-burn dashboard panels.
- C-D13: local-policy-decisions dashboard panels.
- C-D14: local-audit-completeness dashboard panels.
- C-D15: abuse-defence outcomes dashboard.
- C-D16: tenant cost and capacity dashboard dimensions.
- C-D17: trace ids for all controlled operations.
- C-D18: structured logs with PHI redaction.
- C-D19: SLO burn alerts.
- C-D20: anomaly rollback evidence per ADR-0294.

## Response controls
- C-R01: local-fhir-bundle-failure runbook.
- C-R02: local-hl7-ack-latency-burn runbook.
- C-R03: local-consent-sync-lag runbook.
- C-R04: local-breakglass-audit-review runbook.
- C-R05: local-ehr-provenance-gap runbook.
- C-R06: local-patient-match-duplicate runbook.
- C-R07: local-clinical-export-redaction runbook.
- C-R08: local-hipaa-access-review-delay runbook.
- C-R09: incident-response.md escalation.
- C-R10: rollback bundle export.
- C-R11: canary halt and stage rollback.
- C-R12: credential-sidecar rotation hold.
- C-R13: DealSet settlement hold.
- C-R14: privacy review hold.
- C-R15: security review hold.
- C-R16: clinical steward review hold.
- C-R17: catalog owner correction.
- C-R18: SDK package quarantine.
- C-R19: contract version freeze.
- C-R20: pack overlay lock.

## Threat-to-control map
- T01 maps to C-P01, C-P05, C-D02, C-R09.
- T02 maps to C-P02, C-P05, C-D13, C-R09.
- T03 maps to C-P06, C-D04, C-R02.
- T04 maps to C-P23, C-P11, C-D15, C-R11.
- T05 maps to C-P11, C-D15, C-R09.
- T06 maps to C-P07, C-P08, C-D05, C-R03.
- T07 maps to C-P08, C-D05, C-R03, C-R14.
- T08 maps to C-P09, C-D06, C-R04.
- T09 maps to C-P09, C-D07, C-R04.
- T10 maps to C-P09, C-D08, C-R08.
- T11 maps to C-P16, C-D09, C-R05.
- T12 maps to C-P16, C-D09, C-R05.
- T13 maps to C-P17, C-D10, C-R06.
- T14 maps to C-P17, C-D10, C-R06, C-R16.
- T15 maps to C-P12, C-P13, C-D18, C-R12.
- T16 maps to C-P12, C-P13, C-D17, C-R12.
- T17 maps to C-P18, C-D11, C-R13.
- T18 maps to C-P07, C-P16, C-D11, C-R14.
- T19 maps to C-P15, C-D17, C-R11.
- T20 maps to C-P15, C-D17, C-R15.
- T21 maps to C-P22, C-D18, C-R02.
- T22 maps to C-D01, C-D14, C-R09, C-R11.
- T23 maps to C-D12, C-D16, C-R09.
- T24 maps to C-P20, C-D17, C-R18.
- T25 maps to C-P21, C-D12, C-R17.
- T26 maps to C-P19, C-D11, C-R20.
- T27 maps to C-P11, C-D15, C-R15.
- T28 maps to C-P09, C-D15, C-R04.
- T29 maps to C-P16, C-D09, C-R05.
- T30 maps to C-R01, C-R02, C-R03, C-R09.

## Benchmark displacement
- Redox displacement: threat controls are local, tenant-scoped, Cedar-bound, and audit-exportable instead of delegated to a managed interoperability network.
- Rhapsody displacement: route and replay threats are mapped to repo-local controls instead of engine-channel operational knowledge.
- InterSystems IRIS for Health displacement: database/runtime controls are not treated as sufficient for consent, provenance, identity, and DealSet threats.
- Lyniate/Corepoint displacement: point-to-point interface threats are mapped to service-layer contracts, policies, and runbooks.
- Mirth Connect displacement: script/channel threat paths are replaced with typed controls, redacted logs, and replay fingerprints.
- NextGate displacement: patient identity threats are governed by steward review and rollback controls.
- Health Catalyst displacement: analytics pipeline threats cannot override clinical consent, provenance, and PHI delivery controls.
- Redox parity is insufficient without tenant-level deny evidence.
- Rhapsody parity is insufficient without replay and ACK controls outside an engine UI.
- InterSystems parity is insufficient without non-database control ownership.
- Lyniate/Corepoint parity is insufficient without ADR-bound row ownership.
- Mirth parity is insufficient without script-elimination and PHI redaction proof.
- NextGate parity is insufficient without human review and rollback proof.
- Health Catalyst parity is insufficient without extraction governance bound to clinical controls.

## Implementation steps
- Step 1: Enumerate protected assets from PRD.md, capability records, contracts, policies, and threat-model.md.
- Step 2: Enumerate actors and trust boundaries from service runtime paths.
- Step 3: Map threats to capabilities and data classes.
- Step 4: Map preventive controls to local policies, IAC, SDK rules, and contract fields.
- Step 5: Map detective controls to dashboards, SLOs, traces, logs, and audit-chain events.
- Step 6: Map response controls to runbooks and incident-response.md.
- Step 7: Map every threat to at least one preventive control.
- Step 8: Map every threat to at least one detective control.
- Step 9: Map every threat to at least one response control.
- Step 10: Mark threats with residual risk when any control is manual.
- Step 11: Mark threats requiring privacy review.
- Step 12: Mark threats requiring security review.
- Step 13: Mark threats requiring clinical steward review.
- Step 14: Mark threats requiring DealSet settlement review.
- Step 15: Mark threats requiring credential rotation review.
- Step 16: Add benchmark displacement evidence for seven competitors.
- Step 17: Verify all controls cite repo-local files.
- Step 18: Reject controls that depend only on vendor attestations.
- Step 19: Reject threat entries without tenant and data_class dimensions.
- Step 20: Emit control-map evidence for SLO-gated promotion and DPIA packet assembly.

## Tests and evidence
- Threat-map test: every threat has protected asset, actor, trust boundary, capability, and data class.
- Threat-map test: every threat has preventive, detective, and response controls.
- Threat-map test: every preventive control cites local policy, IAC, contract, SDK, or capability evidence.
- Threat-map test: every detective control cites local dashboard, SLO, trace, log, or audit evidence.
- Threat-map test: every response control cites a local runbook or incident-response.md.
- Threat-map test: every PHI threat includes redaction control.
- Threat-map test: every consent threat includes local consent policy evidence.
- Threat-map test: every break-glass threat includes review and expiration controls.
- Threat-map test: every patient-match threat includes steward review controls.
- Threat-map test: every credential threat includes sidecar and OpenBao controls.
- Threat-map test: every transport threat includes HTTP/3, strict TLS, ECH, or PQC classification.
- Threat-map test: every DealSet threat includes settlement hold behavior.
- Threat-map test: every abuse threat includes abuse-defence controls.
- Threat-map test: every audit threat includes audit-chain response behavior.
- Threat-map test: every benchmark displacement row names all seven required competitors.
- Threat-map test: no threat relies on vendor-console-only primary evidence.
- Threat-map test: current Binding ADR references are present.

## Rollback
- Roll back control-map publication when any threat lacks a preventive control.
- Roll back control-map publication when any threat lacks a detective control.
- Roll back control-map publication when any threat lacks a response control.
- Roll back control-map publication when a PHI threat lacks redaction.
- Roll back control-map publication when consent bypass has no Cedar mapping.
- Roll back control-map publication when break-glass misuse has no review mapping.
- Roll back control-map publication when patient-match poisoning lacks steward review.
- Roll back control-map publication when credential threats lack sidecar controls.
- Roll back control-map publication when transport threats lack classification.
- Roll back control-map publication when DealSet threats lack settlement hold controls.
- Preserve failed map evidence and reviewer notes.
- Do not roll back policies, contracts, or ADRs from this IP.
- Do not edit ADR-0321.

## Acceptance criteria
- Threat-model control map covers protected assets, actors, boundaries, threats, preventive controls, detective controls, response controls, evidence, and rollback.
- Map cites threat-model.md, failure-modes.md, incident-response.md, local policies, IAC, PRD.md, and current Binding ADRs.
- Map covers FHIR, HL7, consent, break-glass, provenance, patient match, credentials, DealSet, transport, audit, and abuse-defence threats.
- Map displaces Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst with repo-owned control evidence.
- Map contains no vendor-console-only primary evidence, no suite ownership, no policy bypass, and no ADR-0321 edit requirement.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-024-threat-model-control-map.md:29` - - It did not map local policies, IAC, runbooks, dashboards, contracts, or SLOs to threats.; `microservices/healthcare-integration/IP-024-threat-model-control-map.md:30` - - It did not address clinical-specific abuse such as consent bypass, HL7 source spoofing, break-glass misuse, patient-match poisoning, provenance stripping, or PHI scr....

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-024-threat-model-control-map.md:170` - - C-D16: tenant cost and capacity dashboard dimensions..
