---
id: ADR-MS-001
title: Omnichannel routing, queue, and recording-consent contract for contact-center
status: Proposed
date: 2026-05-20
microservice: contact-center
related_oyatie_adrs:
  - ADR-0003-audit-chain-and-evidence-emission
  - ADR-0007-cedar-authorization-policy-and-persona-tier
  - ADR-0008-data-use-boundary
  - ADR-0009-cell-architecture-per-tenant-per-region
  - ADR-0037-public-api-stability-tiers-and-deprecation
  - ADR-0043-secrets-management-openbao-and-hsm-per-cell
  - ADR-0090-hyper-canonical-http-backbone
  - ADR-0131-per-microservice-flat-layout
decision_owner: axis-contact-center + council-product
---

# ADR-MS-001: Omnichannel routing, queue, and recording-consent contract for contact-center

## Context

- Pressure name: regulated conversation routing pressure.
- `contact-center` owns voice routing, queue, agent desktop, recording consent, quality monitoring, workforce adherence, agent assist, callback, and SLA rescheduling.
- The service PRD requires every operation to carry tenant scope, principal, purpose, data class, pack overlay, idempotency key, trace context, and audit-chain target.
- The service contract exposes `GET /contact-center/capabilities`.
- The service contract exposes `POST /contact-center/actions/{action_id}`.
- The service AsyncAPI publishes `ActionAccepted`.
- Local policy files include `voice-routing-authorization.cedar` and `emergency-services-bypass.cedar`.
- Local policy files also include `abuse-defence.cedar`, `auditor-scope.cedar`, `ci-scope.cedar`, and `data-residency.md`.
- Local SLOs include availability, read latency, write latency, replay freshness, policy decision latency, audit emission lag, and local call-specific objectives.
- Local call-specific SLOs include route decision latency, transfer success, call drop rate, callback schedule latency, recording consent correctness, and agent presence freshness.
- Local dashboards include operating bar overview, local policy decisions, local audit completeness, local SLO burn, local domain throughput, abuse outcomes, and tenant cost and capacity.
- Constraint name: voice cannot be treated like ordinary CRUD.
- A route decision may affect emergency services, call recording legality, queue fairness, and customer harm.
- Queue selection needs agent presence, skill match, tenant pack, call purpose, SLA tier, and emergency bypass state.
- Recording consent must be evaluated before recording begins and whenever participants, jurisdiction, or purpose changes.
- Constraint name: local evidence before vendor parity.
- The service aims at vendor parity with contact-center systems, but the durable decision is tenant-scoped evidence and control.
- External telephony providers may carry media, but Oyatie owns routing decision, consent state, queue state, and audit evidence.
- Constraint name: emergency services bypass is narrow.
- Emergency bypass must not become a general support-operator bypass.
- Emergency bypass decisions require policy evidence, route trace, actor identity, and post-incident review.
- Constraint name: presence freshness.
- Stale agent presence can route calls to unavailable agents or break transfer success.
- The local SLO target for agent presence freshness is 0.999.
- Constraint name: consent correctness.
- Call recording consent correctness has a local target of 0.999.
- A consent failure is a compliance incident, not merely a UX defect.
- Constraint name: audit lag.
- Audit emission lag target is 0.999 and must cover route, transfer, consent, callback, and emergency actions.

## Decision

- Decision name: policy-gated conversation action contract.
- `contact-center` will route every external mutation through `POST /contact-center/actions/{action_id}`.
- The `action_id` values include `voice-routing.create`, `voice-routing.amend`, `voice-routing.approve`, `voice-routing.import`, `voice-routing.export`, and `voice-routing.replay`.
- The `action_id` values include `queue.create`, `queue.amend`, `queue.approve`, `queue.import`, `queue.export`, and `queue.replay`.
- The `action_id` values include `agent-desktop.create`, `agent-desktop.amend`, `recording-consent.create`, `recording-consent.amend`, and `callback-schedule.create`.
- Every action request must include tenant id, principal id, actor role, purpose, data class, pack overlay, idempotency key, trace context, and audit target.
- Every voice routing decision must include channel, entry point, queue id, skill tags, agent presence version, SLA tier, and emergency bypass flag.
- Every queue decision must include queue id, priority band, wait estimate, fairness bucket, tenant capacity budget, and callback eligibility.
- Every recording consent decision must include participant ids, jurisdiction codes, consent basis, capture mode, redaction policy, and consent version.
- Every transfer decision must include source agent, destination agent or queue, reason code, active consent state, and call segment id.
- The service will run Cedar before state mutation and before telephony provider dispatch.
- The service will emit `ActionAccepted` only after policy, idempotency, presence freshness, and audit target validation.
- The service will emit denial evidence for invalid consent, stale presence, unauthorized route, or residency mismatch.
- Emergency services bypass may override queue fairness but may not override tenant boundary, data residency, audit emission, or consent recording legality.
- Emergency services bypass requires `emergency_services_reason`, `operator_principal_id`, `route_trace_id`, and `review_due_at`.
- Agent assist may observe conversation context only after data class and purpose are approved.
- Agent assist cannot change routing or recording state without a separate action request.
- Workforce adherence streams may influence route selection only through current presence and capacity facts.
- Provider media adapters must use cloud-secrets references for SIP trunks, webhook secrets, and telephony credentials.
- The service will keep contact-center availability target at 0.999.
- The service will keep write latency target at 0.999 for defined good events.
- The service will keep route decision latency target at 0.99.
- The service will keep callback schedule latency target at 0.99.
- The service will keep transfer success target at 0.99.
- The service will keep local call drop rate good-event target at 0.995.
- The service will keep recording consent correctness target at 0.999.
- The service will keep agent presence freshness target at 0.999.
- The service will keep audit emission lag target at 0.999.
- The service will preserve replay freshness target at 0.999 for action replay.
- Metrics may include tenant hash, cell tier, action family, channel, and outcome, but not raw phone numbers or conversation content.

## Alternatives Considered

### Alternative 1: Buy a hosted contact-center suite and expose it through connectors

- Pros: mature routing, telephony, workforce, and quality features.
- Pros: lower initial telephony integration burden.
- Cons: consent, queue, and emergency evidence live outside Oyatie control.
- Cons: pack overlays and tenant-scoped audit become vendor-specific.
- Cons: cross-product workflow and Ontology handoff is weaker.
- Rejected because the durable control contract must remain internal even when media providers are external.

### Alternative 2: Treat contact-center as a thin UI over telephony provider APIs

- Pros: fastest visible feature path.
- Pros: provider handles most call-state complexity.
- Cons: routing policy and evidence become scattered across UI callbacks.
- Cons: emergency bypass and consent correctness cannot be centrally verified.
- Cons: queue fairness cannot integrate tenant capacity, SLA, and pack policy.
- Rejected because route and consent must be domain decisions, not UI affordances.

### Alternative 3: Route by static queues only

- Pros: simple operational model.
- Pros: easy to explain to administrators.
- Cons: ignores agent presence freshness and skill availability.
- Cons: cannot handle emergency, SLA, or callback-specific routing.
- Cons: creates poor transfer and call drop outcomes.
- Rejected because dynamic presence and policy are required for correctness.

### Alternative 4: Record all calls and redact later

- Pros: simple capture pipeline.
- Pros: more material available for quality monitoring.
- Cons: unlawful in jurisdictions that require prior consent.
- Cons: creates avoidable sensitive data retention.
- Cons: redaction cannot undo prohibited capture.
- Rejected because consent correctness must precede recording.

### Alternative 5: Let support operators manually override policy

- Pros: practical during incidents.
- Pros: shortens escalations for individual calls.
- Cons: manual override can bypass tenant, pack, and audit boundaries.
- Cons: reviewers cannot reconstruct why a route happened.
- Cons: abuse risk is high for recorded conversations.
- Rejected because emergency bypass must be narrow and evidence-bound.

## Consequences

### Positive

- Route, queue, consent, transfer, callback, and quality events share one action envelope.
- Emergency bypass is explicit, reviewable, and bounded.
- Recording consent becomes a first-class policy decision.
- Agent presence freshness directly protects route correctness.
- Telephony provider changes do not rewrite domain authorization.
- Dashboards can separate route latency, transfer success, call drops, and consent correctness.
- Replay can reconstruct failed actions with the same tenant and policy envelope.
- Auditors can trace contact-center actions without reading conversation content.

### Negative

- The service must keep strong real-time state for agent presence and queue capacity.
- Consent and emergency policy require region-specific maintenance.
- Provider integrations still carry media-path failure modes.
- Route decisions become latency-sensitive domain decisions.
- Operators need clear runbooks for emergency bypass misuse or consent failures.
- Replay of conversation actions can be sensitive and must avoid duplicate provider dispatch.
- Call-state testing requires simulators for voice, chat, and callback channels.

### Neutral

- External telephony providers may remain media carriers.
- Agent-assist intelligence remains a downstream consumer, not routing authority.
- Quality monitoring may process recordings only after consent and retention policy allow it.
- Workforce adherence may provide facts but not override policy.
- Local dashboards are evidence surfaces and not separate policy authorities.

### Follow-up work

- Add synthetic call simulator for route, transfer, callback, and drop scenarios.
- Add jurisdiction consent matrix for recording-consent actions.
- Add emergency bypass post-incident review workflow.
- Add queue fairness property tests for hot-tenant and high-priority events.
- Add provider failover playbooks for SIP and WebRTC media carriers.
- Add replay guard that prevents duplicate external call dispatch.

## Implementation Notes

### Data Shapes

- `ContactCenterActionRequest` fields: `tenant_id`, `principal_id`, `actor_role`, `action_id`, `purpose`, `data_class`, `pack_overlay`, `idempotency_key`, `traceparent`, `audit_target`.
- `VoiceRouteDecision` fields: `route_trace_id`, `channel`, `entry_point`, `queue_id`, `skill_tags`, `agent_presence_version`, `sla_tier`, `emergency_services_flag`, `decision`.
- `QueueState` fields: `queue_id`, `tenant_id_hash`, `priority_band`, `fairness_bucket`, `estimated_wait_seconds`, `callback_eligible`, `capacity_budget_id`.
- `AgentPresence` fields: `agent_id`, `tenant_id_hash`, `state`, `skill_tags`, `last_seen_at`, `presence_version`, `active_interactions`.
- `RecordingConsent` fields: `call_segment_id`, `participant_ids_hash`, `jurisdiction_codes`, `basis`, `capture_mode`, `redaction_policy`, `consent_version`, `valid_until`.
- `TransferDecision` fields: `call_segment_id`, `source_agent_id`, `destination_type`, `destination_id`, `reason_code`, `active_consent_version`, `decision`.
- `CallbackSchedule` fields: `callback_id`, `tenant_id_hash`, `customer_contact_ref`, `queue_id`, `promised_at`, `sla_tier`, `consent_version`.
- `EmergencyBypass` fields: `route_trace_id`, `reason`, `operator_principal_id`, `policy_version`, `review_due_at`, `evidence_id`.
- `ActionAccepted` event fields: `tenant_id_hash`, `action_id`, `resource_id`, `decision`, `policy_version`, `traceparent`, `evidence_id`.

### API Endpoints

- `GET /contact-center/capabilities` lists action families, channels, pack eligibility, and required data classes.
- `POST /contact-center/actions/{action_id}` invokes a policy-gated domain action.
- `POST /contact-center/actions/voice-routing.create` creates an entry-point route.
- `POST /contact-center/actions/queue.amend` changes queue priority or fairness settings.
- `POST /contact-center/actions/recording-consent.create` records lawful consent state before capture.
- `POST /contact-center/actions/callback-schedule.create` schedules callback under SLA policy.
- `POST /contact-center/actions/voice-routing.replay` replays a failed route without duplicate provider dispatch.
- `POST /contact-center/actions/recording-consent.replay` replays consent evidence without recapturing audio.

### Cedar Policies

- `policy/voice-routing-authorization.cedar` authorizes route, queue, transfer, and callback actions.
- `policy/emergency-services-bypass.cedar` permits only narrow emergency route override.
- `policy/abuse-defence.cedar` blocks abusive ingress and suspicious operator behavior.
- `policy/auditor-scope.cedar` grants evidence review without conversation-content access.
- `policy/ci-scope.cedar` permits contract and policy tests without production mutation.
- `policy/data-residency.md` binds recordings, transcripts, and call metadata to pack residency.
- Policy must deny route actions when `agent_presence_version` is stale.
- Policy must deny recording when consent basis is missing or invalid for jurisdiction.
- Policy must deny transfer if destination queue lacks tenant or pack eligibility.

### SLO Targets

- `availability.openslo.yaml`: contact-center availability target 0.999.
- `write-latency.openslo.yaml`: write-latency target 0.999.
- `read-latency.openslo.yaml`: read-latency target 0.999.
- `replay-freshness.openslo.yaml`: replay freshness target 0.999.
- `policy-decision-latency.openslo.yaml`: policy decision latency target 0.999.
- `audit-emission-lag.openslo.yaml`: audit emission lag target 0.999.
- `local-route-decision-latency.openslo.yaml`: route decision latency target 0.99.
- `local-transfer-success.openslo.yaml`: transfer success target 0.99.
- `local-call-drop-rate.openslo.yaml`: call drop good-event target 0.995.
- `local-callback-schedule-latency.openslo.yaml`: callback schedule latency target 0.99.
- `local-recording-consent-correctness.openslo.yaml`: recording consent correctness target 0.999.
- `local-agent-presence-freshness.openslo.yaml`: agent presence freshness target 0.999.

## Verification

- Unit test `action_request_requires_tenant_principal_purpose_data_class_and_audit_target`.
- Unit test `voice_route_decision_requires_presence_version`.
- Unit test `recording_consent_requires_jurisdiction_and_basis`.
- Unit test `emergency_bypass_requires_review_due_at`.
- Unit test `callback_schedule_requires_sla_tier`.
- Property test `queue_fairness_never_starves_low_priority_tenant`.
- Property test `route_idempotency_prevents_duplicate_provider_dispatch`.
- Cedar test `voice_routing_denies_cross_tenant_queue`.
- Cedar test `voice_routing_denies_stale_agent_presence`.
- Cedar test `recording_consent_denies_missing_basis`.
- Cedar test `emergency_bypass_denies_non_emergency_reason`.
- Cedar test `auditor_scope_cannot_read_conversation_content`.
- Contract test `openapi-v1.yaml_contains_capabilities_and_actions`.
- Contract test `asyncapi-v1.yaml_publishes_action_accepted`.
- Integration test `route_action_emits_action_accepted_after_policy`.
- Integration test `recording_start_fails_without_valid_consent`.
- Integration test `transfer_action_preserves_active_consent_version`.
- Integration test `callback_schedule_emits_sla_evidence`.
- Integration test `emergency_bypass_emits_review_due_evidence`.
- Load test `route_decision_latency_meets_099_target`.
- Load test `agent_presence_freshness_meets_0999_target`.
- Load test `callback_schedule_latency_meets_099_target`.
- Load test `audit_emission_lag_meets_0999_target`.
- Chaos test `provider_media_outage_does_not_drop_route_evidence`.
- Chaos test `presence_stream_lag_denies_new_route_assignment`.
- Chaos test `audit_backpressure_blocks_recording_mutation`.
- Replay test `voice_routing_replay_does_not_duplicate_call`.
- Replay test `recording_consent_replay_does_not_capture_audio`.
- Metric `oya_contact_center_route_decision_latency_good_total`.
- Metric `oya_contact_center_agent_presence_freshness_good_total`.
- Metric `oya_contact_center_recording_consent_correctness_good_total`.
- Metric `oya_contact_center_transfer_success_good_total`.
- Metric `oya_contact_center_call_drop_rate_good_total`.
- Metric `oya_contact_center_audit_emission_lag_good_total`.
- Dashboard `dashboards/operating-bar-overview.json`.
- Dashboard `dashboards/local-policy-decisions.json`.
- Dashboard `dashboards/local-audit-completeness.json`.
- Dashboard `dashboards/local-slo-burn.json`.
- Dashboard `dashboards/local-domain-throughput.json`.
- Dashboard `dashboards/tenant-cost-and-capacity.json`.
- Runbook check `runbooks/emergency-services-bypass-review.md` covers misuse review.
- Runbook check `runbooks/recording-consent-failure.md` covers compliance incident path.
- Promotion gate blocks if consent correctness is below 0.999.
- Promotion gate blocks if emergency bypass lacks review evidence.

## References

- Oyatie ADR-0003: Audit chain and evidence emission.
- Oyatie ADR-0007: Cedar authorization policy and persona tier.
- Oyatie ADR-0008: Data use boundary.
- Oyatie ADR-0009: Cell architecture per tenant per region.
- Oyatie ADR-0037: Public API stability tiers and deprecation.
- Oyatie ADR-0043: Secrets management OpenBao and HSM per cell.
- Oyatie ADR-0090: Hyper canonical HTTP backbone.
- Oyatie ADR-0131: Per-microservice flat layout.
- RFC 3261: SIP Session Initiation Protocol.
- RFC 3550: RTP A Transport Protocol for Real-Time Applications.
- RFC 5764: DTLS-SRTP.
- RFC 8825: WebRTC Overview.
- RFC 8866: SDP Session Description Protocol.
- RFC 8224: Authenticated Identity Management in STIR.
- W3C WebRTC and Media Capture specifications.
- Twilio Programmable Voice documentation.
- Amazon administrator and contact flow documentation.
- Google SRE Workbook: SLOs, error budgets, and incident response.
- Cedar policy language documentation.
- NIST SP 800-63B: Authentication lifecycle for operator access.
