---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j145
microservice: workflow-engine
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0145, ADR-0244, ADR-0299, ADR-0311]
---

# workflow-engine — IP slice for j145 (hiring-decision template)

## Scope

Deliver the `hiring_decision_full_loop_v3` template for the employer side. 22 steps from application-received to start-day.

## Template skeleton

```yaml
template_id: hiring_decision_full_loop_v3
version: 3.0.0
required_inputs:
  - application_id
  - role_requisition_id
  - jurisdiction
  - jurisdictional_pack_overlay
steps:
  - id: ai_screen
    µservice: intelligence
    rpc: Screen.Apply
  - id: fairness_audit
    µservice: compliance
    rpc: Audit.Fairness
  - id: hr_review_advance
    µservice: workflow-engine (HR-admin task)
  - id: phone_screen_invite
    µservice: mail
    rpc: OutboundMail.Send (cross-tenant interview_invite)
  - id: phone_screen_meet
    µservice: meet
    blocks_on_event: InterviewRoomEnded{round=phone_screen}
  - id: phone_screen_advance
    µservice: workflow-engine (HR-admin task)
  - id: onsite_loop_schedule
    µservice: calendar + mail
  - id: onsite_loop_rounds (×4 + wrap)
    blocks_on_event: InterviewRoundCompleted × 5
  - id: collect_feedback
    µservice: workflow-engine (HR-admin task)
  - id: hiring_decision
    µservice: workflow-engine (HR-admin + hiring-manager task)
  - id: generate_offer_letter
    µservice: workflow-engine
  - id: deliver_offer
    µservice: community (cross-tenant)
  - id: wait_for_acceptance_or_counter
    timeout: 30d
  - id: handle_counter (optional)
  - id: receive_acceptance
    µservice: community (cross-tenant)
  - id: request_cross_tenant_principal_provisioning
    µservice: identity (cross-tenant)
  - id: wait_for_provisioning_approval
  - id: initiate_background_check
    µservice: connect
  - id: verify_references
    µservice: identity (public attestation verify)
  - id: link_pay_account
    µservice: payments (cross-tenant)
  - id: start_day_login
    µservice: identity (event listener)
  - id: workflow_close
```

## Cedar permits

| Permit | Granted to | Purpose |
|---|---|---|
| `b2b.workflow.hiring_decision.start` | HR-admin | Open workflow |
| `b2b.workflow.hiring_decision.advance` | HR-admin | Round transitions |
| `b2b.workflow.hiring_decision.generate_offer` | HR-admin + hiring-manager | Generate offer letter |

## Acceptance criteria

- [ ] Template loads + validates.
- [ ] All cross-tenant steps emit cross-tenant envelopes correctly.
- [ ] Workflow completes within 30 days SLA.
- [ ] Background-check and provisioning steps wait gracefully.

## Out of scope

- The actual hiring criteria (HR pack).
- The Intelligence screening model (Intelligence IP).
- adapter to background-check vendor (IP).

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j145-hiring-decision-template.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.
