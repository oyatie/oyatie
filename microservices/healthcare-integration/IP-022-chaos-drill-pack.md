# IP-022 Healthcare Integration Chaos Drill Pack

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-022-chaos-drill-pack.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Owner teams: axis-healthcare-integration + reliability-engineering + security-operations
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local authorities: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/failure-modes.md; microservices/healthcare-integration/incident-response.md; microservices/healthcare-integration/runbooks/local-fhir-bundle-failure.md; microservices/healthcare-integration/runbooks/local-hl7-ack-latency-burn.md; microservices/healthcare-integration/runbooks/local-consent-sync-lag.md; microservices/healthcare-integration/runbooks/local-ehr-provenance-gap.md; microservices/healthcare-integration/runbooks/local-breakglass-audit-review.md; microservices/healthcare-integration/runbooks/local-patient-match-duplicate.md; microservices/healthcare-integration/dashboards/local-operator-remediation.json
Benchmark displacement set: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Build a healthcare-specific chaos drill pack that proves the service fails safely under clinical interoperability stress.
- Exercise FHIR, HL7, consent, break-glass, provenance, patient matching, audit, DealSet, credential, transport, and abuse-defence failure modes.
- Convert failure-modes.md from passive documentation into drill definitions with actors, injectors, expected alerts, runbooks, audit evidence, and rollback criteria.
- Preserve tenant scope and Cedar authorization during every drill.
- Preserve PHI redaction during every drill.
- Preserve audit-chain evidence during every drill.
- Preserve rollback evidence during every drill.
- Use ADR-0263 to require telemetry from every injected failure.
- Use ADR-0294 to connect drills to soak anomaly and rollback decisions.
- Use ADR-0297 to include spoof, scrape, bot, and replay abuse scenarios.
- Use ADR-0253-amendment to include transport downgrade, HTTP/3 fallback, ECH, and PQC negotiation scenarios.
- Use ADR-0296 to include credential-sidecar outage and rotation scenarios.
- Use ADR-0314 to include DealSet settlement hold scenarios.
- Use ADR-0321 to prove B2B leader healthcare coverage through operational drills, not vendor parity statements.

## Current thinness being replaced
- The previous file repeated stamped lines and did not define any drill.
- It did not name injectors, controls, expected telemetry, operator actions, safety limits, or rollback states.
- It did not cite failure-modes.md, incident-response.md, local runbooks, dashboards, SLOs, policies, or capability records.
- It did not distinguish clinical failure from generic service outage.
- It did not address PHI redaction, break-glass review, patient-match ambiguity, consent staleness, or EHR provenance gaps.
- It did not displace Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, or Health Catalyst.

## Drill pack scope
- Drill pack includes local deterministic drills.
- Drill pack includes canary-safe production drills with tenant allowlists.
- Drill pack includes shadow replay drills using redacted fixtures.
- Drill pack includes tabletop drills for break-glass and regulator evidence.
- Drill pack includes synthetic traffic drills for FHIR read paths.
- Drill pack includes queue drills for HL7 route paths.
- Drill pack includes consent drift drills.
- Drill pack includes provenance seal failure drills.
- Drill pack includes patient-match duplicate drills.
- Drill pack includes credential-sidecar failure drills.
- Drill pack includes DealSet settlement hold drills.
- Drill pack includes transport fallback drills.
- Drill pack includes abuse-defence drills.
- Drill pack includes observability loss drills.
- Drill pack excludes destructive drills that can expose PHI.
- Drill pack excludes drills that bypass Cedar policy.
- Drill pack excludes drills that require vendor-console-only evidence.
- Drill pack excludes live patient identity merges.
- Drill pack excludes unbounded replay traffic.

## Drill prerequisites
- A named tenant allowlist is required.
- A named cell allowlist is required.
- A named operator and incident commander are required.
- A named rollback owner is required.
- A named clinical steward is required for patient-match drills.
- A named privacy reviewer is required for DPIA-adjacent drills.
- A named security reviewer is required for abuse-defence drills.
- Cedar policy versions must be pinned.
- OpenAPI, AsyncAPI, and proto versions must be pinned.
- SLO thresholds must be recorded before injection.
- Dashboard panels must be green before injection.
- Runbook links must be reachable before injection.
- Audit-chain sink must be healthy before injection.
- Credential sidecar health must be known before injection.
- DealSet settlement state must be known before injection.
- PHI redaction check must pass before injection.
- Rollback rehearsal must be current before production canary drills.
- Stakeholder notification template must be prepared.

## Drill catalog
- Drill FHIR-001 injects FHIR bundle validation failure.
- Drill FHIR-002 injects FHIR upstream timeout.
- Drill FHIR-003 injects search parameter explosion.
- Drill FHIR-004 injects stale consent for a FHIR read.
- Drill HL7-001 injects ACK latency burn.
- Drill HL7-002 injects NACK storm.
- Drill HL7-003 injects dead-letter replay exhaustion.
- Drill HL7-004 injects malformed segment redaction failure attempt.
- Drill CONSENT-001 injects consent-sync lag.
- Drill CONSENT-002 injects conflicting consent source revisions.
- Drill CONSENT-003 injects pack overlay mismatch.
- Drill BREAKGLASS-001 injects emergency access without reason.
- Drill BREAKGLASS-002 injects expired emergency access.
- Drill BREAKGLASS-003 injects overdue post-access review.
- Drill PROVENANCE-001 injects missing source hash.
- Drill PROVENANCE-002 injects seal signature mismatch.
- Drill PROVENANCE-003 injects audit-chain write failure after seal attempt.
- Drill MATCH-001 injects duplicate patient candidate ambiguity.
- Drill MATCH-002 injects low-confidence identity merge attempt.
- Drill MATCH-003 injects steward workflow outage.
- Drill CRED-001 injects credential sidecar timeout.
- Drill CRED-002 injects credential rotation race.
- Drill DEALSET-001 injects settlement hold for provider-network movement.
- Drill DEALSET-002 injects billing-class mismatch.
- Drill TRANSPORT-001 injects HTTP/3 fallback churn.
- Drill TRANSPORT-002 injects TLS downgrade refusal.
- Drill ABUSE-001 injects replayed request fingerprints.
- Drill ABUSE-002 injects spoofed source-system identity.
- Drill ABUSE-003 injects scrape-like FHIR query burst.
- Drill OBS-001 injects missing metric dimensions.
- Drill OBS-002 injects delayed audit event emission.

## Expected evidence per drill
- Drill id.
- Tenant id.
- Cell id.
- Capability name.
- Data class.
- Contract version.
- Policy version.
- Cedar decision id when a policy path is exercised.
- Audit event id.
- Trace id.
- Metric panel reference.
- Log sample reference with PHI redacted.
- Runbook reference.
- Operator action timestamp.
- Rollback decision.
- Customer-impact classification.
- Privacy-impact classification.
- Security-impact classification.
- DealSet state when applicable.
- Credential-sidecar state when applicable.
- Transport fallback state when applicable.
- Benchmark displacement note.

## Capability drill expectations
- fhir-read drills must prove failed reads do not leak PHI in logs.
- fhir-read drills must prove consent freshness is checked before payload return.
- fhir-read drills must prove bundle validation errors are typed and actionable.
- hl7-route drills must prove ACK latency alerts route to local-hl7-ack-latency-burn.md.
- hl7-route drills must prove dead-letter replay is bounded and tenant-scoped.
- hl7-route drills must prove malformed messages preserve source-system and redaction evidence.
- break-glass-authorize drills must prove missing reason is denied by policy.
- break-glass-authorize drills must prove expired emergency access cannot continue silently.
- break-glass-authorize drills must prove overdue reviews appear in audit dashboards.
- consent-sync drills must prove stale consent blocks FHIR and PHI delivery.
- consent-sync drills must prove conflicting sources require remediation workflow.
- ehr-provenance-seal drills must prove unsealed movement is a hard error.
- ehr-provenance-seal drills must prove hash mismatch creates evidence, not silent correction.
- patient-match-review drills must prove ambiguous identity does not auto-merge.
- patient-match-review drills must prove steward outage blocks high-risk match promotion.

## Benchmark displacement
- Redox displacement: drills are repo-owned and tenant-scoped instead of depending on managed connector incident reports.
- Rhapsody displacement: drills expose route, ACK, replay, and operator remediation without relying on engine-console screenshots.
- InterSystems IRIS for Health displacement: drills separate platform health from FHIR, HL7, consent, provenance, and audit control health.
- Lyniate/Corepoint displacement: drills test interface behavior against ADR-bound contracts rather than point-to-point configuration alone.
- Mirth Connect displacement: drills replace channel-script checks with typed failure injectors and audit evidence.
- NextGate displacement: drills treat patient identity ambiguity as a governed workflow failure with SLO and rollback evidence.
- Health Catalyst displacement: drills prove analytics ingestion cannot override clinical consent or provenance failures.
- Redox-like incident summaries are insufficient unless local audit-chain ids exist.
- Rhapsody-like channel state is insufficient unless rollback was rehearsed.
- InterSystems-like platform monitoring is insufficient unless data-class dimensions are visible.
- Lyniate/Corepoint-like interface tests are insufficient unless tenant isolation is proven.
- Mirth-like script simulation is insufficient unless PHI redaction is verified.
- NextGate-like match confidence is insufficient unless human review behavior is drilled.
- Health Catalyst-like pipeline freshness is insufficient unless clinical gate failures block extraction.

## Implementation steps
- Step 1: Map failure-modes.md scenarios to drill ids.
- Step 2: Map each drill to a capability record.
- Step 3: Map each drill to one SLO or dashboard panel.
- Step 4: Map each drill to one local runbook.
- Step 5: Define allowed tenants, cells, and data classes for local drills.
- Step 6: Define canary-safe production drill boundaries.
- Step 7: Define abort thresholds for latency, error rate, consent staleness, audit lag, and queue backlog.
- Step 8: Define PHI redaction assertions for every drill log.
- Step 9: Define Cedar decision assertions for authorization drills.
- Step 10: Define audit-chain assertions for every drill.
- Step 11: Define DealSet assertions for provider-network drills.
- Step 12: Define credential-sidecar assertions for credential drills.
- Step 13: Define transport assertions for HTTP/3 and TLS drills.
- Step 14: Define abuse-defence assertions for spoof, scrape, bot, and replay drills.
- Step 15: Define operator notification and handoff records.
- Step 16: Define rollback rehearsal and completion evidence.
- Step 17: Define post-drill review fields and owner assignment.
- Step 18: Define benchmark displacement evidence rows.
- Step 19: Verify all references are repo-local.
- Step 20: Reject any drill that requires PHI exposure or policy bypass.

## Tests and evidence
- Drill test: every drill has capability, data_class, tenant, cell, and runbook fields.
- Drill test: every drill has expected metric, trace, log, and audit event evidence.
- Drill test: every drill has abort criteria.
- Drill test: every drill has rollback criteria.
- Drill test: PHI redaction assertion exists for every drill.
- Drill test: Cedar assertion exists for authorization-sensitive drills.
- Drill test: consent assertion exists for FHIR and consent-sync drills.
- Drill test: provenance assertion exists for EHR seal drills.
- Drill test: steward assertion exists for patient-match drills.
- Drill test: DealSet assertion exists for provider-network drills.
- Drill test: credential assertion exists for credential-sidecar drills.
- Drill test: transport assertion exists for HTTP/3 and TLS drills.
- Drill test: abuse-defence assertion exists for replay, spoof, scrape, and bot drills.
- Drill test: dashboard reference exists for every production-eligible drill.
- Drill test: incident-response.md is cited for escalation.
- Drill test: benchmark displacement names all seven required competitors.
- Drill test: no drill requires vendor-console-only evidence.
- Drill test: no drill bypasses Cedar policy.
- Drill test: no drill exposes live PHI.

## Rollback
- Abort a drill immediately when PHI redaction fails.
- Abort a drill immediately when tenant isolation is unclear.
- Abort a drill immediately when audit-chain emission is unavailable.
- Abort a drill immediately when rollback owner is unavailable.
- Abort a drill immediately when canary blast radius exceeds tenant allowlist.
- Abort a drill immediately when Cedar policy cannot evaluate.
- Abort a drill immediately when a break-glass workflow cannot be reviewed.
- Roll back injected configuration before declaring drill complete.
- Preserve failed drill artifacts for review.
- Preserve operator notes and timestamps.
- Preserve dashboard snapshots.
- Preserve audit event ids.
- Preserve runbook deviations.
- Do not retry production drills until the failed control is fixed and reverified.

## Acceptance criteria
- Chaos drill pack defines healthcare-specific drills for FHIR, HL7, consent, break-glass, provenance, patient matching, credentials, DealSet, transport, abuse, and observability.
- Every drill has repo-local evidence references and safety boundaries.
- Every drill preserves tenant scope, Cedar policy, PHI redaction, audit-chain evidence, and rollback.
- Benchmark displacement covers Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst.
- The plan keeps current Binding ADR references and does not modify ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-022-chaos-drill-pack.md:17` - - Preserve PHI redaction during every drill.; `microservices/healthcare-integration/IP-022-chaos-drill-pack.md:31` - - It did not cite failure-modes.md, incident-response.md, local runbooks, dashboards, SLOs, policies, or capability records..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-022-chaos-drill-pack.md:108` - - Drill OBS-002 injects delayed audit event emission.; `microservices/healthcare-integration/IP-022-chaos-drill-pack.md:213` - - Abort a drill immediately when audit-chain emission is unavailable..
