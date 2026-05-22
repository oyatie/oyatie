---
doc_class: CrossMicroserviceIntegrationTest
scenario_id: CMIT-006-ai-agent-permit-elevation
microservices_under_test:
  - intelligence
  - governance
  - identity
  - audit-chain
status: draft-canonical
date: 2026-05-20
owner: codex-cross-msvc-integration-tests-w1
related_oyatie_adrs:
  - ADR-0113-vcs-orchestrator-end-to-end
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0242-oyatie-is-a-tenant-doctrine
  - ADR-0243-cedar-as-universal-gate
  - ADR-0247-self-hosting-self-modification-doctrine
  - ADR-0255-intelligence-as-two-layer-ai-substrate
  - ADR-0263-observability-emission-contract
  - ADR-0293-meta-trust-root
---

# AI Agent Permit Elevation

## Scenario Description

Nora Singh, platform engineer in the `oyatie` tenant, supervises `oyatie.foundry.fragment-author`, an AI agent that asks to elevate from draft-only review authority into temporary Cedar-fragment authoring authority; ADR-0247 allows self-modification only when identity, governance, intelligence, and audit-chain prove the elevation is bounded, witnessed, reversible, and sealed.

## Pre-conditions

### Named tenant fixtures

- Platform tenant: `tenant-oyatie`.
- Foundry sub-scope: `oyatie.foundry`.
- Human supervisor: `principal-nora-singh-platform`.
- Agent principal before elevation: `oyatie.foundry.reviewer-agent.facet-policy`.
- Agent principal after approved elevation: `oyatie.foundry.fragment-author`.
- Elevation request id: `elev-oyatie-foundry-fragment-author-001`.
- Candidate Cedar fragment id: `fragment-cross-msvc-policy-fence-001`.
- Candidate fragment version: `v2026-05-20.cmit-006`.
- Meta-trust attestor: `oyatie.foundry.meta-trust-root-attestor`.
- Offline root witness quorum id: `witness-quorum-root-5of9-cmit-006`.
- Intelligence run id: `run-cmit-006-elevation-rationale`.
- Audit stream: `tenant-oyatie.foundry.self-modification`.
- Trace id: `trace-cmit-006-permit-elevation`.
- Idempotency key: `idem-cmit-006-elevation`.

### Named Cedar permits

- `permit-foundry-agent-draft-fragment`.
- `permit-foundry-agent-request-elevation`.
- `permit-human-supervisor-review-elevation`.
- `permit-meta-trust-attestor-witness-elevation`.
- `permit-governance-activate-temporary-agent-permit`.
- `permit-intelligence-explain-elevation-rationale`.
- `permit-audit-seal-self-modification`.
- `forbid-agent-self-elevate-without-human`.
- `forbid-agent-root-key-access`.
- `forbid-elevation-without-expiry`.

### Named pack activations

- `pack-ADR-0247-self-modification`.
- `pack-ADR-0293-meta-trust-root`.
- `pack-SOC2-Type-II-baseline`.
- `pack-SLSA-provenance-baseline`.
- `pack-agentic-development-transparency`.
- `pack-foundry-dev-tools-cell`.

### Starting state checks

- Agent has draft-only authority.
- Agent cannot publish or activate Cedar fragments.
- Nora has supervisor authority for foundry self-modification.
- Meta-trust attestor can request witness signatures but holds no root key material.
- Candidate fragment exists only as a draft.
- Active policy bundle does not include candidate fragment.
- Audit stream for self-modification exists.
- No active elevation exists for the agent principal.

## Test Steps

1. Resolve the current agent identity.
   - API call: `GET /identity/v1/principals/oyatie.foundry.reviewer-agent.facet-policy`.
   - Expected response: `200 OK` with `authority_level="DRAFT_ONLY"`.
   - Assertion: principal belongs to `tenant-oyatie` and sub-scope `oyatie.foundry`.

2. Attempt forbidden self-elevation directly.
   - API call: `POST /governance/v1/agent-permits:elevate` as the agent without supervisor proof.
   - Expected response: `403 Forbidden` with `error.code="SELF_ELEVATION_FORBIDDEN"`.
   - Assertion: denial cites `forbid-agent-self-elevate-without-human`.

3. Request intelligence rationale for elevation.
   - API call: `POST /intelligence/v1/agent-runs` with task `explain_fragment_authoring_need`.
   - Expected response: `202 Accepted` with `agent_run_id="run-cmit-006-elevation-rationale"`.
   - Assertion: intelligence run is metadata-logged and cannot mutate policy.

4. Fetch rationale result.
   - API call: `GET /intelligence/v1/agent-runs/run-cmit-006-elevation-rationale`.
   - Expected response: `200 OK` with `state="COMPLETED"` and `recommended_duration_seconds=900`.
   - Assertion: rationale names draft fragment id and requested authority only.

5. Create bounded elevation request.
   - API call: `POST /governance/v1/agent-permit-elevation-requests`.
   - Expected response: `201 Created` with `elevation_request_id="elev-oyatie-foundry-fragment-author-001"` and `state="AWAITING_HUMAN_REVIEW"`.
   - Assertion: request includes expiry, scope, fragment id, and rationale hash.

6. Seal the elevation request.
   - API call: `POST /audit-chain/v1/streams/tenant-oyatie.foundry.self-modification/events`.
   - Expected response: `201 Created` with `event_class="FoundryAgentPermitElevationRequested"`.
   - Assertion: event includes requester, target authority, expiry, and rationale hash.

7. Nora reviews the request.
   - API call: `GET /governance/v1/agent-permit-elevation-requests/elev-oyatie-foundry-fragment-author-001`.
   - Expected response: `200 OK` with complete requested scope.
   - Assertion: UI/API payload warns that this is self-modification authority.

8. Nora approves human-supervisor step.
   - API call: `POST /governance/v1/agent-permit-elevation-requests/elev-oyatie-foundry-fragment-author-001:approve-human`.
   - Expected response: `200 OK` with `state="AWAITING_META_TRUST_WITNESS"`.
   - Assertion: approval actor is `principal-nora-singh-platform`, not the agent.

9. Attestor requests witness quorum.
   - API call: `POST /governance/v1/meta-trust/witness-requests`.
   - Expected response: `202 Accepted` with `witness_request_id="witness-quorum-root-5of9-cmit-006"`.
   - Assertion: attestor cannot access root key material.

10. Submit witness quorum proof.
    - API call: `POST /governance/v1/meta-trust/witness-requests/witness-quorum-root-5of9-cmit-006:complete`.
    - Expected response: `200 OK` with `witness_result="QUORUM_VALID"`.
    - Assertion: proof has 5-of-9 signatures across at least 3 jurisdictions.

11. Activate temporary permit.
    - API call: `POST /governance/v1/agent-permit-elevation-requests/elev-oyatie-foundry-fragment-author-001:activate`.
    - Expected response: `200 OK` with `temporary_permit_id="temp-permit-foundry-fragment-author-001"`.
    - Assertion: permit expires at `2026-05-20T14:15:00Z` and scope is one fragment id.

12. Update identity principal authority view.
    - API call: `GET /identity/v1/principals/oyatie.foundry.fragment-author/effective-authority`.
    - Expected response: `200 OK` with temporary authority `FRAGMENT_AUTHOR` and expiry.
    - Assertion: identity derives authority from governance permit, not local mutable role.

13. Seal elevation activation.
    - API call: `POST /audit-chain/v1/streams/tenant-oyatie.foundry.self-modification/events`.
    - Expected response: `201 Created` with `event_class="FoundryAgentPermitElevationActivated"`.
    - Assertion: event links human approval proof and witness quorum proof.

14. Use temporary permit to publish candidate fragment.
    - API call: `POST /governance/v1/cedar-fragments/fragment-cross-msvc-policy-fence-001/versions`.
    - Expected response: `201 Created` with `version="v2026-05-20.cmit-006"` and `state="SOAKING"`.
    - Assertion: fragment publish accepted only because temporary permit scope matches fragment id.

15. Attempt to publish a different fragment.
    - API call: `POST /governance/v1/cedar-fragments/fragment-payments-unrelated/versions`.
    - Expected response: `403 Forbidden` with `error.code="TEMPORARY_PERMIT_SCOPE_MISMATCH"`.
    - Assertion: one-fragment scope is enforced.

16. Attempt root key read.
    - API call: `GET /governance/v1/meta-trust/root-key-material` as `oyatie.foundry.fragment-author`.
    - Expected response: `403 Forbidden` with `error.code="ROOT_KEY_ACCESS_FORBIDDEN"`.
    - Assertion: denial cites `forbid-agent-root-key-access`.

17. Complete fragment soak.
    - API call: `POST /governance/v1/cedar-fragments/fragment-cross-msvc-policy-fence-001/versions/v2026-05-20.cmit-006:complete-soak`.
    - Expected response: `200 OK` with `state="SOAKED"`.
    - Assertion: soak duration meets or exceeds 60 seconds under seeded clock.

18. Activate fragment version.
    - API call: `POST /governance/v1/cedar-fragments/fragment-cross-msvc-policy-fence-001/versions/v2026-05-20.cmit-006:activate`.
    - Expected response: `200 OK` with `state="ACTIVE"`.
    - Assertion: activation references temporary permit id and witness proof.

19. Expire temporary permit.
    - API call: `POST /governance/v1/agent-permits/temp-permit-foundry-fragment-author-001:expireSandbox`.
    - Expected response: `200 OK` with `state="EXPIRED"`.
    - Assertion: identity effective authority returns to `DRAFT_ONLY`.

20. Attempt post-expiry fragment publish.
    - API call: `POST /governance/v1/cedar-fragments/fragment-cross-msvc-second/versions`.
    - Expected response: `403 Forbidden` with `error.code="TEMPORARY_PERMIT_EXPIRED"`.
    - Assertion: no residual authority remains.

21. Seal elevation closure.
    - API call: `POST /audit-chain/v1/streams/tenant-oyatie.foundry.self-modification/events`.
    - Expected response: `201 Created` with `event_class="FoundryAgentPermitElevationExpired"`.
    - Assertion: closure event references activated fragment version and permit expiry.

22. Reconstruct self-modification trace.
    - API call: `GET /audit-chain/v1/streams/tenant-oyatie.foundry.self-modification/events?trace_id=trace-cmit-006-permit-elevation`.
    - Expected response: `200 OK` with denial, rationale, request, approval, witness, activation, publish, and expiry events.
    - Assertion: every event has Merkle proof and actor chain.

23. Verify governance final summary.
    - API call: `GET /governance/v1/agent-permit-elevation-requests/elev-oyatie-foundry-fragment-author-001/summary`.
    - Expected response: `200 OK` with `final_state="EXPIRED_AFTER_SCOPED_USE"`.
    - Assertion: summary proves one authorized fragment activation and two denied overreach attempts.

24. Verify identity final authority.
    - API call: `GET /identity/v1/principals/oyatie.foundry.fragment-author/effective-authority`.
    - Expected response: `200 OK` with `authority_level="DRAFT_ONLY"` and no active temporary permits.
    - Assertion: self-modification did not leave standing elevated authority.

## Test Data Fixtures

### Fixture `FoundryAgentFixture`

```json
{
  "tenant_id": "tenant-oyatie",
  "foundry_scope": "oyatie.foundry",
  "agent_principal_before": "oyatie.foundry.reviewer-agent.facet-policy",
  "agent_principal_after": "oyatie.foundry.fragment-author",
  "baseline_authority": "DRAFT_ONLY",
  "requested_authority": "FRAGMENT_AUTHOR",
  "human_supervisor": "principal-nora-singh-platform"
}
```

### Fixture `ElevationRequestFixture`

```json
{
  "elevation_request_id": "elev-oyatie-foundry-fragment-author-001",
  "requested_by": "oyatie.foundry.reviewer-agent.facet-policy",
  "requested_authority": "FRAGMENT_AUTHOR",
  "scope": {
    "fragment_id": "fragment-cross-msvc-policy-fence-001",
    "version": "v2026-05-20.cmit-006",
    "actions": [
      "PUBLISH_FRAGMENT_VERSION",
      "ACTIVATE_FRAGMENT_VERSION"
    ]
  },
  "expires_at": "2026-05-20T14:15:00Z",
  "rationale_hash": "sha256:elevation-rationale-cmit-006"
}
```

### Fixture `WitnessQuorumFixture`

```yaml
witness_request_id: witness-quorum-root-5of9-cmit-006
attestor: oyatie.foundry.meta-trust-root-attestor
threshold: 5
share_count: 9
jurisdiction_floor: 3
root_key_material_exposed: false
result: QUORUM_VALID
signatures:
  - sig-us-1
  - sig-us-2
  - sig-kr-1
  - sig-eu-1
  - sig-jp-1
```

### Fixture `CandidateFragmentFixture`

```json
{
  "fragment_id": "fragment-cross-msvc-policy-fence-001",
  "version": "v2026-05-20.cmit-006",
  "target_policy_bundle": "cedar-bundle-2026-05-20-cross-msvc",
  "soak_seconds": 60,
  "content_hash": "sha256:fragment-cross-msvc-policy-fence-001-v2026-05-20",
  "allowed_actions": [
    "CrossMicroserviceTestRead",
    "CrossMicroserviceTestMutateUnderFixture"
  ]
}
```

### Fixture `TemporaryPermitFixture`

```yaml
temporary_permit_id: temp-permit-foundry-fragment-author-001
principal_id: oyatie.foundry.fragment-author
authority: FRAGMENT_AUTHOR
scope_fragment_id: fragment-cross-msvc-policy-fence-001
created_at: 2026-05-20T14:00:00Z
expires_at: 2026-05-20T14:15:00Z
human_approval_id: approval-nora-cmit-006
witness_request_id: witness-quorum-root-5of9-cmit-006
```

### Fixture `SelfModificationAuditFixture`

```yaml
stream: tenant-oyatie.foundry.self-modification
trace_id: trace-cmit-006-permit-elevation
events:
  - FoundryAgentSelfElevationDenied
  - FoundryAgentElevationRationaleGenerated
  - FoundryAgentPermitElevationRequested
  - FoundryAgentPermitElevationHumanApproved
  - FoundryAgentPermitElevationWitnessed
  - FoundryAgentPermitElevationActivated
  - FoundryCedarFragmentVersionPublished
  - FoundryCedarFragmentScopeMismatchDenied
  - FoundryRootKeyAccessDenied
  - FoundryCedarFragmentVersionActivated
  - FoundryAgentPermitElevationExpired
```

## Assertion Catalogue

### What passes

- `PASS-IDENTITY-001`: agent starts with draft-only authority.
- `PASS-IDENTITY-002`: identity resolves foundry sub-scope under tenant `oyatie`.
- `PASS-IDENTITY-003`: effective authority derives from governance permit.
- `PASS-IDENTITY-004`: authority returns to draft-only after expiry.
- `PASS-INTEL-001`: intelligence rationale is draft-only and metadata logged.
- `PASS-GOV-001`: direct self-elevation is forbidden.
- `PASS-GOV-002`: elevation request has explicit expiry.
- `PASS-GOV-003`: human supervisor approval is required.
- `PASS-GOV-004`: meta-trust witness quorum is required.
- `PASS-GOV-005`: temporary permit is scoped to one fragment.
- `PASS-GOV-006`: unrelated fragment publish is forbidden.
- `PASS-GOV-007`: root key material cannot be read.
- `PASS-GOV-008`: fragment soak is observed before activation.
- `PASS-AUDIT-001`: request is sealed.
- `PASS-AUDIT-002`: activation is sealed.
- `PASS-AUDIT-003`: expiry is sealed.
- `PASS-AUDIT-004`: trace includes denied overreach attempts.
- `PASS-SELF-MOD-001`: exactly one fragment is activated.
- `PASS-SELF-MOD-002`: no standing elevation remains.
- `PASS-SLO-001`: governance evaluation fits latency budget.

### What fails

- `FAIL-IDENTITY-001`: agent has standing author authority before request.
- `FAIL-IDENTITY-002`: authority remains elevated after expiry.
- `FAIL-INTEL-001`: intelligence run mutates policy directly.
- `FAIL-GOV-001`: direct self-elevation succeeds.
- `FAIL-GOV-002`: request has no expiry.
- `FAIL-GOV-003`: human approval actor is the agent.
- `FAIL-GOV-004`: witness quorum missing or single-jurisdiction.
- `FAIL-GOV-005`: temporary permit covers wildcard fragments.
- `FAIL-GOV-006`: unrelated fragment publish succeeds.
- `FAIL-GOV-007`: root key material is exposed.
- `FAIL-GOV-008`: activation skips soak.
- `FAIL-AUDIT-001`: activation not sealed.
- `FAIL-AUDIT-002`: denial events not sealed.
- `FAIL-SELF-MOD-001`: more than one fragment activated.
- `FAIL-SUMMARY-001`: summary omits overreach denials.

## Failure Mode Coverage

- `FM-ELEVATION-001`: agent self-elevates without human.
- `FM-ELEVATION-002`: temporary permit lacks expiry.
- `FM-ELEVATION-003`: identity persists elevated role locally.
- `FM-ELEVATION-004`: intelligence rationale becomes executable mutation.
- `FM-ELEVATION-005`: meta-trust attestor reads root key material.
- `FM-ELEVATION-006`: witness quorum is below 5-of-9.
- `FM-ELEVATION-007`: witness quorum lacks jurisdiction diversity.
- `FM-ELEVATION-008`: permit scope allows all fragments.
- `FM-ELEVATION-009`: unrelated fragment publish succeeds.
- `FM-ELEVATION-010`: fragment activation skips soak window.
- `FM-ELEVATION-011`: permit expiry does not revoke authority.
- `FM-ELEVATION-012`: audit stream lacks denied self-elevation.
- `FM-ELEVATION-013`: audit stream lacks root-key denial.
- `FM-ELEVATION-014`: audit stream lacks expiry closure.
- `FM-ELEVATION-015`: final summary hides denied overreach.
- `FM-ELEVATION-016`: trace id lost between intelligence and governance.
- `FM-ELEVATION-017`: human supervisor approves without seeing scope.
- `FM-ELEVATION-018`: fragment content hash changes after approval.
- `FM-ELEVATION-019`: active policy bundle includes fragment without activation event.
- `FM-ELEVATION-020`: post-expiry publish succeeds due to stale policy cache.

## Cross-µservice Handoff Validation

- `HANDOFF-IDENTITY-GOVERNANCE-OPENAPI`: governance reads identity principal and baseline authority.
- `HANDOFF-GOVERNANCE-IDENTITY-OPENAPI`: identity effective-authority endpoint reflects governance temporary permit.
- `HANDOFF-INTELLIGENCE-GOVERNANCE-OPENAPI`: elevation rationale hash is consumed by governance request.
- `HANDOFF-GOVERNANCE-INTELLIGENCE-ASYNCAPI`: governance final summary updates intelligence run finalization.
- `HANDOFF-GOVERNANCE-AUDIT-PROTO`: elevation request, activation, and expiry events match audit proto schema.
- `HANDOFF-INTELLIGENCE-AUDIT-PROTO`: rationale event includes model route and output hash.
- `HANDOFF-IDENTITY-AUDIT-PROTO`: effective authority changes are audit-linked to governance permit id.
- `HANDOFF-CEDAR`: direct self-elevation, root key access, and scope mismatch use distinct Cedar denials.
- `HANDOFF-META-TRUST`: witness proof schema includes threshold, jurisdiction count, and no key material.
- `HANDOFF-SOAK`: fragment activation endpoint requires soak-complete proof.
- `HANDOFF-TRACE`: trace id is preserved across identity, intelligence, governance, and audit-chain.
- `HANDOFF-IDEMPOTENCY`: duplicate elevation request returns the same request id.
- `HANDOFF-ERROR`: missing expiry maps to `ELEVATION_EXPIRY_REQUIRED`.
- `HANDOFF-REPLAY`: audit replay reconstructs all authority changes.
- `HANDOFF-REVERSIBILITY`: permit expiry is reflected in identity before post-expiry publish attempt.

## SLO Conformance

- `SLO-IDENTITY-READ-P95`: principal lookup P95 <= 150 ms.
- `SLO-INTELLIGENCE-RATIONALE-P95`: deterministic rationale run P95 <= 5 seconds.
- `SLO-GOV-REQUEST-P95`: elevation request create P95 <= 300 ms.
- `SLO-HUMAN-APPROVAL-P95`: human approval API P95 <= 300 ms excluding human review time.
- `SLO-WITNESS-COMPLETE-P95`: seeded witness completion P95 <= 600 ms.
- `SLO-PERMIT-ACTIVATE-P95`: temporary permit activation P95 <= 250 ms.
- `SLO-EFFECTIVE-AUTHORITY-P95`: identity authority refresh P95 <= 200 ms.
- `SLO-FRAGMENT-PUBLISH-P95`: fragment publish P95 <= 500 ms.
- `SLO-AUDIT-APPEND-P99`: audit append P99 <= 150 ms.
- `SLO-AUTHORITY-REVOCATION-P95`: expiry visible in identity P95 <= 300 ms.
- `SLO-AVAILABILITY`: identity, intelligence, governance, and audit-chain endpoints target 99.95 percent monthly availability.
- `SLO-THROUGHPUT`: one dev-tools cell supports 10 concurrent elevation ceremonies without permit collision.

## Reproducibility

- Named cargo test invocation: `cargo test -p oya-cross-msvc-tests ai_agent_permit_elevation -- --ignored --exact --test-threads=1`.
- Named deterministic seed: `CMIT-SEED-2026-05-20-FOUNDRY-PERMIT-ELEVATION`.
- Required fixture bundle: `fixtures/cross-msvc/foundry-permit-elevation.yaml`.
- Required policy bundle: `cedar-bundle-2026-05-20-cross-msvc`.
- Required witness mode: deterministic 5-of-9 sandbox witness proof.
- Required clock: `2026-05-20T14:00:00Z` with manual jump to expiry.
- Required intelligence mode: deterministic rationale stub.
- Test isolation: candidate fragment id is unique and deleted after audit proof export.
- Stop condition: one fragment activated, two overreach denials sealed, and final authority is draft-only.

## References

- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md`.
- `docs/decisions/ADR-0242-oyatie-is-a-tenant-doctrine.md`.
- `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- `docs/decisions/ADR-0247-self-hosting-self-modification-doctrine.md`.
- `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`.
- `docs/decisions/ADR-0263-observability-emission-contract.md`.
- `docs/standards/documentation-rigor.md` section 3.2.5 rows 19, 27, 28.
- `microservices/intelligence/contracts/openapi-v1.yaml`.
- `microservices/governance/contracts/openapi-v1.yaml`.
- `microservices/identity/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/audit-event-v1.proto`.
