# IP-021 Healthcare Integration SLO-Gated Promotion

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-021-slo-gated-promotion.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Owner teams: axis-healthcare-integration + reliability-engineering + council-risk
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local authorities: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/slos/local-fhir-bundle-success.openslo.yaml; microservices/healthcare-integration/slos/local-hl7-ack-latency.openslo.yaml; microservices/healthcare-integration/slos/local-consent-sync-freshness.openslo.yaml; microservices/healthcare-integration/slos/local-phi-delivery-latency.openslo.yaml; microservices/healthcare-integration/slos/local-audit-completeness.openslo.yaml; microservices/healthcare-integration/slos/local-hipaa-access-review-latency.openslo.yaml; microservices/healthcare-integration/dashboards/local-slo-burn.json; microservices/healthcare-integration/dashboards/slo-and-error-budget.json; microservices/healthcare-integration/dashboards/operating-bar-overview.json
Benchmark displacement set: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Make promotion of healthcare-integration changes contingent on SLO evidence, not only contract generation or checklist completion.
- Protect clinical interoperability from shipping when FHIR bundle success, HL7 ACK latency, consent freshness, PHI delivery latency, audit completeness, or break-glass review latency are outside their allowed windows.
- Bind SLO promotion to ADR-0263 observability emission so every gate has metric, trace, log, and audit-chain proof.
- Bind SLO promotion to ADR-0243 Cedar evaluation so authorization-deny spikes block promotion when they indicate policy drift.
- Bind SLO promotion to ADR-0294 soak and anomaly rollback so staged releases can stop before clinical blast radius widens.
- Bind SLO promotion to ADR-0253-amendment so HTTP/3 fallback or TLS negotiation regressions are treated as reliability risk.
- Bind SLO promotion to ADR-0314 so DealSet settlement failures block provider-network movement.
- Bind SLO promotion to ADR-0321 so B2B leader coverage is measured by governed operational evidence instead of benchmark name-dropping.
- Bind SLO promotion to ADR-0131 so gates live inside the flat microservice rather than an external integration-engine release board.
- Treat SLO promotion as a release invariant for fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, and patient-match-review.

## Current thinness being replaced
- The previous file repeated stamped capability lines and did not identify actual SLO documents.
- It did not distinguish latency, freshness, completeness, authorization, and settlement gates.
- It did not explain release stages, burn-rate thresholds, hold states, or rollback triggers.
- It did not tie gates to dashboards, runbooks, policy fragments, or audit evidence.
- It did not mention Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, or Health Catalyst.
- It did not explain why Oyatie promotion evidence is stronger than vendor uptime claims.

## Promotion scope
- Gate every service change that modifies healthcare-integration contracts.
- Gate every service change that modifies healthcare-integration policies.
- Gate every service change that modifies healthcare-integration SLOs.
- Gate every service change that modifies healthcare-integration runbooks.
- Gate every service change that modifies healthcare-integration dashboards.
- Gate every service change that modifies capability records.
- Gate every service change that modifies SDK generation behavior.
- Gate every service change that modifies catalog registration.
- Gate every service change that modifies backfill, replay, import, export, or provider-network settlement behavior.
- Gate every service change that touches PHI-bearing data classes.
- Gate every service change that touches break-glass authorization.
- Gate every service change that touches patient matching.
- Gate every service change that touches provenance sealing.
- Exclude prose-only typo fixes that do not change controls, contracts, SLOs, policies, or evidence paths.
- Exclude local-only exploration branches until they request promotion.

## Gate families
- Contract gate verifies OpenAPI, AsyncAPI, and proto compatibility before SLO burn is evaluated.
- Policy gate verifies Cedar default-deny and local policy fragments before staged rollout.
- Auth gate verifies permit, deny, and refusal telemetry shape.
- FHIR gate verifies local-fhir-bundle-success SLO.
- HL7 gate verifies local-hl7-ack-latency SLO.
- Consent gate verifies local-consent-sync-freshness SLO.
- PHI delivery gate verifies local-phi-delivery-latency SLO.
- Audit gate verifies local-audit-completeness SLO.
- Break-glass gate verifies local-hipaa-access-review-latency SLO.
- Transport gate verifies HTTP/3 fallback and strict TLS telemetry.
- DealSet gate verifies settlement hold and obligation telemetry.
- Credential gate verifies sidecar reference resolution and secret redaction.
- Abuse-defence gate verifies spoof, scrape, bot, and replay signals.
- Dashboard gate verifies local-slo-burn and operating-bar dashboard rows.
- Runbook gate verifies remediation paths exist for current failure modes.
- Rollback gate verifies release can revert without orphaning audit-chain evidence.

## Stage model
- Stage 0 is contract and policy static verification.
- Stage 1 is local deterministic fixture verification.
- Stage 2 is shadow replay against redacted clinical fixtures.
- Stage 3 is single-tenant canary in a non-production cell.
- Stage 4 is single-tenant production canary with low PHI volume.
- Stage 5 is multi-tenant limited release by region and data class.
- Stage 6 is default release for eligible tenants.
- Stage 7 is post-promotion soak with burn-rate monitoring.
- A stage cannot advance when any gate is unknown.
- A stage cannot advance when any required dashboard has stale data.
- A stage cannot advance when audit completeness is below threshold.
- A stage cannot advance when Cedar deny spikes exceed configured anomaly limits.
- A stage cannot advance when break-glass review latency misses target.
- A stage cannot advance when consent freshness is stale.
- A stage cannot advance when patient-match duplicate review backlog exceeds agreed limit.
- A stage cannot advance when DealSet settlement holds are unreviewed.
- A stage cannot advance when HTTP/3 fallback causes unclassified transport errors.
- A stage cannot advance when any rollback rehearsal fails.

## Capability SLO gates
- fhir-read promotion requires FHIR bundle success evidence.
- fhir-read promotion requires read latency evidence from local and service-level SLOs.
- fhir-read promotion requires consent freshness evidence before PHI-bearing resources return.
- fhir-read promotion requires audit emission evidence for accepted and denied reads.
- hl7-route promotion requires ACK latency evidence.
- hl7-route promotion requires dead-letter rate evidence.
- hl7-route promotion requires replay freshness evidence.
- hl7-route promotion requires route backlog evidence.
- break-glass-authorize promotion requires review-latency evidence.
- break-glass-authorize promotion requires post-access audit completeness.
- break-glass-authorize promotion requires emergency reason and expiration evidence.
- consent-sync promotion requires freshness evidence.
- consent-sync promotion requires conflict-rate evidence.
- consent-sync promotion requires stale-projection refusal evidence.
- ehr-provenance-seal promotion requires seal success evidence.
- ehr-provenance-seal promotion requires hash/signature/audit-chain correlation evidence.
- ehr-provenance-seal promotion requires provenance-gap runbook readiness.
- patient-match-review promotion requires duplicate review latency evidence.
- patient-match-review promotion requires false-positive and false-negative review sample evidence.
- patient-match-review promotion requires rollback bundle evidence before merge-adjacent flows proceed.

## Burn-rate and hold rules
- Burn-rate windows must include fast burn, slow burn, and soak burn.
- Fast burn blocks current stage immediately.
- Slow burn blocks next-stage promotion.
- Soak burn blocks default release but can allow bounded canary continuation when audit completeness remains green.
- Unknown metric state is a hold, not a pass.
- Missing dashboard row is a hold.
- Missing runbook reference is a hold.
- Missing audit-chain evidence is a hold.
- Missing Cedar decision id is a hold.
- Missing tenant dimension is a hold.
- Missing data_class dimension is a hold.
- Missing cell_id dimension is a hold.
- Missing capability dimension is a hold.
- DealSet settlement backlog above threshold is a hold.
- Credential-sidecar error above threshold is a hold.
- Consent conflict backlog above threshold is a hold.
- Patient-match review queue above threshold is a hold.
- Break-glass review overdue count above threshold is a hold.
- Provenance seal failure above threshold is a hold.
- HL7 NACK replay exhaustion is a hold.
- FHIR bulk export timeout without classified retry is a hold.

## Benchmark displacement
- Redox displacement: promotion is based on tenant-scoped SLO and policy evidence, not connector availability claims.
- Rhapsody displacement: route health and replay readiness are explicit promotion gates instead of engine-channel status screens.
- InterSystems IRIS for Health displacement: platform uptime is not enough; FHIR, HL7, audit, consent, and tenant evidence must pass independently.
- Lyniate/Corepoint displacement: interface change promotion requires Oyatie contracts, runbooks, and SLO burn evidence instead of interface-engine deployment approval alone.
- Mirth displacement: channel success does not equal promotion success unless audit completeness and replay safety are proven.
- NextGate displacement: patient identity quality and review backlog are promotion blockers, not downstream data-quality tasks.
- Health Catalyst displacement: analytics ingestion success cannot promote clinical integration unless consent, provenance, and PHI delivery SLOs are green.
- Redox-like managed reliability is insufficient without local audit and Cedar evidence.
- Rhapsody-like operational consoles are insufficient without repo-local gate definitions.
- InterSystems-like platform health is insufficient without per-capability gates.
- Lyniate/Corepoint-like interface deployment is insufficient without rollback and tenant-scoped burn evidence.
- Mirth-like channel testing is insufficient without PHI redaction and policy telemetry.
- NextGate-like identity confidence is insufficient without human review SLOs.
- Health Catalyst-like pipeline freshness is insufficient without clinical provenance seal evidence.

## Implementation steps
- Step 1: Enumerate all healthcare-integration SLO files used by current capabilities.
- Step 2: Map each SLO to one or more capability records.
- Step 3: Map each SLO to one dashboard panel in local-slo-burn.json or slo-and-error-budget.json.
- Step 4: Map each SLO to one remediation runbook.
- Step 5: Define stage advancement criteria for contract, policy, fixture, canary, limited release, default release, and soak.
- Step 6: Define hold states for missing metric, stale metric, failed query, threshold breach, and rollback rehearsal failure.
- Step 7: Add Cedar deny anomaly inputs to promotion gates.
- Step 8: Add consent freshness inputs to fhir-read and consent-sync gates.
- Step 9: Add HL7 ACK and dead-letter inputs to hl7-route gates.
- Step 10: Add break-glass review latency inputs to emergency access gates.
- Step 11: Add provenance seal success inputs to EHR provenance gates.
- Step 12: Add patient-match review backlog inputs to patient identity gates.
- Step 13: Add DealSet settlement hold inputs to provider-network gates.
- Step 14: Add transport fallback telemetry inputs to HTTP/3 and TLS gates.
- Step 15: Add abuse-defence anomaly inputs for bot, spoof, scrape, and replay protection.
- Step 16: Add audit completeness as a global promotion gate.
- Step 17: Add stage-specific rollback rehearsal evidence.
- Step 18: Add benchmark displacement evidence rows for the seven required competitors.
- Step 19: Verify every gate cites a local repo authority.
- Step 20: Verify no promotion gate depends on vendor-console-only evidence.

## Tests and evidence
- SLO test: every listed local OpenSLO file is referenced by a promotion gate.
- SLO test: every capability has at least one SLO gate.
- SLO test: audit completeness is global across capability promotions.
- SLO test: unknown metric state blocks promotion.
- SLO test: stale dashboard state blocks promotion.
- SLO test: fast burn blocks current stage.
- SLO test: slow burn blocks next stage.
- SLO test: soak burn blocks default release.
- Policy test: Cedar deny anomaly blocks promotion when threshold is exceeded.
- Policy test: missing policy_decision_id blocks promotion.
- Tenant test: missing tenant_id dimension blocks promotion.
- PHI test: missing data_class dimension blocks promotion.
- DealSet test: settlement hold backlog blocks provider-network promotion.
- Transport test: unclassified HTTP/3 fallback errors block promotion.
- Break-glass test: overdue access review blocks promotion.
- Consent test: stale consent projection blocks promotion.
- Provenance test: seal failures block promotion.
- Patient-match test: duplicate review backlog blocks promotion.
- Benchmark test: all seven named competitors are present in displacement evidence.
- Repository test: every gate cites a local SLO, dashboard, runbook, policy, contract, PRD, or capability file.

## Rollback
- Roll back the promoted change, not the SLO definition, when the gate correctly catches a regression.
- Roll back the SLO definition only when the threshold or query is proven incorrect.
- Preserve failed-gate evidence for post-incident review.
- Keep audit-chain events for stage entry, hold, rollback, and release.
- Reopen canary only after the failed gate has fresh green evidence.
- Do not bypass audit completeness for emergency release.
- Do not bypass Cedar deny anomaly gates for customer pressure.
- Do not bypass consent freshness for backfill throughput.
- Do not bypass break-glass review latency for incident convenience.
- Do not bypass patient-match review backlog for migration speed.
- Do not bypass DealSet holds for revenue pressure.
- Do not use vendor uptime pages as rollback evidence.

## Acceptance criteria
- SLO-gated promotion names and uses local SLO, dashboard, runbook, policy, contract, PRD, and capability references.
- Every capability has explicit gate behavior.
- Unknown, stale, missing, or breached evidence blocks promotion.
- Promotion stages are ordered and rollback-aware.
- Benchmark displacement covers Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst.
- The plan preserves current Binding ADR references and does not change ADR-0321.
- The plan rejects vendor-console-only promotion proof and requires repo-local Oyatie evidence.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-021-slo-gated-promotion.md:1` - # IP-021 Healthcare Integration SLO-Gated Promotion; `microservices/healthcare-integration/IP-021-slo-gated-promotion.md:13` - - Make promotion of healthcare-integration changes contingent on SLO evidence, not only contract generation or checklist completion..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-021-slo-gated-promotion.md:15` - - Bind SLO promotion to ADR-0263 observability emission so every gate has metric, trace, log, and audit-chain proof.; `microservices/healthcare-integration/IP-021-slo-gated-promotion.md:91` - - fhir-read promotion requires audit emission evidence for accepted and denied reads..
