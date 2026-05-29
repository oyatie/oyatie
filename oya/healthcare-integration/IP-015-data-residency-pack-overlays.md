# IP-015 Healthcare Integration Data Residency Pack Overlays

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-015-data-residency-pack-overlays.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Date: 2026-05-20
Owner: axis-healthcare-integration
Capability focus: residency-aware clinical interoperability
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Primary local citations:
- microservices/healthcare-integration/PRD.md
- microservices/healthcare-integration/ARCHITECTURE.md
- microservices/healthcare-integration/policy/data-residency.md
- microservices/healthcare-integration/compliance.md
- microservices/healthcare-integration/dpia.md
- microservices/healthcare-integration/multi-region.md
- microservices/healthcare-integration/capabilities/fhir-read.yaml
- microservices/healthcare-integration/capabilities/hl7-route.yaml
- microservices/healthcare-integration/capabilities/consent-sync.yaml
- microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml
- microservices/healthcare-integration/dashboards/compliance-pack-health.json
- microservices/healthcare-integration/runbooks/hipaa-pack-misconfiguration.md
- microservices/healthcare-integration/slos/local-audit-completeness.openslo.yaml
- docs/standards/documentation-rigor.md
- specs/root-hub-pointers.json
- specs/master-plan-sequencing.json

## 1. Executive Intent
- This IP makes residency a pack overlay, not a scattered adapter convention.
- Healthcare-integration handles PHI, clinical consent, HL7 messages, FHIR resources, provenance, and emergency exceptions.
- Every one of those surfaces needs a pack answer before data movement.
- The overlay model lets tenants activate HIPAA, SOC-2, ISO-27001, GDPR, KR-Medical-Devices, EU-MDR, and GxP constraints without changing the microservice boundary.
- The overlay model uses higher-restriction-wins when packs conflict.
- The overlay model treats region, cell, source system, data class, retention, export, breach timing, and review obligation as independent axes.
- It prevents a Redox-like connector abstraction from hiding residency movement.
- It prevents an interface-engine route from becoming a legal transfer by accident.
- It gives B2B leaders a concrete answer for multi-region healthcare integration.
- It follows ADR-0105 layering by keeping pack resolution outside provider adapters.
- It follows ADR-0243 and ADR-0244 by applying tenant policy before data movement.
- It follows ADR-0257 by projecting ontology reads through library-first controls.
- It follows ADR-0321 documentation depth without editing ADR-0321.

## 2. B2B Leader Problem
- Healthcare buyers ask where PHI lives before they ask how many connectors exist.
- Multinational tenants must run FHIR and HL7 flows across regions without illegal replication.
- Regional tenants need local emergency access that does not silently copy payloads to a global control plane.
- Marketplace providers need to know when a route, replay, export, or analytics handoff is forbidden by pack policy.
- SREs need failover behavior that respects residency even under outage pressure.
- Compliance teams need evidence for why a record stayed local, moved as metadata, or moved as payload.
- Product teams need one residency model across read, route, sync, replay, audit, and export.

## 3. Overlay Inputs
- `tenant_id` is required.
- `principal_id` is required.
- `audience_type` is required.
- `home_cell` is required.
- `jurisdiction_code` is required.
- `source_system_id` is required.
- `data_class` is required.
- `resource_type` is required.
- `workflow_id` is required for async work.
- `policy_decision_id` is required before movement.
- `active_pack_ids` are required.
- `requested_operation` is required.
- `destination_cell` is required when movement is requested.
- `export_purpose` is required when export is requested.
- `emergency_context` is required for emergency override.

## 4. Overlay Outputs
- `effective_residency_mode` can be local-only, metadata-only, payload-allowed, export-forbidden, or emergency-limited.
- `allowed_cells` lists cells where payload can be processed.
- `metadata_allowed_cells` lists cells where identifiers or metadata can move.
- `retention_rule_id` binds to pack retention.
- `breach_notification_rule_id` binds to pack timing.
- `regulator_export_rule_id` binds to pack evidence.
- `consent_interaction_rule_id` binds consent and residency conflict handling.
- `audit_retention_rule_id` binds audit-chain retention.
- `replay_scope_rule_id` binds backfill and replay.
- `settlement_visibility_rule_id` binds DealSet non-PHI disclosure.
- `operator_review_rule_id` binds review queue obligations.
- `deny_reason` is mandatory when movement is blocked.

## 5. Scope
- Define pack overlay resolution for FHIR reads.
- Define pack overlay resolution for HL7 routes.
- Define pack overlay resolution for consent sync.
- Define pack overlay resolution for EHR provenance seals.
- Define pack overlay resolution for emergency bypass.
- Define pack overlay resolution for backfill replay.
- Define pack overlay resolution for audit export.
- Define pack overlay resolution for settlement evidence.
- Define dashboard evidence for pack health.
- Define runbook response for pack misconfiguration.
- Define multi-region failover behavior.
- Define acceptance fixtures for conflicting packs.

## 6. Non-Goals
- Do not create a geography-specific service fork.
- Do not create vendor-specific residency exceptions.
- Do not duplicate compliance service ownership.
- Do not store pack policy in adapter code.
- Do not move PHI through the marketplace settlement layer.
- Do not weaken emergency policy.
- Do not allow global fallback during outage unless pack allows it.
- Do not edit ADR-0321.

## 7. Resolution Algorithm
- Load active tenant packs.
- Validate pack ids against compliance registry.
- Normalize region and jurisdiction codes.
- Normalize data class.
- Resolve operation class.
- Resolve source and destination cells.
- Apply local-only rules first.
- Apply emergency-limited rules second.
- Apply metadata-only rules third.
- Apply payload-allowed rules fourth.
- Apply export rules fifth.
- Apply retention rules sixth.
- Apply breach notification rules seventh.
- Apply regulator evidence rules eighth.
- Apply settlement visibility rules ninth.
- Apply reviewer obligations tenth.
- Select the strictest result for each axis.
- Emit a signed resolution id.
- Attach resolution id to every downstream command.
- Deny movement when resolution is missing.

## 8. Implementation Steps
- Add `ResidencyOverlayResolver` in application layer.
- Keep pack parsing outside domain aggregate mutation.
- Add `ResolvedResidencyOverlay` value object in kernel/domain.
- Add policy-port trait for pack lookup.
- Add local adapter for pack registry.
- Add test fixtures for HIPAA plus GDPR conflict.
- Add test fixtures for KR-Medical-Devices plus GxP conflict.
- Add test fixtures for emergency metadata-only failover.
- Add test fixtures for audit export allowed while payload export denied.
- Add OpenAPI examples carrying resolved overlay id.
- Add AsyncAPI examples for overlay-resolved and overlay-denied events.
- Add proto examples for internal overlay request and response.
- Add dashboard dimensions for pack id, rule id, jurisdiction, operation, and deny reason.
- Add runbook branch for pack misconfiguration.
- Add replay worker dependency on resolved overlay id.
- Add capacity admission dependency on residency-local queue capacity.
- Add cost budget dependency on residency surcharge eligibility.

## 9. Data Movement Matrix
- FHIR read from home cell to same cell can return payload if policy permits.
- FHIR read from home cell to non-home cell returns metadata only unless pack permits payload movement.
- HL7 route within same jurisdiction can send payload when source and destination systems are tenant-approved.
- HL7 route across jurisdiction requires explicit pack permission.
- Consent sync can move consent state only when consent state itself is not more restricted than the resource.
- Provenance seal metadata can move more broadly than payload only when pack allows metadata evidence.
- Emergency bypass can use metadata-only failover when payload movement is forbidden.
- Backfill replay must execute in payload-allowed cells.
- Audit export can include evidence references without payload when pack forbids clinical export.
- Settlement evidence can include non-PHI units and references only.

## 10. Benchmark Displacement
- Redox displacement: Redox abstracts network connectivity; this IP makes residency decisions explicit, signed, and tenant-pack aware before any exchange.
- Rhapsody displacement: Rhapsody routes messages across interfaces; this IP refuses route execution when pack overlays forbid the destination cell.
- InterSystems IRIS for Health displacement: IRIS can centralize data; this IP favors flat microservice cell controls and metadata-only behavior instead of implicit centralization.
- Lyniate/Corepoint displacement: Corepoint projects can encode residency in interface rules; this IP promotes residency to a reusable pack resolver with audit evidence.
- Mirth displacement: Mirth channels can script custom residency behavior; this IP avoids ad hoc scripts and uses typed overlay outputs.
- NextGate displacement: NextGate identity matching can span jurisdictions; this IP requires patient-match work to inherit the most restrictive payload and metadata rule.
- Health Catalyst displacement: Health Catalyst data platforms can feed analytics; this IP blocks analytics or export when operational residency does not permit it.
- Combined displacement: competitors treat residency as configuration, route discipline, or deployment topology; this IP treats it as signed runtime evidence.

## 11. Emergency and Disaster Behavior
- Disaster failover must not ignore active packs.
- If payload cannot move, return metadata-only triage evidence.
- If metadata cannot move, return denial with contact path.
- Emergency responders receive minimum necessary payload only when pack permits.
- Emergency review inherits stricter retention and export rules.
- Regional outage creates degraded evidence.
- Degraded evidence includes the pack decision id.
- Degraded evidence includes cells considered and rejected.
- Degraded evidence includes reviewer assignment.
- Degraded evidence includes expiry and revocation path.

## 12. Backfill and Replay Behavior
- Replay uses the pack active at original event time for historical reconstruction.
- Replay uses current pack for new output movement.
- Replay must record both original and current overlay ids.
- Replay denies rows that would newly violate residency.
- Replay produces adjustment evidence for settlement.
- Replay can run in tenant home cell only when payload rules require it.
- Replay can fan out metadata-only progress outside the home cell when allowed.
- Replay retries must not change the overlay decision without a new policy id.
- Replay DLQ includes row id, rule id, data class, source system, and deny reason.
- Replay acceptance requires audit evidence completeness.

## 13. Observability
- Dashboard shows overlay permit and deny counts.
- Dashboard shows top deny reasons.
- Dashboard shows pack conflict rate.
- Dashboard shows metadata-only fallback count.
- Dashboard shows emergency-limited count.
- Dashboard shows payload export denied count.
- Dashboard shows audit export allowed count.
- SLO tracks local audit completeness.
- Alert fires when pack resolver is unavailable.
- Alert fires when pack conflict exceeds baseline.
- Alert fires when emergency-limited decisions are overdue for review.
- Alert fires when route denials spike after pack activation.

## 14. Failure Modes
- Missing pack id denies movement.
- Unknown jurisdiction denies movement.
- Unknown data class denies movement.
- Pack conflict uses higher-restriction-wins.
- Resolver outage denies payload movement.
- Compliance registry mismatch opens incident response.
- Source-system region mismatch denies route.
- Destination cell capacity shortage queues work locally.
- Audit-chain outage pauses export and high-risk movement.
- Marketplace settlement receives non-PHI evidence only.

## 15. Rollback
- Roll back a bad overlay by pinning the prior resolved pack version.
- Re-evaluate in-flight requests.
- Revoke movements authorized by the bad version.
- Emit overlay rollback events.
- Rebuild audit export packets.
- Re-run affected replay rows.
- Hold affected settlement records.
- Notify tenant compliance owners.
- Use pack misconfiguration runbook.
- Preserve both bad and restored decisions for regulator review.

## 16. Acceptance Evidence
- The IP cites `policy/data-residency.md`.
- The IP cites compliance and DPIA docs.
- The IP cites multi-region behavior.
- The IP defines overlay inputs and outputs.
- The IP defines higher-restriction-wins.
- The IP defines metadata-only and payload movement behavior.
- The IP defines emergency behavior.
- The IP defines replay behavior.
- The IP defines settlement visibility behavior.
- The IP defines dashboard and SLO hooks.
- The IP includes all seven named benchmark families.
- The IP keeps ADR-0321 referenced but unmodified.

## 17. Done Criteria
- OpenAPI examples carry resolved overlay id.
- AsyncAPI examples include overlay-resolved and overlay-denied events.
- Proto examples include internal resolver request and response.
- Cedar tests cover missing pack, conflict, and emergency-limited cases.
- Replay fixtures include original and current overlay ids.
- Dashboard validates pack health dimensions.
- Runbook covers HIPAA pack misconfiguration.
- Settlement evidence excludes PHI.
- Capacity admission respects local-only queues.
- No other file is required for this IP deepening pass.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `ISO27001-2022` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-015-data-residency-pack-overlays.md:17` - - microservices/healthcare-integration/multi-region.md; `microservices/healthcare-integration/IP-015-data-residency-pack-overlays.md:31` - - Healthcare-integration handles PHI, clinical consent, HL7 messages, FHIR resources, provenance, and emergency exceptions..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-015-data-residency-pack-overlays.md:147` - - Add cost budget dependency on residency surcharge eligibility..
