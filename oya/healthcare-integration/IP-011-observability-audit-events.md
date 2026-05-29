# IP-011 Healthcare Integration observability-audit-events

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-011-observability-audit-events.md
Batch: C healthcare-integration IP deepening
Status: implementation-plan-ready
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json; microservices/healthcare-integration/dashboards/slo-and-error-budget.json; microservices/healthcare-integration/dashboards/local-policy-decisions.json; microservices/healthcare-integration/dashboards/local-audit-completeness.json; microservices/healthcare-integration/slos/audit-emission-lag.openslo.yaml; microservices/healthcare-integration/slos/local-audit-completeness.openslo.yaml; microservices/healthcare-integration/slos/policy-decision-latency.openslo.yaml; microservices/healthcare-integration/compliance.md

## Objective
- IP-011-001: Define healthcare-integration observability and audit events so every clinical state transition, policy decision, credential lease, cell route, replay, and denial is traceable without exposing PHI.
- IP-011-002: Preserve ADR-0263 as the primary emission contract and make audit events first-class acceptance evidence, not optional logs.
- IP-011-003: Preserve ADR-0242 and ADR-0244 by including tenant scope in signed audit evidence while keeping high-cardinality tenant ids out of metrics labels.
- IP-011-004: Preserve ADR-0243 and ADR-0246 by making policy decisions observable before side effects.
- IP-011-005: Preserve ADR-0294 by observing policy fragment soak, anomaly, and rollback.
- IP-011-006: Preserve ADR-0296 by observing credential lease request, issue, use, revoke, rotate, and quarantine without leaking secrets.
- IP-011-007: Preserve ADR-0297 by observing abuse-friction and denial outcomes.
- IP-011-008: Preserve ADR-0314 by observing DealSet settlement context for commercial healthcare integration flows.
- IP-011-009: Preserve ADR-0321 by displacing healthcare interoperability competitors with evidence completeness and tenant-scoped auditability.
- IP-011-010: Keep this IP as a documentation/control deepening only; it does not edit dashboards, SLOs, compliance, or telemetry files.

## Current thin content replacement
- IP-011-011: The previous file repeated generic rows and did not define telemetry taxonomy, audit event names, redaction rules, dashboards, SLOs, or evidence acceptance.
- IP-011-012: This rewrite ties the plan to local dashboards, SLOs, compliance.md, PRD requirements, and architecture failure modes.
- IP-011-013: This rewrite covers FHIR, HL7, break-glass, consent, provenance, patient-match, policy, credential, residency, marketplace, and abuse events.
- IP-011-014: This rewrite treats observability as regulatory and operational evidence, not just production debugging.
- IP-011-015: This rewrite explicitly rejects raw PHI in metrics, logs, traces, and error payloads.

## Telemetry principles
- IP-011-016: Metrics are for aggregate service health and must not contain raw tenant id, patient id, resource id, HL7 text, FHIR JSON, or secret path labels.
- IP-011-017: Logs are for operator diagnostics and must use stable evidence references instead of clinical payloads.
- IP-011-018: Traces are for causality and must link ingress, policy, domain, adapter, audit-chain, workflow, and dependency spans.
- IP-011-019: Audit events are for signed accountability and may include tenant id in protected evidence form.
- IP-011-020: Audit events must be emitted for permit and denial, not only successful mutation.
- IP-011-021: Audit event payloads must classify data, action, capability, principal, tenant, pack, cell, and outcome.
- IP-011-022: Audit events must link to policy decision id when policy was evaluated.
- IP-011-023: Audit events must link to credential lease ref when a credential was requested or used.
- IP-011-024: Audit events must link to DealSet ref when commercial settlement applies.
- IP-011-025: Audit events must link to workflow_run_id when workflow-engine participates.
- IP-011-026: Audit events must link to ontology_projection_id when projected data is returned or changed.
- IP-011-027: Audit events must link to replay_batch_id for replay and backfill.
- IP-011-028: Audit events must link to incident_id during degraded, failover, and emergency modes.
- IP-011-029: Audit-chain outage blocks high-risk mutation before evidence loss.
- IP-011-030: Observability is accepted only when dashboards, SLOs, audit fixtures, and redaction tests agree.

## Event taxonomy
- IP-011-031: healthcare.fhir.read.requested records FHIR read intent before policy decision.
- IP-011-032: healthcare.fhir.read.permitted records permitted projection read with data_class and projection version.
- IP-011-033: healthcare.fhir.read.denied records denial reason class and refusal_event_ref.
- IP-011-034: healthcare.fhir.export.requested records export target class and pack overlay.
- IP-011-035: healthcare.fhir.export.completed records evidence bundle id and recipient class.
- IP-011-036: healthcare.hl7.route.accepted records source route, transform id, and canonical message reference.
- IP-011-037: healthcare.hl7.route.acknowledged records ACK/NACK class, destination class, and route id.
- IP-011-038: healthcare.hl7.route.replayed records replay batch, prior decision ref, and last safe event id.
- IP-011-039: healthcare.breakglass.opened records emergency reason class, scope bounds, expiry, and reviewer route.
- IP-011-040: healthcare.breakglass.denied records failed reason without exposing patient payload.
- IP-011-041: healthcare.breakglass.closed records closeout status, reviewer, and evidence completion.
- IP-011-042: healthcare.consent.sync.requested records source consent ref and consent graph version.
- IP-011-043: healthcare.consent.sync.conflict records conflict class and stricter-rule route.
- IP-011-044: healthcare.consent.sync.resolved records selected state and operator justification ref.
- IP-011-045: healthcare.provenance.seal.created records source system, transform id, seal id, and evidence hash.
- IP-011-046: healthcare.provenance.seal.verified records verification state and evidence bundle id.
- IP-011-047: healthcare.patientmatch.review.queued records candidate count band and score band.
- IP-011-048: healthcare.patientmatch.review.decided records reviewer, decision class, and correction workflow ref.
- IP-011-049: healthcare.policy.evaluated records decision state, fragment id, version, and permit scope class.
- IP-011-050: healthcare.policy.fragment.anomaly records fragment hash, soak cohort, and rollback action.
- IP-011-051: healthcare.credential.lease.issued records lease scope and expiry without raw secret details.
- IP-011-052: healthcare.credential.lease.denied records scope denial and refusal ref.
- IP-011-053: healthcare.credential.lease.revoked records revocation trigger and affected lease ids.
- IP-011-054: healthcare.cell.route.decided records source cell, target cell, route reason, and residency result.
- IP-011-055: healthcare.cell.cross_cell.denied records pack, data_class, source cell, and target cell.
- IP-011-056: healthcare.marketplace.dealset.checked records DealSet ref and settlement result.
- IP-011-057: healthcare.abuse.friction.required records risk class and friction route.
- IP-011-058: healthcare.abuse.denied records risk class and refusal event.
- IP-011-059: healthcare.audit.backpressure.blocked records mutation class blocked due to audit-chain risk.
- IP-011-060: healthcare.incident.recovery.closed records incident id, replay range, and audit completeness.

## Metric taxonomy
- IP-011-061: request_total counts entry path, capability, method family, data_class, and outcome.
- IP-011-062: request_latency tracks p50, p95, and p99 by entry path and capability.
- IP-011-063: policy_eval_total counts decision_state, action, data_class, and pack class.
- IP-011-064: policy_eval_latency tracks decision latency against policy-decision-latency SLO.
- IP-011-065: audit_emission_lag tracks time from state transition to audit-chain acknowledgement.
- IP-011-066: audit_completeness_ratio tracks expected vs emitted audit events.
- IP-011-067: credential_lease_issued_total counts credential class and outcome.
- IP-011-068: credential_lease_denied_total counts denial reason class.
- IP-011-069: cell_route_total counts source cell, target cell, route state, and outcome.
- IP-011-070: residency_denied_total counts pack class and data_class.
- IP-011-071: metadata_only_response_total counts degraded safe response shape.
- IP-011-072: hl7_ack_latency tracks ACK/NACK latency by route class.
- IP-011-073: fhir_bundle_success_ratio tracks bundle reads and exports.
- IP-011-074: consent_sync_freshness tracks revocation and grant propagation.
- IP-011-075: breakglass_closeout_latency tracks emergency review closeout.
- IP-011-076: patient_match_review_queue_age tracks human review backlog.
- IP-011-077: provenance_seal_verify_total tracks verification outcomes.
- IP-011-078: dealset_settlement_denied_total tracks commercial settlement denials.
- IP-011-079: abuse_friction_total tracks risk class and friction type.
- IP-011-080: replay_cursor_lag tracks replay freshness and backfill recovery.

## Trace requirements
- IP-011-081: Trace root starts at REST, gRPC, worker, replay, or local operation ingress.
- IP-011-082: Trace includes policy evaluation span before domain mutation.
- IP-011-083: Trace includes consent-graph span before PHI materialization.
- IP-011-084: Trace includes ontology span before projection response.
- IP-011-085: Trace includes credential sidecar span before vendor adapter call.
- IP-011-086: Trace includes vendor adapter span without raw request or response payload.
- IP-011-087: Trace includes workflow-engine span for approval, review, replay, and closeout tasks.
- IP-011-088: Trace includes audit-chain span for every required evidence event.
- IP-011-089: Trace includes marketplace span for DealSet checks.
- IP-011-090: Trace includes abuse-defence span for risk scoring.
- IP-011-091: Trace includes cell route span for cross-cell or failover decisions.
- IP-011-092: Trace includes retry and idempotency span links for duplicate requests.
- IP-011-093: Trace includes replay span links to original event ids.
- IP-011-094: Trace attributes use redacted identifiers or evidence ids.
- IP-011-095: Trace sampling keeps all denial, break-glass, replay, export, and policy anomaly traces.

## Log and redaction rules
- IP-011-096: Logs may include evidence ids, audit ids, decision ids, route ids, transform ids, and workflow ids.
- IP-011-097: Logs may include capability, method, outcome, data_class, pack class, and reason class.
- IP-011-098: Logs may not include raw patient names, MRNs, dates of birth, addresses, or patient contact values.
- IP-011-099: Logs may not include raw HL7 segments.
- IP-011-100: Logs may not include raw FHIR resources.
- IP-011-101: Logs may not include OAuth tokens, client secrets, private keys, certificates, authorization headers, or secret paths.
- IP-011-102: Logs may not include unredacted consent text.
- IP-011-103: Logs may not include source-system patient ids unless tokenized.
- IP-011-104: Error logs must include operator_action_ref when remediation exists.
- IP-011-105: Denial logs must include refusal_event_ref.
- IP-011-106: Break-glass logs must include expiry and review workflow, not raw patient scope.
- IP-011-107: Replay logs must include replay_batch_id and event cursor.
- IP-011-108: Export logs must include recipient class and evidence bundle id.
- IP-011-109: Redaction tests must treat logs, traces, metrics, and errors as separate surfaces.
- IP-011-110: Redaction failure blocks promotion.

## Dashboard expectations
- IP-011-111: dashboards/slo-and-error-budget.json shows availability, latency, audit lag, replay freshness, and error budget burn.
- IP-011-112: dashboards/local-policy-decisions.json shows policy permits, denials, latency, fragment anomalies, and rollback events.
- IP-011-113: dashboards/local-audit-completeness.json shows expected vs emitted audit events by capability and risk class.
- IP-011-114: dashboards/abuse-defence-outcomes.json shows friction, denial, and false-positive review classes.
- IP-011-115: dashboards/local-slo-burn.json shows local healthcare-integration burn by cell and capability.
- IP-011-116: dashboards/compliance-pack-health.json shows pack overlay health without raw tenant labels.
- IP-011-117: dashboards/tenant-cost-and-capacity.json shows cost dimensions by tenant class, not raw tenant id.
- IP-011-118: dashboards/local-domain-throughput.json shows FHIR, HL7, consent, provenance, and patient-match throughput.
- IP-011-119: dashboards/local-operator-remediation.json shows runbook-triggered operator actions.
- IP-011-120: Dashboard acceptance requires all panels to avoid PHI and secret labels.

## SLO expectations
- IP-011-121: audit-emission-lag.openslo.yaml anchors audit emission time.
- IP-011-122: local-audit-completeness.openslo.yaml anchors completeness of required evidence events.
- IP-011-123: policy-decision-latency.openslo.yaml anchors policy evaluation latency.
- IP-011-124: local-hl7-ack-latency.openslo.yaml anchors route acknowledgement latency.
- IP-011-125: local-fhir-bundle-success.openslo.yaml anchors FHIR bundle success.
- IP-011-126: local-consent-sync-freshness.openslo.yaml anchors consent propagation.
- IP-011-127: local-hipaa-access-review-latency.openslo.yaml anchors access review.
- IP-011-128: local-phi-delivery-latency.openslo.yaml anchors PHI delivery latency where permitted.
- IP-011-129: replay-freshness.openslo.yaml anchors backfill and recovery freshness.
- IP-011-130: availability.openslo.yaml anchors service availability.
- IP-011-131: SLO burn events must include capability, cell, data_class, and pack class.
- IP-011-132: SLO burn events must link to runbook and incident id when paging.
- IP-011-133: SLOs must define what is excluded for policy denials versus system failures.
- IP-011-134: Audit completeness SLO must count denied requests that require refusal evidence.
- IP-011-135: Break-glass closeout SLO must start at permit issuance, not incident closure.

## Compliance and evidence
- IP-011-136: compliance.md anchors pack-level evidence obligations.
- IP-011-137: HIPAA evidence includes access, disclosure, break-glass, audit closeout, and breach-relevant events.
- IP-011-138: SOC-2 evidence includes control operation, denial, change, incident, and review events.
- IP-011-139: ISO-27001 evidence includes risk treatment, access control, incident, and supplier evidence.
- IP-011-140: GDPR evidence includes purpose, data minimization, export, deletion/retention, and cross-border denial events.
- IP-011-141: KR-Medical-Devices evidence includes device-regulated audit and change traceability where applicable.
- IP-011-142: EU-MDR evidence includes provenance, traceability, and regulated export evidence.
- IP-011-143: GxP evidence includes validation, signing, immutable provenance, and audit trail completeness.
- IP-011-144: Evidence bundles must be exportable without raw secrets.
- IP-011-145: Evidence bundles must be exportable with redacted clinical references when pack rules require.

## Benchmark displacement
- IP-011-146: Redox is displaced by audit completeness that covers permits, denials, policy, settlement, and replay instead of normalized exchange telemetry alone.
- IP-011-147: Rhapsody is displaced by cross-layer traces and signed evidence rather than route-engine logs.
- IP-011-148: InterSystems IRIS for Health is displaced by service-local, flat-boundary audit events rather than database-centered operational telemetry.
- IP-011-149: Lyniate/Corepoint is displaced by structured denial and recovery evidence rather than channel event logs.
- IP-011-150: Mirth is displaced by redaction-tested logs and audit-chain events instead of script/channel diagnostics.
- IP-011-151: NextGate is displaced by patient-match review queue evidence, decision provenance, and correction workflows.
- IP-011-152: Health Catalyst is displaced by compliance-ready clinical exchange evidence before analytics output.
- IP-011-153: Epic parity pressure is handled through FHIR access audit depth.
- IP-011-154: Cerner parity pressure is handled through HL7 ACK and route evidence.
- IP-011-155: Veeva parity pressure is handled through GxP provenance and audit trail continuity.

## Implementation steps
- IP-011-156: Inventory current events, metrics, traces, logs, dashboards, and SLO files.
- IP-011-157: Define telemetry naming conventions for healthcare-integration event families.
- IP-011-158: Add typed audit event schemas for each event listed in this IP.
- IP-011-159: Add metric definitions and label allowlists.
- IP-011-160: Add redaction tests for logs, traces, metrics, errors, and audit exports.
- IP-011-161: Add trace span requirements to REST, gRPC, worker, replay, policy, credential, and adapter paths.
- IP-011-162: Add audit-chain emission tests for each mutation and denial.
- IP-011-163: Add dashboard checks that required panels exist and avoid disallowed labels.
- IP-011-164: Add SLO checks for audit lag, audit completeness, policy latency, and replay freshness.
- IP-011-165: Add compliance evidence fixtures for HIPAA, SOC-2, ISO-27001, GDPR, EU-MDR, GxP, and KR-Medical-Devices.
- IP-011-166: Add incident linkage for failover, audit backpressure, policy anomaly, and secret quarantine.
- IP-011-167: Add runbook references for each alert family.
- IP-011-168: Add benchmark displacement checklist for telemetry and audit evidence.
- IP-011-169: Add replay verification that denial and permit events remain ordered.
- IP-011-170: Add cost and capacity telemetry for async clinical work without PHI labels.

## Tests and evidence
- IP-011-171: Unit evidence: every mutation emits an audit event.
- IP-011-172: Unit evidence: every denial emits refusal evidence.
- IP-011-173: Unit evidence: metrics label allowlist excludes raw tenant, patient, resource, and secret labels.
- IP-011-174: Unit evidence: logs redact HL7, FHIR, PHI, consent text, and credentials.
- IP-011-175: Unit evidence: traces link policy before domain mutation.
- IP-011-176: Unit evidence: audit-chain outage blocks high-risk mutation.
- IP-011-177: Unit evidence: policy fragment anomaly emits rollback evidence.
- IP-011-178: Unit evidence: credential lease use emits issue/use/revoke evidence.
- IP-011-179: Unit evidence: DealSet denial emits settlement refusal event.
- IP-011-180: Unit evidence: abuse friction emits risk and friction event.
- IP-011-181: Integration evidence: dashboard files contain required panel families.
- IP-011-182: Integration evidence: SLO files map to emitted metrics.
- IP-011-183: Compliance evidence: pack exports include required audit event families.
- IP-011-184: Replay evidence: replay preserves original event ordering and links.
- IP-011-185: Incident evidence: recovery closeout includes audit completeness proof.

## Rollback
- IP-011-186: If metrics leak sensitive labels, disable affected metric export and keep audit-chain evidence.
- IP-011-187: If logs leak PHI, block promotion and rotate impacted log retention/export access if required.
- IP-011-188: If audit event schema is incomplete, block high-risk mutation for affected capability.
- IP-011-189: If dashboard panels misrepresent policy denials as failures, remove panel from readiness evidence.
- IP-011-190: If SLO burn counts policy denials incorrectly, fix query before promotion.
- IP-011-191: If trace sampling drops break-glass or denial traces, force sampling for high-risk families.
- IP-011-192: If compliance export includes raw secrets, quarantine export and rotate affected credentials.
- IP-011-193: If replay ordering breaks, freeze replay and run audit reconciliation.
- IP-011-194: If audit-chain backpressure handling fails, disable high-risk mutation paths.
- IP-011-195: Rollback evidence includes event family, affected capability, data_class, pack, incident id, and audit ids.

## Acceptance criteria
- IP-011-196: Every capability has permit, denial, mutation, replay, and evidence event coverage.
- IP-011-197: Every metric uses an allowed label set.
- IP-011-198: Every high-risk trace is retained.
- IP-011-199: Every audit event links tenant, principal, action, data_class, policy decision, and outcome.
- IP-011-200: Every credential event omits raw secret material.
- IP-011-201: Every dashboard avoids PHI and secret labels.
- IP-011-202: Every SLO maps to emitted metrics.
- IP-011-203: Every benchmark displacement claim maps to telemetry or audit evidence.
- IP-011-204: ADR-0321 remains cited as doctrine and is not edited by this IP.
- IP-011-205: Implementation can proceed without touching unassigned files in this batch.

## Citation summary
- IP-011-206: PRD.md supplies audit-chain, metrics, traces, logs, rollback, and pack evidence requirements.
- IP-011-207: ARCHITECTURE.md supplies failure modes and dependency topology for observability.
- IP-011-208: manifest.json supplies binding ADRs, packs, benchmarks, and dependency list.
- IP-011-209: dashboards/slo-and-error-budget.json anchors service health and burn evidence.
- IP-011-210: dashboards/local-policy-decisions.json anchors policy observability.
- IP-011-211: dashboards/local-audit-completeness.json anchors audit completeness evidence.
- IP-011-212: slos/audit-emission-lag.openslo.yaml anchors audit emission latency.
- IP-011-213: slos/local-audit-completeness.openslo.yaml anchors audit completeness.
- IP-011-214: slos/policy-decision-latency.openslo.yaml anchors policy decision latency.
- IP-011-215: compliance.md anchors pack-level evidence expectations.
- IP-011-216: ADR-0321 remains cited as existing B2B leader coverage doctrine only; this IP does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `ISO27001-2022` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-011-observability-audit-events.md:12` - - IP-011-001: Define healthcare-integration observability and audit events so every clinical state transition, policy decision, credential lease, cell route, replay, a...; `microservices/healthcare-integration/IP-011-observability-audit-events.md:21` - - IP-011-010: Keep this IP as a documentation/control deepening only; it does not edit dashboards, SLOs, compliance, or telemetry files..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-011-observability-audit-events.md:9` - Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json;...; `microservices/healthcare-integration/IP-011-observability-audit-events.md:13` - - IP-011-002: Preserve ADR-0263 as the primary emission contract and make audit events first-class acceptance evidence, not optional logs..
