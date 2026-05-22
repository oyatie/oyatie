# IP-012 Healthcare Integration abuse-defence-edge-waf

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-012-abuse-defence-edge-waf.md
Batch: C healthcare-integration IP deepening
Status: implementation-plan-ready
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json; microservices/healthcare-integration/threat-model.md; microservices/healthcare-integration/policy/abuse-defence.cedar; microservices/healthcare-integration/iac/edge-waf.yaml; microservices/healthcare-integration/iac/production-ingress.yaml; microservices/healthcare-integration/dashboards/abuse-defence-outcomes.json; microservices/healthcare-integration/incident-response.md; microservices/healthcare-integration/runbooks/emergency-services-chaos.md

## Objective
- IP-012-001: Define healthcare-integration abuse defence at the edge and WAF layer so clinical interoperability endpoints resist bot, spoof, scrape, enumeration, replay, credential stuffing, and emergency-flow abuse without blocking legitimate clinical continuity.
- IP-012-002: Preserve ADR-0297 by making abuse-defence baseline controls explicit, observable, and connected to policy decisions.
- IP-012-003: Preserve ADR-0253-amendment by keeping HTTP/3-first, TLS 1.3, ECH/PQC-aware ingress posture visible in WAF and ingress evidence.
- IP-012-004: Preserve ADR-0243 and ADR-0246 by routing high-risk cases through Cedar and not through ad hoc edge allowlists alone.
- IP-012-005: Preserve ADR-0244 by applying tenant scope and data_class to edge risk decisions.
- IP-012-006: Preserve ADR-0263 by emitting abuse friction, denial, bypass, and incident events.
- IP-012-007: Preserve ADR-0314 by protecting marketplace partner flows without bypassing DealSet checks.
- IP-012-008: Preserve ADR-0296 by integrating credential-stuffing controls with sidecar lease denial and rotation events.
- IP-012-009: Preserve ADR-0321 by displacing integration competitors with safety-aware abuse controls, not just commodity WAF rules.
- IP-012-010: Keep this IP as a documentation/control plan only; it does not edit WAF, ingress, Cedar, dashboard, or runbook files.

## Current thin content replacement
- IP-012-011: The previous file repeated generic capability rows and did not define edge assets, threat classes, WAF rules, clean-path protections, emergency behavior, or benchmark displacement.
- IP-012-012: This rewrite cites threat-model.md, abuse-defence.cedar, edge-waf.yaml, production-ingress.yaml, abuse dashboard, incident response, and emergency-services runbook.
- IP-012-013: This rewrite separates public REST ingress, partner APIs, webhook listeners, emergency flows, internal gRPC, and replay workers.
- IP-012-014: This rewrite treats abuse defence as healthcare safety control; friction must be targeted and evidence-backed.
- IP-012-015: This rewrite does not authorize broad edge blocking that would interrupt emergency-services bypass or legitimate clinical exchange.

## Protected surfaces
- IP-012-016: Public FHIR read and search endpoints require enumeration protection.
- IP-012-017: FHIR export endpoints require recipient, pack, and volume anomaly checks.
- IP-012-018: HL7 ingress endpoints require source-system authentication, route binding, and replay-window checks.
- IP-012-019: HL7 ACK endpoints require spoof and duplicate ACK protection.
- IP-012-020: Consent sync endpoints require stale-source, conflicting-source, and mass-revocation anomaly checks.
- IP-012-021: Break-glass endpoints require emergency reason class, scope, expiry, reviewer route, and false-emergency abuse checks.
- IP-012-022: Emergency-services bypass endpoints require clean-path preservation and post-event evidence.
- IP-012-023: EHR provenance endpoints require signing request rate and payload-shape checks.
- IP-012-024: Patient-match endpoints require candidate enumeration and score-harvesting checks.
- IP-012-025: Marketplace partner endpoints require DealSet and partner identity checks.
- IP-012-026: Audit export endpoints require recipient class and export volume checks.
- IP-012-027: Internal gRPC endpoints are not public WAF surfaces but still consume risk signals.
- IP-012-028: Replay workers are not edge surfaces but must honor replay-risk decisions.
- IP-012-029: CI and auditor endpoints require scoped policy and synthetic-data defaults.
- IP-012-030: Health and readiness endpoints must not reveal tenant, route, vendor, or pack details.

## Threat classes
- IP-012-031: Bot traffic includes automated probing of FHIR search, export, and patient-match endpoints.
- IP-012-032: Spoof traffic includes fake EHR source-system callbacks, forged HL7 ACKs, and forged partner webhook signatures.
- IP-012-033: Scrape traffic includes high-cardinality FHIR searches, patient enumeration, and metadata harvesting.
- IP-012-034: Replay traffic includes duplicate HL7 messages, repeated export requests, reused idempotency keys, and stale webhook delivery.
- IP-012-035: Credential stuffing includes repeated OAuth, API key, mTLS, or sidecar lease attempts across tenants.
- IP-012-036: Break-glass abuse includes false emergency reason, excessive scope, repeated open events, and closeout avoidance.
- IP-012-037: Consent abuse includes mass consent grant, mass revocation, source mismatch, and downgrade attempts.
- IP-012-038: Marketplace abuse includes DealSet replay, partner tenant spoofing, and settlement-bypass attempts.
- IP-012-039: Provenance abuse includes signing arbitrary payloads, repeated seal creation, and seal verification flood.
- IP-012-040: Patient-match abuse includes score probing, candidate list expansion, and correction workflow spam.
- IP-012-041: Residency abuse includes requests shaped to force cross-cell materialization.
- IP-012-042: Policy fragment abuse includes requests targeting fragment anomaly windows.
- IP-012-043: Audit evasion includes high-risk mutation during audit-chain degraded state.
- IP-012-044: Downgrade abuse includes forcing HTTP/1.1, weak TLS, or edge fallback.
- IP-012-045: Operational abuse includes health endpoint probing and route discovery.

## Edge controls
- IP-012-046: edge-waf.yaml declares WAF rule groups for bot, spoof, scrape, replay, stuffing, and emergency misuse.
- IP-012-047: production-ingress.yaml declares ingress controls for protocol, TLS, headers, body size, rate, and route shape.
- IP-012-048: Edge requires TLS 1.3 floor where applicable and blocks downgrade patterns.
- IP-012-049: Edge requires strict host and route matching.
- IP-012-050: Edge enforces request body size limits by endpoint class.
- IP-012-051: Edge enforces method allowlists by route.
- IP-012-052: Edge enforces header presence for trace, idempotency, tenant route, and partner signature where applicable.
- IP-012-053: Edge validates partner webhook signatures before queueing.
- IP-012-054: Edge rejects repeated idempotency-key misuse across incompatible requests.
- IP-012-055: Edge rate limits FHIR search by tenant class, principal class, and risk class.
- IP-012-056: Edge rate limits HL7 ingress by route and source-system binding.
- IP-012-057: Edge rate limits break-glass opens more strictly than routine reads.
- IP-012-058: Edge rate limits provenance signing requests by tenant and transform class.
- IP-012-059: Edge rate limits patient-match candidate queueing by workflow and tenant class.
- IP-012-060: Edge sends risk signals to policy rather than deciding protected clinical scope alone.

## Clean-path and friction rules
- IP-012-061: Clean routine traffic should not see extra friction beyond normal authentication, policy, and consent gates.
- IP-012-062: Clean emergency-services traffic must keep a low-latency path while preserving evidence.
- IP-012-063: Suspicious FHIR search may be narrowed, challenged, or denied based on data_class and purpose.
- IP-012-064: Suspicious HL7 replay may be quarantined with safe ACK/NACK behavior.
- IP-012-065: Suspicious break-glass request may require immediate secondary approval or deny on scope mismatch.
- IP-012-066: Suspicious consent sync may require human review before applying a less restrictive grant.
- IP-012-067: Suspicious provenance signing may be denied unless payload hash and transform id match.
- IP-012-068: Suspicious patient-match review may cap candidate expansion and require reviewer confirmation.
- IP-012-069: Suspicious marketplace call may require DealSet revalidation.
- IP-012-070: Suspicious credential lease may deny issue and trigger rotation review.
- IP-012-071: Friction decisions must include reason class and operator action.
- IP-012-072: Friction must not reveal which patient, tenant, or resource exists.
- IP-012-073: Friction must not return different errors for existent versus nonexistent patient identifiers.
- IP-012-074: Denials must produce refusal_event_ref.
- IP-012-075: False-positive review must feed abuse dashboard and tuning workflow.

## Policy integration
- IP-012-076: abuse-defence.cedar is the policy evidence surface for risk-to-action mapping.
- IP-012-077: Edge risk signals become policy input fields, not standalone hidden decisions.
- IP-012-078: Policy sees risk_class, signal_source, confidence_band, endpoint_class, and requested_action.
- IP-012-079: Policy sees tenant class, audience_type, data_class, purpose, and pack ids.
- IP-012-080: Policy sees partner identity and DealSet context when commercial flow is involved.
- IP-012-081: Policy sees emergency_context for break-glass and emergency-services routes.
- IP-012-082: Policy sees credential lease context for stuffing and secret misuse.
- IP-012-083: Policy sees replay context for idempotency and source event reuse.
- IP-012-084: Policy may permit, deny, require friction, require review, quarantine, or metadata-only response.
- IP-012-085: Policy abstain fails closed for high-risk mutation.
- IP-012-086: Policy decision id is returned to edge and included in audit.
- IP-012-087: Policy fragment anomalies roll back under ADR-0294.
- IP-012-088: Policy cannot lower pack residency or consent restrictions due to clean risk score.
- IP-012-089: Policy cannot permit credential use beyond sidecar lease scope.
- IP-012-090: Policy cannot bypass DealSet settlement for partner calls.

## Emergency safeguards
- IP-012-091: Emergency-services bypass has explicit route names and cannot be reached by generic break-glass traffic.
- IP-012-092: Emergency route requires reason class, source identity, scope, expiry, and post-event review evidence.
- IP-012-093: Emergency route keeps clean traffic low friction.
- IP-012-094: Emergency route blocks source spoofing and repeated false emergency patterns.
- IP-012-095: Emergency route logs no raw PHI at edge.
- IP-012-096: Emergency route emits audit event before and after access where possible.
- IP-012-097: Emergency route queues audit evidence if audit-chain is degraded only where policy explicitly permits.
- IP-012-098: Emergency route creates post-event review workflow.
- IP-012-099: Emergency route closes with reviewer decision and scope verification.
- IP-012-100: Emergency route false-positive handling is reviewed separately from routine bot tuning.
- IP-012-101: Emergency route cannot create long-lived credential leases.
- IP-012-102: Emergency route cannot authorize broad cross-cell PHI replication.
- IP-012-103: Emergency route cannot bypass consent where law or pack requires explicit denial.
- IP-012-104: Emergency route chaos tests use runbooks/emergency-services-chaos.md.
- IP-012-105: Emergency safeguards prioritize clinical continuity and audit accountability together.

## Observability
- IP-012-106: abuse-defence-outcomes.json is the primary dashboard evidence target.
- IP-012-107: Metrics include edge_request_total, edge_denied_total, edge_friction_total, spoof_denied_total, scrape_limited_total, replay_quarantined_total, and stuffing_denied_total.
- IP-012-108: Metrics include endpoint_class, capability, risk_class, action_taken, data_class, and pack class.
- IP-012-109: Metrics exclude raw tenant id, patient id, source IP, secret path, and resource id.
- IP-012-110: Traces link ingress, WAF decision, policy decision, credential sidecar, domain command, and audit event.
- IP-012-111: Logs include risk class and evidence refs, not raw request bodies.
- IP-012-112: Audit events include abuse.friction.required, abuse.denied, spoof.denied, replay.quarantined, stuffing.denied, and emergency.safeguard.applied.
- IP-012-113: False-positive review events include operator decision and tuning recommendation.
- IP-012-114: Incident-response.md captures high-risk abuse incidents.
- IP-012-115: Threat-model.md captures bypass paths and control coverage.
- IP-012-116: SLO impact separates policy denials from system availability failures.
- IP-012-117: WAF rule changes emit change evidence and rollback id.
- IP-012-118: Partner abuse events link to DealSet and settlement refs.
- IP-012-119: Credential stuffing events link to lease denial or rotation evidence.
- IP-012-120: Emergency abuse events link to review workflow and closeout evidence.

## Benchmark displacement
- IP-012-121: Redox is displaced by risk-aware clinical exchange that preserves tenant, policy, settlement, and audit context at the edge.
- IP-012-122: Rhapsody is displaced by WAF and Cedar integration rather than route-engine rate rules alone.
- IP-012-123: InterSystems IRIS for Health is displaced by flat service edge controls decoupled from a central integration database.
- IP-012-124: Lyniate/Corepoint is displaced by spoof/replay protections tied to policy and evidence instead of channel trust alone.
- IP-012-125: Mirth Connect is displaced by governed edge controls rather than script or channel-specific abuse handling.
- IP-012-126: NextGate is displaced by patient-match anti-enumeration and candidate review protections.
- IP-012-127: Health Catalyst is displaced by abuse controls before clinical data reaches analytics or evidence projection.
- IP-012-128: Epic parity pressure is handled through FHIR enumeration and export protection.
- IP-012-129: Cerner parity pressure is handled through HL7 source spoofing and replay protection.
- IP-012-130: Veeva parity pressure is handled through GxP-grade audit of edge decisions.

## Implementation steps
- IP-012-131: Inventory edge-waf.yaml and production-ingress.yaml for current route, protocol, header, body, and rate controls.
- IP-012-132: Map each protected surface to endpoint_class, capability, action, risk class, and policy input.
- IP-012-133: Add or validate WAF rules for bot, spoof, scrape, replay, stuffing, break-glass abuse, and emergency misuse.
- IP-012-134: Add risk signal propagation from edge to policy evaluation.
- IP-012-135: Add Cedar decision handling for permit, deny, friction, review, quarantine, and metadata-only outcomes.
- IP-012-136: Add redaction checks for WAF logs and ingress logs.
- IP-012-137: Add no-enumeration response tests for FHIR and patient-match routes.
- IP-012-138: Add HL7 replay quarantine tests.
- IP-012-139: Add partner webhook spoof denial tests.
- IP-012-140: Add credential stuffing sidecar-denial tests.
- IP-012-141: Add break-glass abuse false-emergency tests.
- IP-012-142: Add emergency clean-path latency and evidence tests.
- IP-012-143: Add dashboard and audit event checks for abuse outcomes.
- IP-012-144: Add incident drill for coordinated scrape and credential stuffing.
- IP-012-145: Add rollback fixture for bad WAF rule deployment.

## Tests and evidence
- IP-012-146: Unit evidence: FHIR search enumeration returns uniform denial or friction without existence leaks.
- IP-012-147: Unit evidence: FHIR export burst requires policy review or denies.
- IP-012-148: Unit evidence: forged HL7 source signature denies before queueing.
- IP-012-149: Unit evidence: duplicate HL7 replay quarantines with safe ACK/NACK behavior.
- IP-012-150: Unit evidence: forged partner webhook denies and emits spoof evidence.
- IP-012-151: Unit evidence: credential stuffing denies lease issue and emits sidecar evidence.
- IP-012-152: Unit evidence: break-glass excessive scope requires review or denies.
- IP-012-153: Unit evidence: emergency clean path remains low friction for valid request.
- IP-012-154: Unit evidence: emergency false pattern triggers post-event review.
- IP-012-155: Unit evidence: patient-match score probing is capped or denied.
- IP-012-156: Unit evidence: provenance signing flood denies arbitrary payload signing.
- IP-012-157: Integration evidence: WAF rule emits policy input and audit event.
- IP-012-158: Integration evidence: ingress blocks protocol downgrade patterns.
- IP-012-159: Dashboard evidence: abuse outcomes appear by risk class and action taken.
- IP-012-160: Redaction evidence: edge and WAF logs contain no PHI, raw source IP labels in metrics, or secrets.

## Rollback
- IP-012-161: If a WAF rule blocks clean emergency traffic, disable that rule and keep policy/audit evidence for emergency path.
- IP-012-162: If a WAF rule leaks route existence, replace with uniform response behavior.
- IP-012-163: If a WAF rule drops trace or policy context, route through prior accepted ingress config.
- IP-012-164: If abuse policy creates false denial for clean FHIR reads, lower to friction while review runs.
- IP-012-165: If bot controls leak patient existence, block affected response path.
- IP-012-166: If credential stuffing detection creates false lease revocation, restore prior lease generation and require review.
- IP-012-167: If partner spoof controls block valid DealSet flow, pin partner route to prior signature config.
- IP-012-168: If HL7 replay quarantine breaks ACK behavior, switch to safe NACK and pause route.
- IP-012-169: If dashboard labels leak sensitive data, disable dashboard export.
- IP-012-170: Rollback evidence includes WAF rule id, endpoint class, capability, risk class, policy decision, audit ids, and incident id.

## Acceptance criteria
- IP-012-171: Every public healthcare-integration endpoint has an endpoint_class and abuse risk profile.
- IP-012-172: Every edge risk signal reaches policy evaluation.
- IP-012-173: Every high-risk denial emits refusal_event_ref.
- IP-012-174: Every friction action emits operator-visible reason and audit evidence.
- IP-012-175: Every clean emergency flow preserves low-friction access and post-event evidence.
- IP-012-176: Every forged partner webhook is denied before queueing.
- IP-012-177: Every source spoof attempt is denied before clinical state mutation.
- IP-012-178: Every replay quarantine is idempotency-aware.
- IP-012-179: Every credential stuffing event ties to sidecar lease denial or rotation review.
- IP-012-180: Every WAF log and metric avoids PHI and secret values.
- IP-012-181: Every route downgrade attempt is blocked or audited.
- IP-012-182: Every DealSet-governed call preserves settlement checks.
- IP-012-183: Every pack or residency restriction outranks clean risk score.
- IP-012-184: Every benchmark displacement claim maps to edge, policy, evidence, or emergency safety control.
- IP-012-185: ADR-0297 behavior is explicit, observable, and rollback-ready.
- IP-012-186: ADR-0321 remains cited as doctrine and is not edited by this IP.
- IP-012-187: Implementation can proceed without touching unassigned files in this batch.
- IP-012-188: Verification can be done through WAF tests, policy tests, redaction tests, emergency drills, and dashboard checks.
- IP-012-189: No vendor-specific suite boundary or channel-specific abuse logic is introduced.
- IP-012-190: The plan preserves healthcare safety and abuse resistance together.

## Citation summary
- IP-012-191: PRD.md supplies clinical continuity, pack overlay, audit, latency, and quality expectations.
- IP-012-192: ARCHITECTURE.md supplies dependency topology and failure modes.
- IP-012-193: manifest.json supplies binding ADRs, audience type, packs, benchmarks, and cell rules.
- IP-012-194: threat-model.md anchors abuse, spoof, scrape, replay, stuffing, and bypass review.
- IP-012-195: policy/abuse-defence.cedar anchors risk-to-policy behavior.
- IP-012-196: iac/edge-waf.yaml anchors WAF rule evidence.
- IP-012-197: iac/production-ingress.yaml anchors public ingress evidence.
- IP-012-198: dashboards/abuse-defence-outcomes.json anchors abuse outcome observability.
- IP-012-199: incident-response.md anchors abuse incident handling and rollback evidence.
- IP-012-200: runbooks/emergency-services-chaos.md anchors emergency clean-path and chaos evidence.
- IP-012-201: policy/clinical-interoperability-authorization.cedar anchors policy permit/deny integration.
- IP-012-202: iac/secret-bindings.yaml anchors credential stuffing linkage to secret controls.
- IP-012-203: slos/availability.openslo.yaml anchors clean-path availability evidence.
- IP-012-204: ADR-0297 anchors abuse-defence doctrine.
- IP-012-205: ADR-0321 remains cited as existing B2B leader coverage doctrine only; this IP does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-012-abuse-defence-edge-waf.md:120` - - IP-012-095: Emergency route logs no raw PHI at edge.; `microservices/healthcare-integration/IP-012-abuse-defence-edge-waf.md:127` - - IP-012-102: Emergency route cannot authorize broad cross-cell PHI replication..
