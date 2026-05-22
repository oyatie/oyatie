# IP-007 Healthcare Integration grpc-internal-surface

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-007-grpc-internal-surface.md
Batch: C healthcare-integration IP deepening
Status: implementation-plan-ready
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json; microservices/healthcare-integration/contracts/healthcare-integration-v1.proto; microservices/healthcare-integration/contracts/local-operations-v1.proto; microservices/healthcare-integration/capabilities/fhir-read.yaml; microservices/healthcare-integration/capabilities/hl7-route.yaml; microservices/healthcare-integration/capabilities/break-glass-authorize.yaml; microservices/healthcare-integration/capabilities/consent-sync.yaml; microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml; microservices/healthcare-integration/capabilities/patient-match-review.yaml

## Objective
- IP-007-001: Define the internal gRPC surface that lets healthcare-integration workers and adjacent microservices invoke clinical interoperability work without bypassing REST, Cedar, tenancy, audit-chain, marketplace DealSet, or pack overlay rules.
- IP-007-002: Treat gRPC as an internal synchronous command/query envelope only; public partner traffic remains bound to OpenAPI surfaces and async work remains bound to AsyncAPI surfaces from the service contract directory.
- IP-007-003: Preserve ADR-0105 layer separation by keeping proto message schemas in contract space, request validation in rest/api adapters, orchestration in application/usecase, invariants in domain/kernel, and vendor transport in adapters.
- IP-007-004: Preserve ADR-0131 flat microservice layout by keeping all healthcare-integration proto, runbook, SLO, and policy evidence under microservices/healthcare-integration rather than creating vendor folders.
- IP-007-005: Preserve ADR-0242 and ADR-0244 by requiring tenant_id, principal_id, audience_type, home_cell, jurisdiction_code, purpose, and data_class on every unary and streaming call.
- IP-007-006: Preserve ADR-0243 and ADR-0246 by requiring a Cedar decision reference before any command leaves the gRPC adapter toward a domain handler.
- IP-007-007: Preserve ADR-0253-amendment by requiring TLS 1.3, HTTP/3-first edge posture, strict fallback metadata, and PQC/ECH evidence where the internal call crosses a mesh boundary.
- IP-007-008: Preserve ADR-0257 by keeping ontology reads library-first and returning ontology projection references rather than raw vendor state.
- IP-007-009: Preserve ADR-0258 by making proto package, service name, method name, and message version explicit in every compatibility rule.
- IP-007-010: Preserve ADR-0263 by requiring trace_id, span_id, audit_event_class, audit_chain_ref, and refusal_event_ref on the gRPC metadata envelope.
- IP-007-011: Preserve ADR-0294 by attaching Cedar fragment version and soak cohort to gRPC calls that use policy fragments.
- IP-007-012: Preserve ADR-0296 by passing credential lease handles rather than raw EHR credentials.
- IP-007-013: Preserve ADR-0297 by keeping bot, spoof, scrape, and replay controls visible even for internal calls sourced from workers.
- IP-007-014: Preserve ADR-0314 by carrying DealSet settlement references on partner-driven clinical exchange even when the synchronous hop is internal.
- IP-007-015: Preserve ADR-0321 by displacing category leaders through stronger tenant isolation, auditability, policy-first execution, and migration-ready contracts rather than imitating their suite boundaries.

## Current thin content replacement
- IP-007-016: The previous file repeated generic healthcare-integration rows and did not specify proto packages, call contracts, compatibility rules, tenant metadata, or benchmark displacement.
- IP-007-017: This rewrite binds the plan directly to contracts/healthcare-integration-v1.proto and contracts/local-operations-v1.proto so implementation can be reviewed against a concrete gRPC surface.
- IP-007-018: The plan treats fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, and patient-match-review as first-class methods or method families, matching the capability records.
- IP-007-019: The plan does not claim that the proto files are complete; it defines the acceptance standard for deepening them without editing those files in this batch.
- IP-007-020: The plan keeps benchmark discussion on displacement and control coverage, not on vendor feature copying.

## Surface boundary
- IP-007-021: Public REST remains the ingress of record for tenant administrators, partner API clients, and marketplace flows.
- IP-007-022: Internal gRPC is allowed for workflow-engine orchestration, replay workers, audit-chain callbacks, consent-graph lookups, and ontology projection calls.
- IP-007-023: Internal gRPC is not allowed for direct EHR vendor ingress, patient portal ingress, unsupported vendor webhooks, or human browser traffic.
- IP-007-024: Any gRPC command that mutates clinical state must include the same idempotency key required by PRD functional requirements.
- IP-007-025: Any gRPC query that reads PHI-bearing or quasi-identifier data must include purpose, data_class, and pack overlay context.
- IP-007-026: Streaming gRPC is reserved for bounded replay status, HL7 batch acknowledgement windows, and audit-chain evidence transfer.
- IP-007-027: Streaming gRPC is not permitted for unbounded raw clinical feed replication.
- IP-007-028: The service must expose health, readiness, and version methods that return no tenant data.
- IP-007-029: Version methods must state supported proto package version, minimum client version, and deprecation timestamp.
- IP-007-030: Administrative gRPC methods must be narrower than REST admin actions and must not bypass marketplace settlement or policy gates.
- IP-007-031: Internal clients must identify their calling service using service_id plus workload identity, not caller-supplied free text.
- IP-007-032: Workload identity must be checked against mesh identity and policy allowlist before request deserialization reaches domain state.
- IP-007-033: gRPC metadata must not carry PHI fields; PHI stays inside typed request bodies with data_class tags and redaction rules.
- IP-007-034: gRPC errors must not echo raw FHIR resources, HL7 segments, patient identifiers, or unredacted consent text.
- IP-007-035: The surface must support explicit degraded-mode responses for audit-chain, consent-graph, and ontology dependency outages.

## Method families
- IP-007-036: FhirReadService.GetResource returns a tenant-scoped resource projection, not the raw source-system document.
- IP-007-037: FhirReadService.SearchResources requires a bounded result window, search purpose, jurisdiction, and policy decision id.
- IP-007-038: FhirReadService.ExportBundle requires audit_chain_ref, export_reason, recipient class, and pack export policy.
- IP-007-039: Hl7RouteService.RouteMessage accepts canonicalized HL7 message references and never accepts an unauthenticated raw listener payload.
- IP-007-040: Hl7RouteService.AcknowledgeMessage returns ACK/NACK evidence that includes route policy, transform version, and tenant cell.
- IP-007-041: Hl7RouteService.ReplayMessage requires replay_batch_id, source_event_id, idempotency_key, and prior denial evidence when retrying a rejected message.
- IP-007-042: BreakGlassService.AuthorizeBreakGlass requires emergency reason, reviewer route, expiry, patient-scope bounds, and post-event review target.
- IP-007-043: BreakGlassService.CloseBreakGlass requires audit completion and cannot silently expire without evidence.
- IP-007-044: ConsentSyncService.SyncConsent requires source consent reference, normalized consent purpose, revocation state, and consent-graph version.
- IP-007-045: ConsentSyncService.ResolveConflict requires both source versions, the selected stricter result, and operator justification.
- IP-007-046: EhrProvenanceService.SealProvenance requires source_system_id, transform_id, ontology_projection_id, and cryptographic evidence reference.
- IP-007-047: EhrProvenanceService.VerifySeal returns verification state and never returns private signing material.
- IP-007-048: PatientMatchService.QueueReview accepts candidate references, match score bands, and review workflow target.
- IP-007-049: PatientMatchService.DecideReview records human decision, policy context, and downstream correction workflow references.
- IP-007-050: OperationsService.GetReplayStatus reads backfill-replay.md semantics without allowing replay mutation from an unaudited caller.

## Request envelope
- IP-007-051: Every request envelope includes tenant_id, principal_id, audience_type, purpose, data_class, home_cell, jurisdiction_code, and pack_ids.
- IP-007-052: Every request envelope includes trace_id, span_id, idempotency_key for commands, request_time, and caller_service_id.
- IP-007-053: Every request envelope includes cedar_decision_id, cedar_fragment_version, policy_soak_cohort, and policy_denial_handling mode.
- IP-007-054: Every request envelope includes audit_chain_ref or audit_chain_deferred_ref depending on the method risk class.
- IP-007-055: Every request envelope includes dealset_ref when the call is triggered by a billable partner, marketplace, or migration transaction.
- IP-007-056: Every request envelope includes ontology_projection_id when reading or mutating projected clinical objects.
- IP-007-057: Every request envelope includes credential_lease_ref when a downstream vendor or EHR credential will be used.
- IP-007-058: Every request envelope includes replay_batch_id when invoked from backfill, migration, or reconciliation workers.
- IP-007-059: Every request envelope includes emergency_context only for break-glass methods and emergency-services bypass flows.
- IP-007-060: Every request envelope includes abuse_context for anomaly, bot, spoof, scrape, or replay-risk scoring under ADR-0297.
- IP-007-061: Missing tenant_id, principal_id, data_class, purpose, or policy decision is INVALID_ARGUMENT before domain command handling.
- IP-007-062: Missing audit_chain_ref on high-risk mutations is FAILED_PRECONDITION, not an asynchronous warning.
- IP-007-063: Missing credential_lease_ref on vendor-bound methods is PERMISSION_DENIED, not an adapter-level retry.
- IP-007-064: Missing dealset_ref on a commercial partner call is FAILED_PRECONDITION and creates settlement refusal evidence.
- IP-007-065: Metadata mismatch between gRPC metadata and typed body fails closed and emits an ADR-0263 audit event.

## Response envelope
- IP-007-066: Success responses include resource_ref, event_ref, audit_event_ref, policy_decision_ref, and projection_version where applicable.
- IP-007-067: Success responses for mutation methods include idempotency_result with created, replayed, duplicate, or rejected status.
- IP-007-068: Success responses for HL7 route calls include ack_code, route_id, transform_id, destination_class, and operator-visible status.
- IP-007-069: Success responses for FHIR read calls include projection_version, source_last_seen_at, and stale_region_metadata where applicable.
- IP-007-070: Success responses for consent sync include consent_state, stricter_rule_applied, conflict_status, and consent_graph_version.
- IP-007-071: Success responses for break-glass include expiry, scope, review_workflow_id, and audit_closeout_deadline.
- IP-007-072: Success responses for provenance sealing include seal_id, signing_key_ref, verification_state, and exportable evidence bundle id.
- IP-007-073: Error responses use canonical gRPC status plus healthcare_error_code.
- IP-007-074: Error responses include refusal_event_ref when policy, residency, consent, settlement, or abuse gates deny the call.
- IP-007-075: Error responses include retry_after only when retry is safe under idempotency and policy version rules.
- IP-007-076: Error responses include degraded_dependency only for dependency outages, not policy denials.
- IP-007-077: Error responses never include raw PHI, raw HL7 message text, raw FHIR JSON, or credential material.
- IP-007-078: Error responses include operator_action_ref when runbooks can remediate the condition.
- IP-007-079: Error responses for replay operations include replay_batch_id and last_safe_event_id.
- IP-007-080: Error responses for version mismatch include supported_versions and deprecation policy link.

## Authorization and policy binding
- IP-007-081: gRPC method authorization maps method family plus data_class to Cedar action names.
- IP-007-082: FHIR read methods map to healthcare-integration.fhir-read actions.
- IP-007-083: HL7 route methods map to healthcare-integration.hl7-route actions.
- IP-007-084: Break-glass methods map to healthcare-integration.break-glass-authorize actions.
- IP-007-085: Consent sync methods map to healthcare-integration.consent-sync actions.
- IP-007-086: EHR provenance methods map to healthcare-integration.ehr-provenance-seal actions.
- IP-007-087: Patient match methods map to healthcare-integration.patient-match-review actions.
- IP-007-088: Policy must evaluate the caller service, human principal, tenant, purpose, data class, pack ids, and home cell.
- IP-007-089: Policy denial must return PERMISSION_DENIED with refusal_event_ref and no domain side effect.
- IP-007-090: Policy abstain, missing fragment, or fragment anomaly must fail closed for mutations.
- IP-007-091: Read-only calls may return degraded unavailable only when no sensitive data is disclosed.
- IP-007-092: Break-glass calls must include higher-friction evidence and post-review workflow references.
- IP-007-093: Emergency-services bypass flow remains separate and must not be conflated with generic break-glass.
- IP-007-094: CI and auditor gRPC access must use scoped policies from policy/ci-scope.cedar and policy/auditor-scope.cedar.
- IP-007-095: Data residency policy is evaluated before cross-cell response materialization.

## Compatibility and versioning
- IP-007-096: Proto packages use explicit healthcare.integration.v1 naming and cannot overload methods by vendor.
- IP-007-097: New fields must be optional or additive until a major version migration is declared under ADR-0258.
- IP-007-098: Removed fields require deprecation notice, compatibility soak, generated client update, and replay fixture update.
- IP-007-099: Field numbers cannot be reused after deletion.
- IP-007-100: Enum expansion requires unknown-value handling in clients.
- IP-007-101: Method additions require contract tests for caller metadata, policy mapping, response evidence, and error redaction.
- IP-007-102: Method removal requires an ADR-0258 migration record and downstream workflow-engine call audit.
- IP-007-103: Streaming method changes require backpressure tests and audit-chain loss tests.
- IP-007-104: Deadline defaults must be method-specific and must not allow unbounded clinical feed reads.
- IP-007-105: Retry policies must be idempotency-aware and must not retry denied calls.
- IP-007-106: Generated SDKs must preserve strongly typed metadata rather than loose maps.
- IP-007-107: Internal client generation must align with sdk-plan.md and IP-019 when that plan is executed.
- IP-007-108: Local operations proto must stay separate from public clinical contract proto if operational methods are not product APIs.
- IP-007-109: Compatibility evidence must cite both proto diff and replay fixture diff.
- IP-007-110: Deprecated method calls emit warning metrics before removal.

## Observability and audit
- IP-007-111: Every method emits request_total, request_denied_total, request_latency, and dependency_latency metrics without raw tenant_id labels.
- IP-007-112: Every mutation emits an audit-chain event with method name, capability, tenant, principal, policy decision, and data class.
- IP-007-113: FHIR read calls emit resource type, projection version, source freshness, and pack overlay class.
- IP-007-114: HL7 route calls emit route id, transform version, ACK class, destination class, and replay status.
- IP-007-115: Break-glass calls emit emergency reason class, scope size, expiry, reviewer route, and closeout status.
- IP-007-116: Consent sync calls emit source consent version, resolved state, conflict flag, and stricter-rule decision.
- IP-007-117: Provenance seal calls emit source-system id class, transform id, seal verification, and evidence bundle id.
- IP-007-118: Patient match calls emit score band, candidate count band, review queue id, and final decision class.
- IP-007-119: Audit events follow ADR-0263 and link to observability-audit-events IP-011 evidence.
- IP-007-120: Traces must include span links to REST ingress, workflow-engine run, vendor adapter call, and audit-chain write.
- IP-007-121: Logs redact PHI and identify operator action through stable references.
- IP-007-122: Dashboards/slo-and-error-budget.json and dashboards/local-domain-throughput.json become review targets for method-level signal coverage.
- IP-007-123: SLOs/read-latency.openslo.yaml and slos/write-latency.openslo.yaml set initial latency evidence expectations.
- IP-007-124: Audit-chain backpressure stops high-risk mutation methods before evidence loss.
- IP-007-125: Observability must distinguish policy denial, consent denial, residency denial, settlement denial, and abuse denial.

## Benchmark displacement
- IP-007-126: Redox displacement comes from policy-aware internal calls that preserve tenant, purpose, data_class, settlement, and audit refs rather than acting as a neutral pipe.
- IP-007-127: Rhapsody displacement comes from typed proto and replay-safe idempotency that prevent route-engine convenience from hiding policy and provenance.
- IP-007-128: InterSystems IRIS for Health displacement comes from Oyatie flat-service ownership and explicit ontology projection instead of database-centered suite gravity.
- IP-007-129: Lyniate/Corepoint displacement comes from policy, DealSet, and audit evidence at each call boundary instead of adapter-centric channel configuration.
- IP-007-130: Mirth Connect displacement comes from governed internal contracts, domain invariants, and signed evidence rather than scriptable channel transforms.
- IP-007-131: NextGate displacement comes from patient-match review methods that expose human decision provenance and tenant-scoped correction workflows.
- IP-007-132: Health Catalyst displacement comes from evidence-preserving clinical exchange and replay controls before analytics projection.
- IP-007-133: Epic and Cerner parity pressure is handled as FHIR/HL7 depth, not as an EHR-owned boundary.
- IP-007-134: Allscripts parity pressure is handled as route and consent interoperability, not as a legacy integration channel.
- IP-007-135: Veeva parity pressure is handled as regulated provenance and GxP evidence, not as CRM suite coupling.

## Implementation steps
- IP-007-136: Inventory healthcare-integration-v1.proto and local-operations-v1.proto for current package, service, method, message, enum, and metadata coverage.
- IP-007-137: Add or deepen RequestContext and ResponseEvidence messages with fields listed in this IP.
- IP-007-138: Add method-level option comments that map each method to capability record, Cedar action, audit event class, and data class.
- IP-007-139: Add command methods for fhir-read, hl7-route, break-glass-authorize, consent-sync, ehr-provenance-seal, and patient-match-review only where synchronous internal invocation is justified.
- IP-007-140: Add operation-status methods for replay, audit closeout, and dependency degraded mode.
- IP-007-141: Add canonical error message structures for policy, consent, residency, settlement, abuse, validation, dependency, and version errors.
- IP-007-142: Add generated contract snapshot checks to prevent field-number reuse.
- IP-007-143: Add unit tests for metadata/body mismatch failure.
- IP-007-144: Add policy tests that deny missing tenant_id, principal_id, data_class, purpose, and Cedar decision id.
- IP-007-145: Add redaction tests for gRPC error payloads.
- IP-007-146: Add idempotent replay tests for duplicate commands.
- IP-007-147: Add streaming backpressure tests for replay status and HL7 ACK windows.
- IP-007-148: Add SLO wiring for read and write latency methods.
- IP-007-149: Add audit-chain fixture expectations for each mutation.
- IP-007-150: Add compatibility notes for generated internal clients and SDK plan alignment.

## Tests and evidence
- IP-007-151: Contract evidence: proto diff shows package, method, message, field, enum, and service changes.
- IP-007-152: Contract evidence: field-number reuse check passes for changed proto files.
- IP-007-153: Contract evidence: generated client smoke test compiles against the changed proto.
- IP-007-154: Authorization evidence: missing tenant_id returns INVALID_ARGUMENT before domain execution.
- IP-007-155: Authorization evidence: Cedar denial returns PERMISSION_DENIED with refusal_event_ref.
- IP-007-156: Authorization evidence: audit-chain outage blocks break-glass mutation.
- IP-007-157: Authorization evidence: credential-bound method fails when credential_lease_ref is missing.
- IP-007-158: Residency evidence: cross-cell read returns metadata-only unless pack permits materialization.
- IP-007-159: Settlement evidence: commercial partner call fails closed when dealset_ref is missing.
- IP-007-160: Abuse evidence: replay-risk score triggers friction or denial under ADR-0297 rules.
- IP-007-161: Observability evidence: metrics do not expose raw tenant_id labels.
- IP-007-162: Observability evidence: audit event includes method, capability, data class, policy decision, and trace context.
- IP-007-163: Replay evidence: duplicate idempotency key returns prior result.
- IP-007-164: Streaming evidence: replay status stream respects bounded window and deadline.
- IP-007-165: Redaction evidence: error payloads contain no raw PHI, HL7 text, FHIR JSON, or credentials.

## Rollback
- IP-007-166: Proto rollout uses additive fields first, then generated internal clients, then server enforcement, then optional deprecation.
- IP-007-167: If a new method causes policy mismatch, disable the method route and keep existing REST/AsyncAPI paths active.
- IP-007-168: If a new field causes client incompatibility, mark it ignored server-side and keep field number reserved.
- IP-007-169: If audit-chain evidence is incomplete, block mutation methods and allow safe read-only degraded responses.
- IP-007-170: If credential lease binding fails, disable vendor-bound method families and preserve non-vendor status methods.
- IP-007-171: If DealSet evidence is missing, route commercial partner calls to settlement refusal rather than silent success.
- IP-007-172: If streaming backpressure fails, disable streaming methods and require polling status methods.
- IP-007-173: If generated clients drift, pin clients to the last accepted proto snapshot and open a compatibility fix.
- IP-007-174: If policy fragment soak fails, rollback to prior soaked Cedar fragment per ADR-0294.
- IP-007-175: Rollback evidence must cite method family, proto version, client version, audit event ids, and operator action.

## Acceptance criteria
- IP-007-176: Each gRPC method maps to an explicit capability record and Cedar action.
- IP-007-177: Each gRPC method includes tenant, principal, audience, purpose, data_class, home_cell, and jurisdiction context.
- IP-007-178: Each mutation method requires idempotency, policy decision, audit-chain ref, and data residency posture.
- IP-007-179: Each vendor-bound method uses credential lease references only.
- IP-007-180: Each commercial partner-triggered method carries DealSet evidence.
- IP-007-181: Each method has redacted, typed error responses.
- IP-007-182: Each high-risk method has runbook-linked operator actions.
- IP-007-183: Each method emits metrics, traces, logs, and audit-chain events under ADR-0263.
- IP-007-184: Each proto change is versioned under ADR-0258.
- IP-007-185: Each policy fragment change includes soak and rollback evidence under ADR-0294.
- IP-007-186: Each abuse-sensitive method includes ADR-0297 friction or denial behavior.
- IP-007-187: Each ontology response cites projection version under ADR-0257.
- IP-007-188: Each internal client identifies workload identity and caller service.
- IP-007-189: Each degraded-mode response distinguishes dependency outage from policy denial.
- IP-007-190: Each benchmark displacement claim is traceable to a concrete method, control, or evidence field.

## Citation summary
- IP-007-191: PRD.md supplies the service problem, user stories, functional requirements, pack overlay requirements, and quality gates.
- IP-007-192: ARCHITECTURE.md supplies the ADR-0105 layer map, bounded contexts, dependencies, and failure modes.
- IP-007-193: manifest.json supplies tier, audience_type, binding ADRs, benchmark roster, dependency list, cell eligibility, and contract-version doctrine.
- IP-007-194: contracts/healthcare-integration-v1.proto is the primary clinical gRPC contract reference.
- IP-007-195: contracts/local-operations-v1.proto is the local operations gRPC contract reference.
- IP-007-196: capabilities/fhir-read.yaml supplies fhir-read tenant scope, marketplace settlement, and policy mode.
- IP-007-197: capabilities/hl7-route.yaml supplies hl7-route tenant scope, marketplace settlement, and policy mode.
- IP-007-198: capabilities/break-glass-authorize.yaml supplies break-glass-authorize tenant scope, marketplace settlement, and policy mode.
- IP-007-199: capabilities/consent-sync.yaml supplies consent-sync tenant scope, marketplace settlement, and policy mode.
- IP-007-200: capabilities/ehr-provenance-seal.yaml supplies ehr-provenance-seal tenant scope, marketplace settlement, and policy mode.
- IP-007-201: capabilities/patient-match-review.yaml supplies patient-match-review tenant scope, marketplace settlement, and policy mode.
- IP-007-202: policy/ci-scope.cedar and policy/auditor-scope.cedar anchor scoped non-human access.
- IP-007-203: slos/read-latency.openslo.yaml and slos/write-latency.openslo.yaml anchor method latency expectations.
- IP-007-204: dashboards/slo-and-error-budget.json and dashboards/local-domain-throughput.json anchor operations review.
- IP-007-205: ADR-0321 remains cited as existing doctrine only; this IP does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-007-grpc-internal-surface.md:9` - Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json;...; `microservices/healthcare-integration/IP-007-grpc-internal-surface.md:30` - - IP-007-017: This rewrite binds the plan directly to contracts/healthcare-integration-v1.proto and contracts/local-operations-v1.proto so implementation can be review....
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-007-grpc-internal-surface.md:15` - - IP-007-004: Preserve ADR-0131 flat microservice layout by keeping all healthcare-integration proto, runbook, SLO, and policy evidence under microservices/healthcare-...; `microservices/healthcare-integration/IP-007-grpc-internal-surface.md:40` - - IP-007-025: Any gRPC query that reads PHI-bearing or quasi-identifier data must include purpose, data_class, and pack overlay context..
