# IP-010 Healthcare Integration multi-region-cell-layout

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-010-multi-region-cell-layout.md
Batch: C healthcare-integration IP deepening
Status: implementation-plan-ready
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json; microservices/healthcare-integration/multi-region.md; microservices/healthcare-integration/policy/data-residency.md; microservices/healthcare-integration/iac/dr-failover.yaml; microservices/healthcare-integration/iac/local-network-policy.yaml; microservices/healthcare-integration/iac/network-policy.yaml; microservices/healthcare-integration/slos/availability.openslo.yaml; microservices/healthcare-integration/slos/replay-freshness.openslo.yaml

## Objective
- IP-010-001: Define the multi-region cell layout for healthcare-integration so clinical interoperability remains tenant-home-cell first, pack-aware, audit-sealed, and recoverable during regional failure.
- IP-010-002: Preserve manifest cell eligibility: tier-0, tier-1, and tier-2 are eligible, tenant_home_cell_required is true, sovereign_pack_overrides_allowed is true, and cross_cell_replication is metadata-only-unless-pack-allows.
- IP-010-003: Preserve ADR-0244 by treating tenant_id and home_cell as mandatory routing inputs rather than optional placement hints.
- IP-010-004: Preserve ADR-0242 by ensuring Oyatie tenant doctrine governs all clinical PHI materialization.
- IP-010-005: Preserve ADR-0253-amendment by requiring strict transport, edge, and failover posture for cell-to-cell calls.
- IP-010-006: Preserve ADR-0263 by making failover, degraded reads, queueing, replay, and rollback observable audit events.
- IP-010-007: Preserve ADR-0296 by keeping credentials cell-local unless a pack explicitly permits remote metadata use.
- IP-010-008: Preserve ADR-0314 by keeping DealSet and marketplace evidence intact across cell failover.
- IP-010-009: Preserve ADR-0321 by displacing integration leaders with stronger residency and recovery semantics than generic interoperability hubs.
- IP-010-010: Keep this IP as a documentation/control plan only; it does not edit multi-region, IaC, SLO, or policy files.

## Current thin content replacement
- IP-010-011: The previous file repeated generic rows and did not define cell roles, replication rules, failover states, residency gates, or benchmark displacement.
- IP-010-012: This rewrite cites multi-region.md, data-residency.md, dr-failover.yaml, network policies, and availability/replay SLOs as the review surfaces.
- IP-010-013: This rewrite handles FHIR, HL7, consent, break-glass, provenance, and patient-match data separately because their residency and recovery profiles differ.
- IP-010-014: This rewrite treats regional outage as a clinical safety and compliance problem, not just an infrastructure availability event.
- IP-010-015: This rewrite refuses cross-cell PHI replication unless pack, consent, and policy evidence allow it.

## Cell roles
- IP-010-016: Tenant home cell owns canonical PHI materialization, consent state binding, credential leases, and audit-chain writes for protected clinical data.
- IP-010-017: Read shadow cell may hold metadata-only projections when pack rules permit.
- IP-010-018: Replay cell may process idempotent backfill jobs only when source events remain tied to home-cell audit evidence.
- IP-010-019: Disaster recovery cell may accept queued writes but cannot materialize PHI if residency or pack rules prohibit it.
- IP-010-020: Edge ingress cell terminates external traffic and forwards only after tenant route lookup and policy precheck.
- IP-010-021: Audit cell may receive signed event copies if audit export rules permit.
- IP-010-022: Marketplace cell may receive DealSet settlement references without clinical payload.
- IP-010-023: Analytics cell may receive redacted or aggregate evidence only after healthcare-integration policy permits export.
- IP-010-024: CI cell may run fixtures with synthetic data only.
- IP-010-025: Auditor cell may receive evidence bundles through scoped export workflows.
- IP-010-026: Emergency-services bypass uses home-cell first unless failure mode explicitly permits a bounded emergency route.
- IP-010-027: Break-glass access does not itself authorize cross-cell PHI replication.
- IP-010-028: Credential sidecars are cell-local and tied to OpenBao policy in that cell.
- IP-010-029: Workflow-engine orchestration must carry tenant home cell and target cell in every task.
- IP-010-030: Ontology projection may be replicated as metadata only when it contains no PHI payload.

## Data placement rules
- IP-010-031: FHIR resource payloads remain in tenant home cell unless pack and consent allow export.
- IP-010-032: FHIR metadata may replicate across permitted cells with source freshness and stale-region markers.
- IP-010-033: HL7 raw messages remain in tenant home cell or source route cell approved by residency policy.
- IP-010-034: HL7 ACK/NACK evidence may replicate as audit metadata.
- IP-010-035: Clinical consent state remains in home cell and consent-graph approved replicas.
- IP-010-036: Consent revocation metadata replicates faster than consent grant metadata.
- IP-010-037: Break-glass event payload remains home-cell constrained.
- IP-010-038: Break-glass closeout metadata replicates to audit and review cells.
- IP-010-039: EHR provenance seals may replicate as cryptographic evidence without raw source payload.
- IP-010-040: Signing private keys do not replicate across cells unless a pack-approved key ceremony exists.
- IP-010-041: Patient-match candidate identifiers remain home-cell constrained.
- IP-010-042: Patient-match score bands may replicate only as redacted review metadata.
- IP-010-043: DealSet references may replicate without clinical payload.
- IP-010-044: Audit event hashes may replicate for integrity checks.
- IP-010-045: Raw PHI never replicates to analytics cells by default.

## Routing and admission
- IP-010-046: Every ingress request resolves tenant home cell before selecting service instance.
- IP-010-047: Admission denies requests missing tenant_id, home_cell, jurisdiction_code, pack_ids, or data_class.
- IP-010-048: Admission checks data-residency.md before any cross-cell call.
- IP-010-049: Admission checks Cedar policy before remote materialization.
- IP-010-050: Admission checks consent state before PHI read or export.
- IP-010-051: Admission checks DealSet state before marketplace-triggered exchange.
- IP-010-052: Admission checks abuse risk before high-volume FHIR search, HL7 replay, or patient-match queueing.
- IP-010-053: Admission checks credential lease cell before vendor adapter calls.
- IP-010-054: Admission rejects source-system routes that would land in the wrong tenant home cell.
- IP-010-055: Admission marks stale-region responses with source_last_seen_at.
- IP-010-056: Admission returns metadata-only degraded response when materialization is blocked but safe metadata is allowed.
- IP-010-057: Admission returns residency_denied when pack rules prohibit cross-cell data movement.
- IP-010-058: Admission returns failover_queued only when the queued write can be replayed idempotently in the home cell.
- IP-010-059: Admission must not auto-promote a DR cell to canonical writer without governance evidence.
- IP-010-060: Admission emits audit events for every failover, denial, and metadata-only decision.

## Replication lanes
- IP-010-061: Lane A replicates audit hashes and event refs for integrity checks.
- IP-010-062: Lane B replicates redacted operational status for dashboards and SLOs.
- IP-010-063: Lane C replicates consent revocation signals when policy allows.
- IP-010-064: Lane D replicates replay cursor metadata and last safe event id.
- IP-010-065: Lane E replicates DealSet settlement refs and marketplace evidence.
- IP-010-066: Lane F replicates provenance seal verification state.
- IP-010-067: Lane G replicates workflow task state with no PHI payload.
- IP-010-068: Lane H replicates synthetic CI fixtures only.
- IP-010-069: Lane I replicates auditor evidence bundles through export workflows.
- IP-010-070: Lane J replicates emergency-services metadata for continuity only under emergency policy.
- IP-010-071: No lane replicates raw FHIR resource payload by default.
- IP-010-072: No lane replicates raw HL7 messages by default.
- IP-010-073: No lane replicates signing private keys by default.
- IP-010-074: No lane replicates raw patient-match candidate identifiers by default.
- IP-010-075: Each lane declares source cell, target cell, data class, pack rule, replay semantics, and rollback rule.

## Failover states
- IP-010-076: Normal state routes reads and writes to tenant home cell.
- IP-010-077: Read-degraded state serves safe metadata with stale markers and no PHI materialization.
- IP-010-078: Write-queued state accepts idempotent commands into a bounded queue for home-cell replay.
- IP-010-079: Write-blocked state rejects high-risk mutation until audit, policy, or residency recovers.
- IP-010-080: Audit-degraded state blocks high-risk mutation and allows safe status reads.
- IP-010-081: Policy-degraded state blocks mutation and PHI reads.
- IP-010-082: Consent-degraded state blocks PHI reads and consent-affecting writes.
- IP-010-083: Credential-degraded state blocks vendor-bound adapter calls and allows local evidence status.
- IP-010-084: Marketplace-degraded state blocks partner settlement-triggered calls and allows non-commercial status.
- IP-010-085: Emergency-limited state permits only explicitly scoped emergency-services flows.
- IP-010-086: DR-candidate state starts evidence collection but does not become canonical writer.
- IP-010-087: DR-active state requires governance approval, pack allowance, audit-chain continuity, and replay cursor lock.
- IP-010-088: Recovery-sync state replays queued idempotent writes and reconciles audit refs.
- IP-010-089: Recovery-verify state compares home-cell state, audit hashes, replay cursors, and denial evidence.
- IP-010-090: Recovered state resumes home-cell routing and closes incident evidence.

## Capability-specific behavior
- IP-010-091: fhir-read serves local projection in home cell and metadata-only stale response in allowed shadow cells.
- IP-010-092: fhir-read export is blocked when recipient cell violates pack residency.
- IP-010-093: hl7-route accepts source messages only in approved route cells.
- IP-010-094: hl7-route ACK windows must survive network partition without duplicate clinical state mutation.
- IP-010-095: hl7-route replay uses last_safe_event_id and idempotency keys.
- IP-010-096: break-glass-authorize requires home-cell audit closeout even if emergency metadata is routed elsewhere.
- IP-010-097: break-glass-authorize never creates permanent cross-cell access.
- IP-010-098: consent-sync prioritizes revocation replication and stricter conflict resolution.
- IP-010-099: consent-sync never resolves conflict by choosing the less restrictive region rule.
- IP-010-100: ehr-provenance-seal allows verification evidence to replicate but keeps signing authority constrained.
- IP-010-101: ehr-provenance-seal must preserve verification for pre-failover and post-failover evidence bundles.
- IP-010-102: patient-match-review queues candidate review in home cell.
- IP-010-103: patient-match-review can export redacted score-band evidence to auditor workflows.
- IP-010-104: patient-match-review correction workflows must replay in canonical cell.
- IP-010-105: Backfill and replay workers process only current-cell authorized batches.

## Network and IaC expectations
- IP-010-106: dr-failover.yaml is the deployment evidence target for DR routing behavior.
- IP-010-107: network-policy.yaml is the production cross-service traffic evidence target.
- IP-010-108: local-network-policy.yaml is the local cell traffic evidence target.
- IP-010-109: Network policy must deny direct database access from non-home cells.
- IP-010-110: Network policy must deny sidecar secret access from non-owning cells.
- IP-010-111: Network policy must allow audit event egress only to approved audit-chain endpoints.
- IP-010-112: Network policy must distinguish workflow-engine orchestration from vendor adapter traffic.
- IP-010-113: Failover routing must preserve trace context and audit_chain_ref.
- IP-010-114: Failover routing must not strip DealSet references.
- IP-010-115: Failover routing must not strip consent version.
- IP-010-116: Failover routing must include source cell, target cell, reason, and incident id.
- IP-010-117: Cell routing config must be reviewable without reading live secrets.
- IP-010-118: Cell promotion requires explicit governance evidence rather than automatic leader election.
- IP-010-119: Cell demotion preserves replay queues for post-incident verification.
- IP-010-120: IaC evidence must align with manifest cell_eligibility.

## Observability and SLOs
- IP-010-121: availability.openslo.yaml anchors availability evidence.
- IP-010-122: replay-freshness.openslo.yaml anchors replay cursor freshness evidence.
- IP-010-123: Metrics include cell_route_total, residency_denied_total, metadata_only_response_total, failover_queued_total, and replay_cursor_lag.
- IP-010-124: Metrics include capability, data_class, pack_class, source_cell, target_cell, and failover_state.
- IP-010-125: Metrics do not include raw tenant ids or patient ids.
- IP-010-126: Traces link edge ingress, policy decision, cell admission, dependency call, and audit-chain write.
- IP-010-127: Audit events include cell_route_decided, cross_cell_denied, failover_queued, replay_reconciled, and recovery_closed.
- IP-010-128: Dashboards/local-slo-burn.json and dashboards/slo-and-error-budget.json review SLO burn.
- IP-010-129: Runbooks/emergency-services-chaos.md handles emergency failover drill evidence.
- IP-010-130: Runbooks/local-hl7-ack-latency-burn.md handles route latency during failover.
- IP-010-131: Runbooks/local-consent-sync-lag.md handles consent revocation lag.
- IP-010-132: Incident-response.md records incident id, affected cells, pack constraints, replay range, and audit ids.
- IP-010-133: Failure-modes.md records regional outage, stale projection, and audit backpressure behavior.
- IP-010-134: Capacity-model.md records queue and replay sizing by tenant and cell.
- IP-010-135: Cost-budget.md records cross-cell cost dimensions without encouraging PHI replication.

## Benchmark displacement
- IP-010-136: Redox is displaced by tenant-home-cell residency and metadata-only cross-cell rules instead of neutral hub routing.
- IP-010-137: Rhapsody is displaced by explicit failover state machines rather than route-engine availability assumptions.
- IP-010-138: InterSystems IRIS for Health is displaced by cell-aware flat service boundaries rather than a central integration database.
- IP-010-139: Lyniate/Corepoint is displaced by policy-governed cross-cell routing instead of channel failover configuration.
- IP-010-140: Mirth Connect is displaced by replay-safe queues and audit evidence rather than script-level recovery.
- IP-010-141: NextGate is displaced by home-cell patient-match review and redacted evidence export.
- IP-010-142: Health Catalyst is displaced by data-minimizing replication before analytics or warehouse projection.
- IP-010-143: Epic parity pressure is handled by FHIR local projection and export residency rules.
- IP-010-144: Cerner parity pressure is handled by HL7 ACK windows and route-cell controls.
- IP-010-145: Veeva parity pressure is handled by GxP evidence continuity across failover.

## Implementation steps
- IP-010-146: Inventory multi-region.md for declared cells, routing states, and residency posture.
- IP-010-147: Add machine-checkable cell route table if missing in future implementation.
- IP-010-148: Add admission checks for tenant home cell, pack ids, jurisdiction, data_class, and policy decision.
- IP-010-149: Add replication lane declarations for audit hashes, operational status, consent revocation, replay cursors, DealSet refs, and provenance verification.
- IP-010-150: Add failover state machine tests for normal, read-degraded, write-queued, write-blocked, audit-degraded, and recovery states.
- IP-010-151: Add residency tests for PHI materialization denial across disallowed cells.
- IP-010-152: Add replay tests that reconcile queued writes by last_safe_event_id.
- IP-010-153: Add network policy checks for secret sidecar and database access.
- IP-010-154: Add SLO checks for availability and replay freshness.
- IP-010-155: Add dashboard panels for metadata-only responses and residency denials.
- IP-010-156: Add runbook links for regional outage, consent lag, HL7 ACK latency, and emergency-services chaos.
- IP-010-157: Add cost and capacity evidence for replay queues by tenant and cell.
- IP-010-158: Add audit events for every cell route decision.
- IP-010-159: Add incident evidence fixtures for DR activation and recovery closeout.
- IP-010-160: Add benchmark displacement review against each named healthcare integration competitor.

## Tests and evidence
- IP-010-161: Unit evidence: request missing home_cell is denied before routing.
- IP-010-162: Unit evidence: disallowed cross-cell PHI materialization is denied.
- IP-010-163: Unit evidence: metadata-only response includes stale-region metadata.
- IP-010-164: Unit evidence: queued write requires idempotency key and replay cursor.
- IP-010-165: Unit evidence: audit-chain outage blocks high-risk mutation.
- IP-010-166: Unit evidence: policy outage blocks PHI reads and mutation.
- IP-010-167: Unit evidence: consent revocation outranks stale grant replicas.
- IP-010-168: Unit evidence: DealSet references survive failover queueing.
- IP-010-169: Unit evidence: credential lease cell mismatch denies adapter call.
- IP-010-170: Integration evidence: dr-failover.yaml routes only approved failover states.
- IP-010-171: Integration evidence: network policy denies non-home database access.
- IP-010-172: SLO evidence: availability and replay freshness targets are reported by cell.
- IP-010-173: Audit evidence: cell_route_decided event includes source and target cell.
- IP-010-174: Incident evidence: recovery closeout includes replay range and audit ids.
- IP-010-175: Redaction evidence: cross-cell operational status contains no PHI payload.

## Rollback
- IP-010-176: If route admission fails open, disable cross-cell materialization and force home-cell routing.
- IP-010-177: If metadata-only response leaks PHI, disable shadow-cell reads.
- IP-010-178: If write queue loses idempotency, block write-queued state.
- IP-010-179: If DR activation lacks audit continuity, keep DR-candidate and block canonical writer promotion.
- IP-010-180: If network policy allows non-home DB access, isolate affected cell.
- IP-010-181: If consent revocation replication lags beyond SLO, block PHI reads in affected target cells.
- IP-010-182: If DealSet evidence drops in failover, block marketplace-triggered exchange.
- IP-010-183: If replay cursor reconciliation fails, freeze recovery-sync and run manual evidence review.
- IP-010-184: If credential sidecar cell mismatch occurs, block vendor-bound adapter calls.
- IP-010-185: Rollback evidence includes cells, capability, pack, incident id, route decisions, replay cursors, and audit ids.

## Acceptance criteria
- IP-010-186: Every request carries tenant_id, home_cell, jurisdiction_code, pack_ids, data_class, and policy decision.
- IP-010-187: Every cross-cell route passes residency, consent, policy, and audit checks.
- IP-010-188: Every raw PHI payload remains home-cell constrained unless pack and consent permit export.
- IP-010-189: Every metadata-only response is explicitly marked stale or degraded.
- IP-010-190: Every queued write is idempotent and replayable.
- IP-010-191: Every DR state transition emits audit evidence.
- IP-010-192: Every failover path preserves DealSet, consent, policy, trace, and audit refs.
- IP-010-193: Every credential-bound adapter call uses a cell-local lease.
- IP-010-194: Every SLO and dashboard view separates cell state without sensitive labels.
- IP-010-195: Every benchmark displacement claim maps to residency, failover, replay, or evidence controls.
- IP-010-196: ADR-0321 remains cited as doctrine and is not edited by this IP.
- IP-010-197: The plan supports implementation without touching unassigned files in this batch.
- IP-010-198: The implementation can be verified with admission, residency, failover, replay, network, SLO, and audit tests.
- IP-010-199: No suite, vendor, or region-specific folder boundary is introduced.
- IP-010-200: The cell layout remains compatible with ADR-0131 flat service ownership.

## Citation summary
- IP-010-201: PRD.md supplies service scope, pack overlays, latency, availability, and replay expectations.
- IP-010-202: ARCHITECTURE.md supplies regional outage, stale projection, audit backpressure, and key compromise failure modes.
- IP-010-203: manifest.json supplies cell eligibility, dependency list, packs, binding ADRs, and benchmark roster.
- IP-010-204: multi-region.md anchors service-local regional behavior.
- IP-010-205: policy/data-residency.md anchors residency gate behavior.
- IP-010-206: iac/dr-failover.yaml anchors failover deployment evidence.
- IP-010-207: iac/network-policy.yaml and iac/local-network-policy.yaml anchor cross-cell network restrictions.
- IP-010-208: slos/availability.openslo.yaml anchors availability evidence.
- IP-010-209: slos/replay-freshness.openslo.yaml anchors replay recovery evidence.
- IP-010-210: ADR-0321 remains cited as existing B2B leader coverage doctrine only; this IP does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-010-multi-region-cell-layout.md:1` - # IP-010 Healthcare Integration multi-region-cell-layout; `microservices/healthcare-integration/IP-010-multi-region-cell-layout.md:4` - ChangeSet scope: microservices/healthcare-integration/IP-010-multi-region-cell-layout.md.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-010-multi-region-cell-layout.md:164` - - IP-010-135: Cost-budget.md records cross-cell cost dimensions without encouraging PHI replication.; `microservices/healthcare-integration/IP-010-multi-region-cell-layout.md:190` - - IP-010-157: Add cost and capacity evidence for replay queues by tenant and cell..
