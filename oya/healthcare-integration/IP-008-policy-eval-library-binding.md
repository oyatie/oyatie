# IP-008 Healthcare Integration policy-eval-library-binding

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-008-policy-eval-library-binding.md
Batch: C healthcare-integration IP deepening
Status: implementation-plan-ready
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json; microservices/healthcare-integration/policy/clinical-interoperability-authorization.cedar; microservices/healthcare-integration/policy/abuse-defence.cedar; microservices/healthcare-integration/policy/data-residency.md; microservices/healthcare-integration/policies/local-fhir-exchange-consent.cedar; microservices/healthcare-integration/policies/local-patient-consent-sync.cedar; microservices/healthcare-integration/policies/local-breakglass-access-control.cedar; specs/cedar-fragment-schema.json

## Objective
- IP-008-001: Bind healthcare-integration to a caller-side policy evaluation library so every clinical interoperability call can prove permit, deny, fragment version, soak cohort, and refusal evidence before domain execution.
- IP-008-002: Keep Cedar evaluation as a reusable library contract, not a hidden adapter side effect, so REST, gRPC, workers, replay, and tests evaluate the same request shape.
- IP-008-003: Preserve ADR-0243 by treating Cedar as the universal gate for healthcare-integration actions.
- IP-008-004: Preserve ADR-0246 by keeping policy substrate behavior explicit and testable at service boundaries.
- IP-008-005: Preserve ADR-0294 by requiring fragment soak, anomaly detection, rollback metadata, and current fragment id in every evaluation result.
- IP-008-006: Preserve ADR-0263 by turning every decision into structured audit evidence rather than logs-only diagnostics.
- IP-008-007: Preserve ADR-0242 and ADR-0244 by rejecting policy evaluation when tenant, principal, purpose, audience, data_class, jurisdiction, or home_cell is absent.
- IP-008-008: Preserve ADR-0314 by including marketplace DealSet context in policies that control billable partner actions.
- IP-008-009: Preserve ADR-0297 by allowing abuse-defence signals to influence friction, denial, and post-review routing.
- IP-008-010: Preserve ADR-0321 by making leader displacement measurable: Oyatie policy evidence must be deeper than generic integration engine rule tables.

## Current thin content replacement
- IP-008-011: The previous file repeated generic rows and did not identify policy files, evaluation inputs, output schema, cache discipline, failure modes, or benchmark gaps.
- IP-008-012: This plan ties the policy library to clinical-interoperability-authorization.cedar, abuse-defence.cedar, data-residency.md, and local policy fragments.
- IP-008-013: This plan uses capability records as the action catalog for fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, and patient-match-review.
- IP-008-014: This plan treats specs/cedar-fragment-schema.json as the compatibility target for fragment payloads.
- IP-008-015: This plan does not edit Cedar policy files; it defines the binding contract that implementation must satisfy.

## Policy library boundary
- IP-008-016: The library accepts a typed HealthcarePolicyRequest and returns a typed HealthcarePolicyDecision.
- IP-008-017: The library is called before REST command handling, before gRPC domain dispatch, before worker adapter calls, and before replay side effects.
- IP-008-018: The library is not allowed to read raw PHI fields unless data_class and purpose explicitly require field-level policy.
- IP-008-019: The library is not allowed to call vendor EHR adapters.
- IP-008-020: The library is not allowed to mint credentials or refresh OpenBao leases.
- IP-008-021: The library may read soaked Cedar fragments, tenant pack overlays, caller workload identity, and risk signals.
- IP-008-022: The library may emit decision evidence, refusal evidence, and fragment anomaly evidence.
- IP-008-023: The library must expose deterministic evaluation suitable for property tests.
- IP-008-024: The library must expose policy version and fragment hash in every result.
- IP-008-025: The library must expose deny reason classes without exposing sensitive rule internals.
- IP-008-026: The library must expose permit scope as an explicit set of resource, action, purpose, data_class, and time bounds.
- IP-008-027: The library must not silently downgrade a deny to a degraded dependency response.
- IP-008-028: The library must make abstain and missing policy states distinct from deny.
- IP-008-029: The library must fail closed for mutation when abstain, missing fragment, or anomaly occurs.
- IP-008-030: The library may allow metadata-only reads in degraded mode only when all pack and consent rules allow it.

## Evaluation inputs
- IP-008-031: Input includes tenant_id, tenant_home_cell, jurisdiction_code, pack_ids, tenant_status, and data residency class.
- IP-008-032: Input includes principal_id, principal_type, audience_type, support_impersonation flag, and delegated authority chain.
- IP-008-033: Input includes action mapped from capability and method family.
- IP-008-034: Input includes resource_ref, resource_type, source_system_id, ontology_projection_id, and data_class.
- IP-008-035: Input includes purpose, reason_code, workflow_run_id, and operator justification when required.
- IP-008-036: Input includes consent_state, consent_scope, consent_version, and stricter-rule decision from consent-graph where applicable.
- IP-008-037: Input includes emergency_context for break-glass and emergency-services bypass only.
- IP-008-038: Input includes dealset_ref, buyer_tenant, provider_tenant, settlement_state, and marketplace flow type when commercial obligations apply.
- IP-008-039: Input includes credential_lease_ref and lease_scope when a vendor adapter will be invoked.
- IP-008-040: Input includes risk signals from abuse-defence for bot, spoof, scrape, enumeration, replay, and credential stuffing cases.
- IP-008-041: Input includes trace_id, request_id, idempotency_key, and caller_service_id.
- IP-008-042: Input includes policy_fragment_id, fragment_version, fragment_hash, and soak cohort.
- IP-008-043: Input includes requested response shape: raw, projection, metadata-only, redacted, or evidence-only.
- IP-008-044: Input includes export target class, recipient class, and audit export class for export calls.
- IP-008-045: Input includes replay_batch_id and prior_decision_ref for replay calls.

## Decision outputs
- IP-008-046: Output includes decision_id, decision_state, deny_reason_class, permit_scope, and evaluated_at timestamp.
- IP-008-047: Output includes policy_fragment_id, fragment_version, fragment_hash, and soak cohort.
- IP-008-048: Output includes audit_event_class and audit_payload_ref for ADR-0263 emission.
- IP-008-049: Output includes refusal_event_ref when decision_state is deny, abstain, anomaly, or missing-policy.
- IP-008-050: Output includes permitted_data_classes and permitted_response_shape.
- IP-008-051: Output includes permitted_cells and cross_cell_materialization rule.
- IP-008-052: Output includes permitted_credential_scope when a credential lease can be used.
- IP-008-053: Output includes required_human_review when policy permits only after review.
- IP-008-054: Output includes required_friction when abuse-defence raises risk but does not deny.
- IP-008-055: Output includes break_glass_expiry and review_workflow_id for emergency permit.
- IP-008-056: Output includes dealset_required, dealset_ref, and settlement_refusal_ref when marketplace context fails.
- IP-008-057: Output includes stricter_pack_applied and pack_delta_ref when packs alter base policy.
- IP-008-058: Output includes redaction_profile for returned data.
- IP-008-059: Output includes cacheability class and maximum TTL.
- IP-008-060: Output includes retryability for clients and workers.

## Capability action map
- IP-008-061: fhir-read.read-projection covers FHIR resource lookup after tenant, consent, and residency checks.
- IP-008-062: fhir-read.search-projection covers bounded search and must deny enumeration-shaped queries.
- IP-008-063: fhir-read.export-bundle covers export to authorized recipients and must evaluate export pack rules.
- IP-008-064: hl7-route.accept-message covers canonical listener ingestion after source-system scope validation.
- IP-008-065: hl7-route.route-message covers destination routing after transform, tenant, and consent checks.
- IP-008-066: hl7-route.replay-message covers idempotent replay after prior decision reference is checked.
- IP-008-067: break-glass-authorize.open covers emergency scoped access and requires reason, expiry, reviewer route, and audit closeout.
- IP-008-068: break-glass-authorize.close covers post-event review and evidence completion.
- IP-008-069: consent-sync.sync covers consent import, update, revocation, and conflict state.
- IP-008-070: consent-sync.resolve-conflict covers stricter-result selection and human justification.
- IP-008-071: ehr-provenance-seal.create covers provenance sealing after source and transform checks.
- IP-008-072: ehr-provenance-seal.verify covers evidence verification without signing material.
- IP-008-073: patient-match-review.queue covers candidate queueing with score band and source constraints.
- IP-008-074: patient-match-review.decide covers human review decisions and correction workflow creation.
- IP-008-075: operations.replay-status covers evidence-only status reads for backfill and replay.

## Cache and consistency
- IP-008-076: Permit decisions for mutations are not reused across commands.
- IP-008-077: Read permits may be cached only when policy fragment, consent state, pack state, principal state, and tenant state are unchanged.
- IP-008-078: Break-glass permits are never cached beyond their explicit expiry.
- IP-008-079: Abuse-friction decisions are never cached across source IP, workload identity, or risk cohort.
- IP-008-080: Credential lease permits are bounded by the shorter of policy TTL and OpenBao lease TTL.
- IP-008-081: DealSet settlement permits are invalidated when settlement state changes.
- IP-008-082: Consent sync permits are invalidated on consent_graph_version change.
- IP-008-083: Data residency permits are invalidated on pack overlay change.
- IP-008-084: Policy fragment cache entries are keyed by fragment hash and soak cohort.
- IP-008-085: Cache hit evidence must include decision_id of the original decision and cache_decision_id for the reuse event.
- IP-008-086: Cache lookup cannot precede tenant and principal validation.
- IP-008-087: Cache misses must not call external vendor systems.
- IP-008-088: Cache poisoning attempts trigger abuse-defence evidence.
- IP-008-089: Cache TTLs are observable through metrics without tenant cardinality.
- IP-008-090: Cache rollback follows ADR-0294 by reverting to the previous soaked fragment set.

## Failure handling
- IP-008-091: Missing tenant_id returns invalid_request and emits refusal evidence.
- IP-008-092: Missing principal_id returns invalid_request and emits refusal evidence.
- IP-008-093: Missing data_class returns invalid_request and emits refusal evidence.
- IP-008-094: Missing purpose returns invalid_request and emits refusal evidence.
- IP-008-095: Missing pack overlay returns failed_precondition for protected clinical data.
- IP-008-096: Missing consent state returns deny for PHI reads and mutations.
- IP-008-097: Consent conflict returns human_review_required or deny based on pack rule.
- IP-008-098: Residency conflict returns deny for materialized data and may permit metadata-only if policy allows.
- IP-008-099: DealSet missing returns settlement_refused.
- IP-008-100: Abuse high risk returns deny or friction_required depending on action risk class.
- IP-008-101: Fragment anomaly returns policy_anomaly and fails closed for mutation.
- IP-008-102: Cedar engine unavailable returns policy_unavailable and fails closed for mutation.
- IP-008-103: Audit-chain unavailable returns audit_required_unavailable for high-risk mutation.
- IP-008-104: Credential lease mismatch returns credential_scope_denied.
- IP-008-105: Replay prior decision mismatch returns replay_policy_mismatch.

## Observability and evidence
- IP-008-106: Metrics include policy_eval_total, policy_eval_denied_total, policy_eval_latency, policy_fragment_anomaly_total, and policy_cache_hit_total.
- IP-008-107: Metrics label capability, action, data_class, decision_state, pack_class, and risk_class.
- IP-008-108: Metrics do not label raw tenant_id, patient id, resource id, or source-system patient id.
- IP-008-109: Traces include policy span before domain command span.
- IP-008-110: Traces include fragment hash, decision state, and audit event reference.
- IP-008-111: Logs include redacted reason class and operator action reference.
- IP-008-112: Audit-chain events include tenant, principal, action, data_class, policy decision, fragment, and outcome.
- IP-008-113: Denial events distinguish policy, consent, residency, settlement, abuse, credential, and validation.
- IP-008-114: Dashboards/local-policy-decisions.json is the first dashboard evidence target.
- IP-008-115: Dashboards/abuse-defence-outcomes.json is the abuse evidence target.
- IP-008-116: SLOs/policy-decision-latency.openslo.yaml is the decision latency target.
- IP-008-117: Runbooks/local-hipaa-access-review-delay.md and local-breakglass-audit-review.md are post-review targets.
- IP-008-118: Incident-response evidence must link decision_id to workflow_run_id and audit_event_ref.
- IP-008-119: DPIA evidence must show how PHI-minimizing policy inputs avoid unnecessary raw clinical data exposure.
- IP-008-120: Threat-model evidence must show policy bypass, fragment poisoning, cache poisoning, and replay denial paths.

## Benchmark displacement
- IP-008-121: Redox is displaced by policy-first exchange where every request carries tenant, purpose, data_class, consent, and audit proof rather than just normalized API access.
- IP-008-122: Rhapsody is displaced by policy library reuse across REST, gRPC, worker, and replay lanes rather than route-engine-specific rule wiring.
- IP-008-123: InterSystems IRIS for Health is displaced by externalized Cedar decisions and explicit audit events rather than database-resident access logic.
- IP-008-124: Lyniate/Corepoint is displaced by strict default-deny and fragment rollback evidence rather than channel-level allowlists.
- IP-008-125: Mirth is displaced by typed, testable policy requests rather than script fragments embedded in transformations.
- IP-008-126: NextGate is displaced by patient-match policies that require human review provenance and correction workflow linkage.
- IP-008-127: Health Catalyst is displaced by proving policy, consent, and residency before analytics or evidence projection.
- IP-008-128: Epic parity pressure is handled through FHIR consent and purpose enforcement at resource read time.
- IP-008-129: Cerner parity pressure is handled through HL7 route authorization and bounded replay.
- IP-008-130: Veeva parity pressure is handled through GxP-ready decision evidence and immutable provenance.

## Implementation steps
- IP-008-131: Define HealthcarePolicyRequest and HealthcarePolicyDecision in the service code boundary that owns policy library integration.
- IP-008-132: Map every capability action to Cedar action strings and policy files.
- IP-008-133: Bind clinical-interoperability-authorization.cedar as the default service authorization fragment.
- IP-008-134: Bind local-fhir-exchange-consent.cedar for FHIR reads and exports.
- IP-008-135: Bind local-patient-consent-sync.cedar for consent synchronization.
- IP-008-136: Bind local-breakglass-access-control.cedar for break-glass authorization.
- IP-008-137: Bind abuse-defence.cedar for risk signal evaluation.
- IP-008-138: Bind data-residency.md rules into machine-checkable residency input fields.
- IP-008-139: Add fragment schema validation against specs/cedar-fragment-schema.json.
- IP-008-140: Add decision audit emission before domain dispatch for permits and before response for denials.
- IP-008-141: Add negative tests for missing tenant, missing purpose, missing consent, missing DealSet, and fragment anomaly.
- IP-008-142: Add property tests for monotonic stricter-pack behavior.
- IP-008-143: Add replay tests that verify prior decision refs cannot authorize new scope.
- IP-008-144: Add cache tests that validate invalidation on consent, pack, fragment, principal, and settlement changes.
- IP-008-145: Add metrics tests for no raw tenant labels.

## Tests and evidence
- IP-008-146: Contract evidence: each policy input field maps to a request envelope field from REST, gRPC, or worker caller.
- IP-008-147: Policy evidence: fhir-read read without consent denies.
- IP-008-148: Policy evidence: hl7-route replay without prior decision denies.
- IP-008-149: Policy evidence: break-glass without expiry denies.
- IP-008-150: Policy evidence: consent-sync conflict requires stricter result or review.
- IP-008-151: Policy evidence: ehr-provenance-seal without source provenance denies.
- IP-008-152: Policy evidence: patient-match-review decide without human reviewer denies.
- IP-008-153: Residency evidence: cross-cell materialization is denied unless pack allows.
- IP-008-154: Settlement evidence: DealSet missing denies commercial partner action.
- IP-008-155: Abuse evidence: high-risk replay shape denies or requires friction.
- IP-008-156: Fragment evidence: anomaly rolls back to previous soaked fragment for mutation.
- IP-008-157: Cache evidence: permit cache invalidates when consent_graph_version changes.
- IP-008-158: Cache evidence: credential scope does not outlive credential lease.
- IP-008-159: Observability evidence: denial event includes refusal_event_ref and no PHI.
- IP-008-160: Audit evidence: permitted mutation links decision_id to audit_chain_ref.

## Rollback
- IP-008-161: If the policy library emits incomplete decisions, block mutation integration and keep previous inline gate behavior.
- IP-008-162: If fragment validation fails, roll back to prior soaked fragment per ADR-0294.
- IP-008-163: If cache invalidation fails, disable permit caching and run uncached evaluation.
- IP-008-164: If metrics expose sensitive labels, disable policy metric export until labels are remediated.
- IP-008-165: If denial mapping is wrong, fail closed and route affected methods through manual review.
- IP-008-166: If DealSet policy produces false permits, block commercial partner calls.
- IP-008-167: If abuse risk causes false friction on clean healthcare emergency flows, restore clean-path bypass while preserving emergency evidence.
- IP-008-168: If consent versioning mismatches, deny PHI reads until consent-graph returns stable version.
- IP-008-169: If residency mapping drifts, force metadata-only responses until pack overlay evidence is repaired.
- IP-008-170: Rollback evidence must include fragment id, affected capability, decision ids, refusal ids, and operator action.

## Acceptance criteria
- IP-008-171: All healthcare-integration entry paths call the same policy library shape before side effects.
- IP-008-172: All mutations fail closed when the policy library is unavailable.
- IP-008-173: All decision outputs include decision_id, fragment version, permit scope, and audit evidence.
- IP-008-174: All denial outputs include refusal_event_ref and reason class.
- IP-008-175: All capability actions are mapped to Cedar actions.
- IP-008-176: All pack overlay decisions apply the stricter rule when conflicts arise.
- IP-008-177: All DealSet-governed actions include settlement context.
- IP-008-178: All credential-bound actions include lease scope.
- IP-008-179: All replay actions include prior decision references.
- IP-008-180: All abuse-sensitive actions include risk signal inputs.
- IP-008-181: All cacheable reads publish cacheability and TTL.
- IP-008-182: No cached permit survives consent, pack, fragment, principal, credential, or settlement state changes.
- IP-008-183: No policy metric exposes raw tenant, patient, resource, or source-system identifiers.
- IP-008-184: No denial silently degrades into dependency outage.
- IP-008-185: No policy bypass exists for gRPC, worker, replay, or local operation paths.
- IP-008-186: Each benchmark displacement claim maps to a policy control and test.
- IP-008-187: Each ADR in the binding set is represented by a field, behavior, test, or rollback rule.
- IP-008-188: ADR-0321 remains cited as doctrine and is not edited by this IP.
- IP-008-189: The plan supports future implementation without modifying unassigned files in this batch.
- IP-008-190: The final implementation can be verified through policy tests, contract tests, metrics tests, and audit evidence.

## Citation summary
- IP-008-191: PRD.md supplies tenant-scoped, Cedar-gated, pack-aware, audit-chain-sealed functional requirements.
- IP-008-192: ARCHITECTURE.md supplies bounded contexts, dependency topology, and failure modes.
- IP-008-193: manifest.json supplies binding ADRs, benchmark roster, contract versions, and dependency lists.
- IP-008-194: policy/clinical-interoperability-authorization.cedar anchors default clinical authorization.
- IP-008-195: policy/abuse-defence.cedar anchors risk signal integration.
- IP-008-196: policy/data-residency.md anchors residency decision input and fallback behavior.
- IP-008-197: policies/local-fhir-exchange-consent.cedar anchors FHIR read/export consent checks.
- IP-008-198: policies/local-patient-consent-sync.cedar anchors consent sync policy behavior.
- IP-008-199: policies/local-breakglass-access-control.cedar anchors break-glass policy behavior.
- IP-008-200: specs/cedar-fragment-schema.json anchors fragment validation.
- IP-008-201: dashboards/local-policy-decisions.json anchors decision observability.
- IP-008-202: dashboards/abuse-defence-outcomes.json anchors abuse outcome evidence.
- IP-008-203: slos/policy-decision-latency.openslo.yaml anchors policy latency evidence.
- IP-008-204: threat-model.md anchors bypass, cache, fragment, and replay abuse review.
- IP-008-205: ADR-0321 remains cited as existing B2B leader coverage doctrine only; this IP does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-008-policy-eval-library-binding.md:33` - - IP-008-018: The library is not allowed to read raw PHI fields unless data_class and purpose explicitly require field-level policy.; `microservices/healthcare-integration/IP-008-policy-eval-library-binding.md:121` - - IP-008-096: Missing consent state returns deny for PHI reads and mutations..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-008-policy-eval-library-binding.md:67` - - IP-008-048: Output includes audit_event_class and audit_payload_ref for ADR-0263 emission.; `microservices/healthcare-integration/IP-008-policy-eval-library-binding.md:171` - - IP-008-140: Add decision audit emission before domain dispatch for permits and before response for denials..
