# IP-026 Healthcare Integration HL7 ACK route custody

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Status: authoring-ready
Owner: axis-healthcare-integration
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabilities/hl7-route.yaml; microservices/healthcare-integration/contracts/openapi-v1.yaml; microservices/healthcare-integration/contracts/asyncapi-v1.yaml; microservices/healthcare-integration/contracts/healthcare-integration-v1.proto; microservices/healthcare-integration/policies/local-hl7-ingest-source-scope.cedar; microservices/healthcare-integration/runbooks/local-hl7-ack-latency-burn.md; microservices/healthcare-integration/runbooks/hl7-queue-backlog.md; microservices/healthcare-integration/slos/local-hl7-ack-latency.openslo.yaml
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Define an atomic implementation plan for HL7 route custody and ACK handling.
- Make every inbound HL7 message tenant-scoped before route evaluation.
- Make every ACK, NACK, retry, quarantine, and replay event audit-chain visible.
- Bind route custody to hl7-route capability records and ADR-0243 Cedar permits.
- Prevent interface-engine console state from becoming the authority.
- Preserve ADR-0105 layer separation between transport, usecase, domain, adapter, worker, and governance.
- Keep this IP documentation-only and limited to the assigned file.

## Inbound route model
- Route field 001: tenant_id is required before message parsing.
- Route field 002: principal_id is required before source-system credentials are requested.
- Route field 003: audience_type must be HEALTHCARE_OPERATOR or approved automated-worker class.
- Route field 004: purpose must name treatment, operations, payment, public-health, or approved pack-specific purpose.
- Route field 005: data_class must be hl7_message.
- Route field 006: source_system_id must identify the EHR, lab, pharmacy, payer, device, or clearinghouse endpoint.
- Route field 007: route_id must resolve to a tenant-owned route definition.
- Route field 008: route_version must be immutable during message custody.
- Route field 009: message_control_id must be normalized and preserved.
- Route field 010: sending_application must be captured as source metadata, not authority.
- Route field 011: sending_facility must be captured as source metadata, not tenant identity.
- Route field 012: receiving_application must map to the tenant route endpoint.
- Route field 013: receiving_facility must map to tenant sub-scope.
- Route field 014: message_type must bind to route policy.
- Route field 015: trigger_event must bind to route policy.
- Route field 016: processing_id must reject production data on non-production route where pack requires it.
- Route field 017: version_id must be recorded for transform compatibility.
- Route field 018: jurisdiction_code must be attached before persistence.
- Route field 019: home_cell must be attached before persistence.
- Route field 020: audit_event_class must be attached before ACK decision.

## ACK custody states
- State 001: received means bytes arrived and transport identity was recorded.
- State 002: scoped means tenant, principal, audience, purpose, and data_class passed shape validation.
- State 003: policy-evaluated means Cedar returned permit or deny.
- State 004: parsed means HL7 envelope and segment indexes were extracted without PHI log leakage.
- State 005: normalized means route-specific canonical fields were mapped.
- State 006: routed means destination workflow was selected.
- State 007: ack-pending means ACK content is ready but not emitted.
- State 008: ack-emitted means ACK was sent and audit-chain reference was written.
- State 009: nack-emitted means NACK was sent with safe reason code.
- State 010: quarantined means message cannot continue but is retained by pack policy.
- State 011: retry-scheduled means retry budget remains.
- State 012: replay-ready means idempotent replay can occur with same message_control_id.
- State 013: replayed means replay completed with new workflow_run_id and prior audit reference.
- State 014: expired means retention or retry window closed.
- State 015: superseded means a corrected message replaced this custody record.

## Cedar and policy gates
- Gate 001: deny when tenant_id is absent.
- Gate 002: deny when principal_id is absent.
- Gate 003: deny when audience_type is not allowed for HL7 ingress.
- Gate 004: deny when purpose does not match tenant route.
- Gate 005: deny when data_class is not hl7_message.
- Gate 006: deny when source_system_id is not bound to tenant.
- Gate 007: deny when route_id is not active.
- Gate 008: deny when route_version is expired outside replay.
- Gate 009: deny when processing_id conflicts with environment policy.
- Gate 010: deny when pack overlay forbids the jurisdiction.
- Gate 011: deny when credential sidecar cannot provide a short-lived source handle.
- Gate 012: deny when fragment soak requirements from ADR-0294 are not met.
- Gate 013: deny when transport downgrade lacks ADR-0253-amendment evidence.
- Gate 014: deny when audit-chain is unavailable for mutation.
- Gate 015: deny when ACK would expose PHI in clear text reason.
- Gate 016: permit only after route custody record can be written.
- Gate 017: permit replay only with original custody hash.
- Gate 018: permit quarantine release only with reviewer decision id.
- Gate 019: permit manual override only through break-glass IP-028 path.
- Gate 020: emit refusal evidence for every deny.

## ACK generation rules
- ACK rule 001: ACK includes message_control_id correlation without leaking parsed PHI.
- ACK rule 002: ACK code AA requires successful custody write and downstream handoff.
- ACK rule 003: ACK code AE requires safe application-error classification.
- ACK rule 004: ACK code AR requires safe rejection classification.
- ACK rule 005: NACK reason text must be from tenant-approved code set.
- ACK rule 006: ACK emission waits for audit-chain event id.
- ACK rule 007: ACK emission records route_id, route_version, source_system_id, and workflow_run_id.
- ACK rule 008: ACK emission records transform version.
- ACK rule 009: ACK emission records Cedar decision id.
- ACK rule 010: ACK emission records DealSet hold or release reference where commercial flow applies.
- ACK rule 011: ACK emission records home_cell and jurisdiction.
- ACK rule 012: ACK emission records transport protocol and downgrade state.
- ACK rule 013: ACK emission records retry budget.
- ACK rule 014: ACK emission records quarantine reference if NACK creates one.
- ACK rule 015: ACK emission records idempotency key.
- ACK rule 016: ACK body is not the evidence source of truth.
- ACK rule 017: ACK body is derived from the signed custody record.
- ACK rule 018: ACK replay uses original message hash and new replay reason.
- ACK rule 019: ACK replay cannot overwrite the original ACK.
- ACK rule 020: ACK replay links prior and new audit event ids.

## HL7 ACK Custody Benchmark Displacement
- Displacement claim: this IP measures competitors against ACK custody, so every comparison must prove message_control_id handling, ACK class, route version, custody hash, and audit-chain linkage.
- Non-generic rule: a vendor comparison that does not explain AA/AE/AR behavior, quarantine, replay, or route ownership is insufficient for this ACK custody plan.
- Redox displacement: Redox normalizes exchange connectivity; Oyatie requires per-message custody, Cedar decision id, and signed ACK evidence.
- Redox proof: local references include hl7-route capability, AsyncAPI channels, and local HL7 ACK SLO.
- Rhapsody displacement: Rhapsody route state is interface-engine centered; Oyatie route state is tenant-custody centered.
- Rhapsody proof: every route transition records route_version, workflow_run_id, and audit event id.
- InterSystems IRIS for Health displacement: IRIS can centralize clinical integration in one platform; Oyatie keeps flat service ownership and ADR-0105 layers.
- InterSystems proof: contracts and architecture files remain the route authority, not a suite console.
- Lyniate/Corepoint displacement: Corepoint channel views are displaced by signed custody packets and local runbooks.
- Lyniate/Corepoint proof: local-hl7-ack-latency-burn.md and hl7-queue-backlog.md own operator response.
- Mirth displacement: Mirth channel scripts are displaced by declarative route policy, contracts, and replay evidence.
- Mirth proof: Cedar gate list and contract references define behavior before implementation.
- NextGate displacement: NextGate identity decisions are not ACK authority; MPI ambiguity routes to IP-029 before ACK AA where required.
- NextGate proof: duplicate or uncertain patient matches produce AE/AR plus adjudication workflow references.
- Health Catalyst displacement: analytics ingestion quality is insufficient; ACK custody includes operational, policy, and provenance evidence.
- Health Catalyst proof: SLO, dashboard, trace, metric, log, and audit evidence are required.

## Failure modes
- Failure 001: malformed HL7 envelope emits safe NACK and quarantine reference.
- Failure 002: missing tenant_id emits Cedar deny and no parser persistence.
- Failure 003: unknown source_system_id emits Cedar deny and no ACK AA.
- Failure 004: route_version expired emits NACK unless replay exception is permitted.
- Failure 005: transform drift emits quarantine and workflow remediation.
- Failure 006: audit-chain unavailable pauses ACK AA and allows safe NACK only when evidence can be signed.
- Failure 007: downstream workflow unavailable schedules retry within budget.
- Failure 008: retry budget exhausted emits quarantine and operator runbook trigger.
- Failure 009: transport downgrade without disclosure emits reject evidence.
- Failure 010: credential sidecar unavailable emits temporary failure without exposing credential material.
- Failure 011: duplicate message_control_id returns idempotent prior custody state.
- Failure 012: conflicting duplicate message_control_id opens adjudication.
- Failure 013: MPI ambiguity routes to patient-match-review before final acceptance.
- Failure 014: pack residency conflict rejects cross-cell route.
- Failure 015: reviewer override without break-glass justification is denied.

## Capacity and performance
- Capacity 001: ACK p95 follows slos/local-hl7-ack-latency.openslo.yaml.
- Capacity 002: ACK p99 must account for Cedar evaluation, custody write, audit-chain write, and ACK emission.
- Capacity 003: route queues partition by tenant_id, home_cell, source_system_id, route_id, and data_class.
- Capacity 004: retry queues partition separately from first-attempt ingress.
- Capacity 005: quarantine queues partition by pack and reviewer queue.
- Capacity 006: metric labels must not include raw patient identifiers.
- Capacity 007: message_control_id cardinality belongs in audit evidence, not metrics.
- Capacity 008: route cache invalidation follows route_version and ADR-0258 deprecation.
- Capacity 009: high-volume lab feeds require backpressure before cross-tenant starvation.
- Capacity 010: Little's Law calculation uses arrival rate, route service time, and queue depth per tenant.
- Capacity 011: ACK latency dashboard separates network, policy, parse, custody, workflow, and audit spans.
- Capacity 012: replay traffic has a separate budget from live clinical traffic.
- Capacity 013: source-system bursts cannot bypass Cedar or audit writes.
- Capacity 014: degraded cell behavior must stay inside residency constraints.
- Capacity 015: break-glass traffic stays challenge-free only when clean emergency rules apply.

## Observability
- Event `oya.healthcare.integration.hl7.route.received` records receipt.
- Event `oya.healthcare.integration.hl7.route.policy_denied` records Cedar denial.
- Event `oya.healthcare.integration.hl7.route.ack_emitted` records ACK AA, AE, or AR.
- Event `oya.healthcare.integration.hl7.route.quarantined` records quarantine.
- Event `oya.healthcare.integration.hl7.route.replayed` records replay.
- Metric `healthcare_integration_hl7_ack_latency_ms` dimensions: tenant_hash, cell, source_type, route_id, ack_code.
- Metric `healthcare_integration_hl7_quarantine_total` dimensions: reason_code, cell, pack, source_type.
- Metric `healthcare_integration_hl7_retry_budget_remaining` dimensions: route_id, cell, source_type.
- Trace span `healthcare.hl7.route.evaluate` wraps policy, parse, transform, custody, and ACK emission.
- Log schema includes event_id, route_id, route_version, source_system_id, decision_id, audit_event_id, and workflow_run_id.
- Dashboard reference: dashboards/local-domain-throughput.json.
- Dashboard reference: dashboards/local-audit-completeness.json.
- Runbook reference: runbooks/local-hl7-ack-latency-burn.md.
- Runbook reference: runbooks/hl7-queue-backlog.md.
- SLO reference: slos/local-hl7-ack-latency.openslo.yaml.

## Implementation steps
- Step 001: Add route custody aggregate in the domain layer.
- Step 002: Add route custody value objects in the kernel layer.
- Step 003: Add route custody command handler in the usecase layer.
- Step 004: Add route custody DTO validation in the api/rest layers.
- Step 005: Add source-system adapter ports without embedding policy logic.
- Step 006: Add worker processing for retry, quarantine, and replay.
- Step 007: Add Cedar fragment references through library-first policy evaluation.
- Step 008: Add audit-chain emission for every custody state.
- Step 009: Add dashboard rows for ACK latency and quarantine.
- Step 010: Add runbook links for ACK burn and backlog.
- Step 011: Add contract examples for ACK AA, AE, AR, quarantine, and replay.
- Step 012: Add property tests for idempotent duplicate message_control_id behavior.
- Step 013: Add replay tests for original custody hash preservation.
- Step 014: Add policy tests for deny conditions.
- Step 015: Add benchmark displacement evidence to review packet.

## Tests and evidence
- Test 001: line count for this IP is at least 200.
- Test 002: ADR scan finds ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0253-amendment, ADR-0258, ADR-0263, ADR-0314, and ADR-0321.
- Test 003: benchmark scan finds Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, and Health Catalyst.
- Test 004: local reference scan finds hl7-route.yaml.
- Test 005: local reference scan finds openapi-v1.yaml, asyncapi-v1.yaml, and healthcare-integration-v1.proto.
- Test 006: local reference scan finds local-hl7-ingest-source-scope.cedar.
- Test 007: local reference scan finds local-hl7-ack-latency.openslo.yaml.
- Test 008: local reference scan finds both HL7 runbooks.
- Test 009: review confirms ADR-0321 was not edited.
- Test 010: review confirms no oya vcs verify, done, or promote was run.

## Rollback
- Rollback 001: disable new route_version for affected tenant only.
- Rollback 002: keep previous route_version active until ACK custody tests pass.
- Rollback 003: replay quarantined messages idempotently after fix.
- Rollback 004: do not delete original custody records.
- Rollback 005: do not rewrite prior ACK evidence.
- Rollback 006: mark failed ACK attempts superseded with audit reference.
- Rollback 007: restore prior Cedar fragment only after soak window rules permit.
- Rollback 008: keep DealSet holds until ACK evidence is accepted.
- Rollback 009: notify tenant admin through existing workflow path.
- Rollback 010: attach reviewer decision id to every rollback.

## Acceptance criteria
- AC01: Every HL7 route decision carries tenant_id, principal_id, audience_type, purpose, data_class, source_system_id, route_id, and route_version.
- AC02: Every ACK, NACK, retry, quarantine, and replay state emits ADR-0263 audit evidence.
- AC03: Cedar denies happen before parser persistence or downstream workflow handoff.
- AC04: The ACK body is never the evidence source of truth.
- AC05: Duplicate message_control_id handling is idempotent.
- AC06: Conflicting duplicates route to adjudication.
- AC07: Transport downgrade is visible and governed by ADR-0253-amendment.
- AC08: DealSet settlement holds are tied to ACK evidence.
- AC09: All seven named benchmarks are explicitly displaced.
- AC10: This plan remains scoped to the assigned IP file.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md:10` - Repo-local references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/capabili...; `microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md:196` - - Test 005: local reference scan finds openapi-v1.yaml, asyncapi-v1.yaml, and healthcare-integration-v1.proto..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `86400s` RTO p99 and `3600s` RPO p99.
- Applicable compliance pack floor: `PCI-DSS-L1-v4` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=86400`, `rpo_p99_seconds=3600`, `multi_region_required=false`, `drill_cadence_required=annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md:26` - - Route field 004: purpose must name treatment, operations, payment, public-health, or approved pack-specific purpose.; `microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md:48` - - State 004: parsed means HL7 envelope and segment indexes were extracted without PHI log leakage..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md:89` - - ACK rule 006: ACK emission waits for audit-chain event id.; `microservices/healthcare-integration/IP-026-hl7-ack-route-custody.md:90` - - ACK rule 007: ACK emission records route_id, route_version, source_system_id, and workflow_run_id..
