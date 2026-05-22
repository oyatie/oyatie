---
doc_class: Implementation-Plan-Journey-Slice
journey_id: j147
microservice: workflow-engine
status: draft
date: 2026-05-20
authority_tier: 3
intern_buildable: true
adr_anchors: [ADR-0244, ADR-0249, ADR-0292, ADR-0299, ADR-0307, ADR-0311]
---

# workflow-engine — IP slice for j147 (cohort governance + cross-tenant referral routing)

## Scope

1. **Cross-tenant referral application routing** — when a cohort member clicks a structured-referral link, route the application to the hiring tenant with all attestations + referral-of-record attached.
2. **Cohort governance-vote workflow** — open, voting period, closure with quorum + supermajority rules.
3. **Long-lived ownership transition workflow** — at T+12mo, transition the alumni sub-tenant to a community-co-op tenant.
4. **Referral-bonus settlement workflow** — facilitate the cross-tenant payable + the voluntary "split with the referred candidate" event.

## API surface

```proto
service CohortReferralRouting {
  rpc Submit(SubmitRequest) returns (SubmitResponse);  // routes click → application
}

service CohortGovernance {
  rpc OpenVoteWorkflow(OpenRequest) returns (OpenResponse);
  rpc CloseVoteWorkflow(CloseRequest) returns (CloseResponse);
  rpc TransitionOwnership(TransitionRequest) returns (TransitionResponse);
}

service ReferralBonus {
  rpc Settle(SettleRequest) returns (SettleResponse);
}
```

## Implementation tasks

### T1 — Referral routing workflow

```
template: cohort_referral_routing_v1
steps:
  - validate_attestation
  - pre_fill_applicant_context (from cohort member's profile)
  - cross_tenant_application_submit (to hiring tenant's workflow-engine)
  - on_hire_event_emitted (eventual; per j145)
  - referral_bonus_settle (if hire confirmed and bonus applicable)
```

### T2 — Governance workflow

```
template: cohort_governance_vote_v1
steps:
  - open_vote (records question, quorum_threshold, supermajority_threshold)
  - collect_votes (over N days; default 7)
  - check_quorum
  - tally
  - close_with_outcome
  - emit_downstream_action (e.g., trigger transition_ownership)
```

### T3 — Ownership transition workflow

```
template: cohort_ownership_transition_v1
steps:
  - validate_council (5 elected; passkey signatures)
  - clone_sub_tenant_state
  - transfer_billing
  - rebind_cedar_policies (new owner = council)
  - emit_CohortGovernanceTransition
```

### T4 — Referral bonus settle

Cross-tenant payable from hiring-tenant to referrer's personal-tenant Payments. Voluntary-split between referrer and referred candidate is a second cross-tenant payable.

## Cedar permits

| Permit | Granted to | Purpose |
|---|---|---|
| `b2alumni.workflow.referral_routing.submit` | cohort member | Trigger referral |
| `b2alumni.workflow.governance_vote.open` | moderator | Open vote |
| `b2alumni.workflow.governance_vote.close` | moderator | Close vote |
| `b2alumni.workflow.ownership_transition.execute` | council (5 signatures) | Transition |
| `b2b.workflow.referral_bonus.settle` | hiring-tenant workflow-engine | Settle |

## Audit emissions

- `CrossTenantReferralRoutingStarted`, `ApplicationSubmittedWithReferral`
- `GovernanceVoteWorkflowOpened`, `Closed`
- `CohortGovernanceTransition`
- `ReferralBonusPayableSettled`, `RewardSplit`

## Performance

- Referral routing end-to-end ≤ 5s.
- Governance vote close + transition ≤ 30s.

## Acceptance criteria

- [ ] B.5, B.6, B.8 pass.
- [ ] Council signatures required for ownership transition (5 distinct passkey-signed votes).
- [ ] Referral-bonus voluntary-split flow works.

## Out of scope

- The Community surface itself (community IP).
- The detection-substrate evaluator (separate IP).
- The Payments primitive (covered in j142, j145, j146 payments IPs).

## Wave 15 row-loop remediation

The generated completion-expansion task loop was deleted as un-grounded speculation. The implementation plan above remains the authoritative slice because it names concrete workflow state, contracts, Cedar policy, latency/evidence expectations, and service boundaries. Future additions must cite a real workflow-engine contract artifact or a planned IP before adding rows.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j147-cohort-governance-and-cross-tenant-referrals.md` matched `payment`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/workflow-engine/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/workflow-engine/slos/payload-bytes-budget-correctness.openslo.yaml`, `microservices/workflow-engine/slos/replay-determinism-correctness.openslo.yaml`, `microservices/workflow-engine/slos/worker-poll-availability.openslo.yaml`, `microservices/workflow-engine/slos/workflow-completion-availability.openslo.yaml`, `microservices/workflow-engine/policy/auditor-scope.cedar`.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/workflow-engine/IP-journey-j147-cohort-governance-and-cross-tenant-referrals.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/workflow-engine/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: not deferrable for runtime placement; carbon fields still emit, but ADR-0344 D-9 compliance-pack and realtime exclusions block carbon-aware delay.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
