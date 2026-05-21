---
doc_class: CrossMicroserviceIntegrationTest
scenario_id: CMIT-008-compliance-pack-activation-cascade
microservices_under_test:
  - compliance
  - governance
  - tenancy
  - identity
  - drive
  - mail
  - messenger
  - payments
  - workflow-engine
  - ontology
  - intelligence
  - audit-chain
status: draft-canonical
date: 2026-05-20
owner: codex-cross-msvc-integration-tests-w1
related_oyatie_adrs:
  - ADR-0113-vcs-orchestrator-end-to-end
  - ADR-0145-inter-microservice-communication-reform
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0251-compliance-pack-cell-certification-levels
  - ADR-0255-intelligence-as-two-layer-ai-substrate
  - ADR-0263-observability-emission-contract
  - ADR-0308-ml-model-lifecycle-ai-act-compliance
---

# Compliance Pack Activation Cascade

## Scenario Description

Dr. Amina Patel, compliance officer for `tenant-atlas-health-eu`, activates HIPAA and EU AI Act packs on a tenant that already uses Drive, Mail, Messenger, Payments, Workflow Engine, Ontology, and Intelligence; the cascade must publish per-µservice Cedar policy overlays and data-model deltas, prove each service acknowledged the pack, and seal the whole activation in audit-chain.

## Pre-conditions

### Named tenant fixtures

- Tenant: `tenant-atlas-health-eu`.
- Compliance officer: `principal-amina-patel-compliance`.
- Tenant region: `eu-central-1-cell-a`.
- DR pair: `eu-west-1-cell-b`.
- Compliance activation request: `pack-activation-atlas-hipaa-euai-001`.
- Pack one: `pack-HIPAA`.
- Pack two: `pack-EU-AI-Act`.
- Baseline pack: `pack-GDPR`.
- Governance policy bundle before: `cedar-bundle-atlas-baseline-2026-05-20`.
- Governance policy bundle after: `cedar-bundle-atlas-hipaa-euai-2026-05-20`.
- Data-model delta bundle: `delta-atlas-health-hipaa-euai-001`.
- Audit trace id: `trace-cmit-008-pack-cascade`.
- Idempotency key: `idem-cmit-008-pack-activation`.

### Named Cedar permits

- `permit-compliance-activate-pack`.
- `permit-governance-publish-pack-overlay`.
- `permit-tenancy-record-pack-activation`.
- `permit-identity-apply-healthcare-role-claims`.
- `permit-drive-apply-phi-labels`.
- `permit-mail-apply-phi-retention`.
- `permit-messenger-apply-clinical-message-controls`.
- `permit-payments-apply-healthcare-billing-controls`.
- `permit-workflow-engine-apply-healthcare-breakglass`.
- `permit-ontology-apply-ai-act-risk-fields`.
- `permit-intelligence-apply-eu-ai-act-model-controls`.
- `permit-audit-seal-pack-cascade`.
- `forbid-ai-high-risk-action-without-human-review`.
- `forbid-phi-export-without-hipaa-purpose`.

### Named pack activations

- Existing activation: `pack-GDPR`.
- New activation: `pack-HIPAA`.
- New activation: `pack-EU-AI-Act`.
- Cascade profile: `cascade-healthcare-ai-eu`.
- Required affected service roster: `atlas-health-affected-roster-2026-05-20`.
- Required rollback profile: `rollback-pack-activation-atlas-001`.

### Starting state checks

- Tenant is `ACTIVE` with GDPR baseline only.
- HIPAA pack is available for tenant region and cell tier.
- EU AI Act pack is available for tenant region and cell tier.
- Governance overlay registry has no Atlas-specific HIPAA/EU-AI bundle.
- Affected service roster includes 10 services plus compliance, governance, and audit-chain.
- Each affected service exposes `/pack-deltas:preview`.
- Audit stream `tenant-atlas-health-eu.compliance` exists.
- No pack activation event exists for this request id.

## Test Steps

1. Preview compliance pack activation.
   - API call: `POST /compliance/v1/tenants/tenant-atlas-health-eu/pack-activations:preview`.
   - Expected response: `200 OK` with affected services and delta bundle id.
   - Assertion: roster includes governance, tenancy, identity, drive, mail, messenger, payments, workflow-engine, ontology, intelligence, and audit-chain.

2. Validate pack compatibility.
   - API call: `POST /compliance/v1/pack-activations/pack-activation-atlas-hipaa-euai-001:validate`.
   - Expected response: `200 OK` with `compatible=true`.
   - Assertion: HIPAA and EU AI Act overlays do not conflict with existing GDPR baseline.

3. Create pack activation request.
   - API call: `POST /compliance/v1/tenants/tenant-atlas-health-eu/pack-activations`.
   - Expected response: `201 Created` with `state="AWAITING_GOVERNANCE_OVERLAY"`.
   - Assertion: request records both pack ids and rollback profile.

4. Publish governance Cedar overlay.
   - API call: `POST /governance/v1/policy-bundles/cedar-bundle-atlas-hipaa-euai-2026-05-20`.
   - Expected response: `201 Created` with `state="SOAKING"`.
   - Assertion: overlay includes HIPAA purpose-of-use and EU AI Act high-risk human-review rules.

5. Complete governance overlay soak.
   - API call: `POST /governance/v1/policy-bundles/cedar-bundle-atlas-hipaa-euai-2026-05-20:complete-soak`.
   - Expected response: `200 OK` with `state="READY_TO_ACTIVATE"`.
   - Assertion: soak duration is at least 60 seconds under seeded clock.

6. Record tenant-level pack activation.
   - API call: `POST /tenancy/v1/tenants/tenant-atlas-health-eu/pack-activations`.
   - Expected response: `200 OK` with active packs `GDPR`, `HIPAA`, and `EU-AI-Act`.
   - Assertion: tenancy records activation version and affected service roster.

7. Apply identity data-model delta.
   - API call: `POST /identity/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
   - Expected response: `200 OK` with added claims `hipaa_role`, `clinical_breakglass_eligible`, and `ai_act_reviewer`.
   - Assertion: identity requires explicit healthcare role claim for PHI access.

8. Apply drive data-model delta.
   - API call: `POST /drive/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
   - Expected response: `200 OK` with added labels `PHI`, `LEGAL_MEDICAL_RECORD`, and retention `HIPAA_6Y`.
   - Assertion: drive rejects PHI export unless purpose-of-use exists.

9. Apply mail data-model delta.
   - API call: `POST /mail/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
   - Expected response: `200 OK` with PHI retention and secure transport requirement.
   - Assertion: mail outbound PHI requires TLS policy and purpose tag.

10. Apply messenger data-model delta.
    - API call: `POST /messenger/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
    - Expected response: `200 OK` with clinical channel type and emergency break-glass flag.
    - Assertion: clinical messages gain HIPAA retention and audit class.

11. Apply payments data-model delta.
    - API call: `POST /payments/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
    - Expected response: `200 OK` with healthcare billing purpose codes.
    - Assertion: payment metadata separates billing data from PHI.

12. Apply workflow-engine data-model delta.
    - API call: `POST /workflow-engine/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
    - Expected response: `200 OK` with break-glass workflow and human-review gate.
    - Assertion: high-risk AI workflow steps require human reviewer.

13. Apply ontology data-model delta.
    - API call: `POST /ontology/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
    - Expected response: `200 OK` with node fields `ai_act_risk_tier`, `phi_classification`, and `clinical_basis`.
    - Assertion: ontology projections cannot omit risk-tier field for AI-assisted objects.

14. Apply intelligence data-model delta.
    - API call: `POST /intelligence/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
    - Expected response: `200 OK` with guardrail profile `eu-ai-act-high-risk-healthcare`.
    - Assertion: high-risk model action is blocked without human review.

15. Apply audit-chain data-model delta.
    - API call: `POST /audit-chain/v1/tenants/tenant-atlas-health-eu/pack-deltas:apply`.
    - Expected response: `200 OK` with event classes `PhiAccessed`, `AiHighRiskDecisionReviewed`, and `PackActivationCascaded`.
    - Assertion: audit-chain can seal every new event class before activation completes.

16. Activate governance policy bundle.
    - API call: `POST /governance/v1/policy-bundles/cedar-bundle-atlas-hipaa-euai-2026-05-20:activate`.
    - Expected response: `200 OK` with `state="ACTIVE"`.
    - Assertion: activation waits for every affected service delta acknowledgement.

17. Test HIPAA PHI export denial.
    - API call: `POST /drive/v1/files/phi-sample-chart-001:export` without purpose-of-use.
    - Expected response: `403 Forbidden` with `error.code="HIPAA_PURPOSE_REQUIRED"`.
    - Assertion: denial cites `forbid-phi-export-without-hipaa-purpose`.

18. Test HIPAA PHI export with purpose.
    - API call: `POST /drive/v1/files/phi-sample-chart-001:export` with purpose `TREATMENT`.
    - Expected response: `202 Accepted` with `export_id="phi-export-treatment-001"`.
    - Assertion: export emits `PhiAccessed` and purpose-of-use audit event.

19. Test EU AI Act high-risk denial.
    - API call: `POST /intelligence/v1/actions/clinical-triage:execute` without human reviewer.
    - Expected response: `403 Forbidden` with `error.code="AI_ACT_HUMAN_REVIEW_REQUIRED"`.
    - Assertion: denial cites `forbid-ai-high-risk-action-without-human-review`.

20. Test EU AI Act high-risk allowed with reviewer.
    - API call: `POST /workflow-engine/v1/workflows/clinical-triage-human-review:run`.
    - Expected response: `202 Accepted` with `workflow_execution_id="wfexec-ai-review-atlas-001"`.
    - Assertion: workflow includes human reviewer before intelligence action executes.

21. Seal pack activation cascade.
    - API call: `POST /audit-chain/v1/streams/tenant-atlas-health-eu.compliance/events`.
    - Expected response: `201 Created` with `event_class="PackActivationCascaded"`.
    - Assertion: event includes all service acknowledgements and governance bundle id.

22. Read compliance activation status.
    - API call: `GET /compliance/v1/pack-activations/pack-activation-atlas-hipaa-euai-001`.
    - Expected response: `200 OK` with `state="ACTIVE"`.
    - Assertion: state is active only when every service reports delta applied.

23. Replay affected-service acknowledgement stream.
    - API call: `GET /eventing/v1/replay?topic=compliance.pack.activation.ack.v1&trace_id=trace-cmit-008-pack-cascade`.
    - Expected response: `200 OK` with acknowledgements from all affected services.
    - Assertion: no service acknowledgement is missing or duplicated.

24. Verify rollback readiness.
    - API call: `GET /compliance/v1/pack-activations/pack-activation-atlas-hipaa-euai-001/rollback-readiness`.
    - Expected response: `200 OK` with `rollback_ready=true`.
    - Assertion: rollback profile can deactivate overlays in reverse dependency order without deleting audit evidence.

25. Verify final policy bundle everywhere.
    - API call: `GET /governance/v1/policy-bundles/active?tenant_id=tenant-atlas-health-eu`.
    - Expected response: `200 OK` with `active_bundle_id="cedar-bundle-atlas-hipaa-euai-2026-05-20"`.
    - Assertion: every affected service reports the same active bundle id.

## Test Data Fixtures

### Fixture `PackActivationRequestFixture`

```json
{
  "activation_request_id": "pack-activation-atlas-hipaa-euai-001",
  "tenant_id": "tenant-atlas-health-eu",
  "requested_by": "principal-amina-patel-compliance",
  "packs_to_activate": [
    "pack-HIPAA",
    "pack-EU-AI-Act"
  ],
  "existing_packs": [
    "pack-GDPR"
  ],
  "cascade_profile": "cascade-healthcare-ai-eu",
  "rollback_profile": "rollback-pack-activation-atlas-001"
}
```

### Fixture `AffectedServiceRosterFixture`

```yaml
roster_id: atlas-health-affected-roster-2026-05-20
services:
  - governance
  - tenancy
  - identity
  - drive
  - mail
  - messenger
  - payments
  - workflow-engine
  - ontology
  - intelligence
  - audit-chain
required_ack_count: 11
activation_requires_all_acks: true
```

### Fixture `GovernanceOverlayFixture`

```json
{
  "policy_bundle_id": "cedar-bundle-atlas-hipaa-euai-2026-05-20",
  "tenant_id": "tenant-atlas-health-eu",
  "fragments": [
    "policy/hipaa-purpose-of-use.cedar",
    "policy/hipaa-phi-export.cedar",
    "policy/eu-ai-act-high-risk-human-review.cedar",
    "policy/eu-ai-act-transparency.cedar"
  ],
  "forbids": [
    "forbid-phi-export-without-hipaa-purpose",
    "forbid-ai-high-risk-action-without-human-review"
  ],
  "soak_seconds": 60
}
```

### Fixture `DataModelDeltaFixture`

```json
{
  "delta_bundle_id": "delta-atlas-health-hipaa-euai-001",
  "tenant_id": "tenant-atlas-health-eu",
  "deltas": {
    "identity": [
      "hipaa_role",
      "clinical_breakglass_eligible",
      "ai_act_reviewer"
    ],
    "drive": [
      "phi_classification",
      "hipaa_retention_6y",
      "purpose_of_use_required"
    ],
    "ontology": [
      "ai_act_risk_tier",
      "clinical_basis",
      "phi_classification"
    ],
    "intelligence": [
      "high_risk_action_requires_human_review",
      "model_transparency_record"
    ]
  }
}
```

### Fixture `ServiceAcknowledgementFixture`

```yaml
trace_id: trace-cmit-008-pack-cascade
acks:
  governance: cedar-bundle-atlas-hipaa-euai-2026-05-20
  tenancy: pack-activation-atlas-hipaa-euai-001
  identity: delta-atlas-health-identity-001
  drive: delta-atlas-health-drive-001
  mail: delta-atlas-health-mail-001
  messenger: delta-atlas-health-messenger-001
  payments: delta-atlas-health-payments-001
  workflow-engine: delta-atlas-health-workflow-001
  ontology: delta-atlas-health-ontology-001
  intelligence: delta-atlas-health-intelligence-001
  audit-chain: delta-atlas-health-audit-001
```

### Fixture `RuntimePolicyProbeFixture`

```json
{
  "phi_export_without_purpose": {
    "api": "POST /drive/v1/files/phi-sample-chart-001:export",
    "expected_status": 403,
    "expected_error": "HIPAA_PURPOSE_REQUIRED"
  },
  "phi_export_with_treatment_purpose": {
    "api": "POST /drive/v1/files/phi-sample-chart-001:export",
    "expected_status": 202,
    "expected_event": "PhiAccessed"
  },
  "ai_high_risk_without_review": {
    "api": "POST /intelligence/v1/actions/clinical-triage:execute",
    "expected_status": 403,
    "expected_error": "AI_ACT_HUMAN_REVIEW_REQUIRED"
  }
}
```

### Fixture `PackCascadeAuditFixture`

```yaml
stream: tenant-atlas-health-eu.compliance
trace_id: trace-cmit-008-pack-cascade
events:
  - PackActivationPreviewed
  - PackActivationValidated
  - GovernanceOverlaySoaked
  - ServicePackDeltaApplied
  - GovernanceOverlayActivated
  - PhiExportDenied
  - PhiAccessed
  - AiHighRiskActionDenied
  - AiHighRiskDecisionReviewed
  - PackActivationCascaded
```

## Assertion Catalogue

### What passes

- `PASS-COMPLIANCE-001`: preview enumerates every affected service.
- `PASS-COMPLIANCE-002`: compatibility check includes GDPR, HIPAA, and EU AI Act.
- `PASS-COMPLIANCE-003`: activation request stores rollback profile.
- `PASS-GOV-001`: Cedar overlay includes HIPAA purpose-of-use.
- `PASS-GOV-002`: Cedar overlay includes EU AI Act human-review forbid.
- `PASS-GOV-003`: overlay soaks before activation.
- `PASS-TENANCY-001`: tenant active packs include GDPR, HIPAA, and EU AI Act.
- `PASS-IDENTITY-001`: identity adds healthcare role claims.
- `PASS-DRIVE-001`: drive adds PHI labels and retention.
- `PASS-MAIL-001`: mail adds PHI secure transport policy.
- `PASS-MESSENGER-001`: messenger adds clinical channel controls.
- `PASS-PAYMENTS-001`: payments separates healthcare billing from PHI.
- `PASS-WORKFLOW-001`: workflow-engine adds human-review gate.
- `PASS-ONTOLOGY-001`: ontology adds AI risk and PHI fields.
- `PASS-INTEL-001`: intelligence adds high-risk guardrail profile.
- `PASS-AUDIT-001`: audit-chain registers new event classes.
- `PASS-RUNTIME-001`: PHI export without purpose is denied.
- `PASS-RUNTIME-002`: PHI export with treatment purpose is allowed and audited.
- `PASS-RUNTIME-003`: high-risk AI action without human review is denied.
- `PASS-CASCADE-001`: activation reaches active only after all acknowledgements.

### What fails

- `FAIL-COMPLIANCE-001`: affected service roster omits a service.
- `FAIL-COMPLIANCE-002`: activation ignores GDPR compatibility.
- `FAIL-GOV-001`: overlay activates before soak.
- `FAIL-GOV-002`: overlay lacks high-risk AI human-review forbid.
- `FAIL-TENANCY-001`: tenant active pack list not updated.
- `FAIL-IDENTITY-001`: healthcare role claim missing.
- `FAIL-DRIVE-001`: PHI export allowed without purpose.
- `FAIL-MAIL-001`: mail PHI transport requirement missing.
- `FAIL-MESSENGER-001`: clinical message controls missing.
- `FAIL-PAYMENTS-001`: payment metadata stores PHI.
- `FAIL-WORKFLOW-001`: workflow high-risk step has no human reviewer.
- `FAIL-ONTOLOGY-001`: AI risk tier field missing.
- `FAIL-INTEL-001`: high-risk action executes without review.
- `FAIL-AUDIT-001`: audit event class missing.
- `FAIL-CASCADE-001`: active state before all acknowledgements.

## Failure Mode Coverage

- `FM-PACK-001`: compliance preview misses affected microservice.
- `FM-PACK-002`: incompatible pack combination activates.
- `FM-PACK-003`: governance overlay skips soak.
- `FM-PACK-004`: tenancy pack activation ledger not updated.
- `FM-PACK-005`: identity claims lag behind active pack.
- `FM-PACK-006`: drive PHI labels missing.
- `FM-PACK-007`: mail sends PHI without secure transport.
- `FM-PACK-008`: messenger clinical channel lacks retention.
- `FM-PACK-009`: payments stores PHI in billing metadata.
- `FM-PACK-010`: workflow high-risk AI step lacks human-review gate.
- `FM-PACK-011`: ontology projection omits AI risk tier.
- `FM-PACK-012`: intelligence guardrail profile not switched.
- `FM-PACK-013`: audit-chain cannot seal new event class.
- `FM-PACK-014`: PHI export allowed without purpose-of-use.
- `FM-PACK-015`: high-risk AI action executes without reviewer.
- `FM-PACK-016`: service acknowledgement duplicated.
- `FM-PACK-017`: service acknowledgement missing but activation succeeds.
- `FM-PACK-018`: active policy bundle differs across services.
- `FM-PACK-019`: rollback profile cannot reverse dependency order.
- `FM-PACK-020`: cascade audit event omits service acknowledgement.

## Cross-µservice Handoff Validation

- `HANDOFF-COMPLIANCE-GOVERNANCE-OPENAPI`: compliance preview produces governance overlay publish request.
- `HANDOFF-GOVERNANCE-COMPLIANCE-ASYNCAPI`: governance overlay soak and activation events update compliance state.
- `HANDOFF-COMPLIANCE-TENANCY-OPENAPI`: tenancy records pack activation id and pack versions.
- `HANDOFF-COMPLIANCE-IDENTITY-OPENAPI`: identity delta applies healthcare role claims.
- `HANDOFF-COMPLIANCE-DRIVE-OPENAPI`: drive delta applies PHI labels and retention.
- `HANDOFF-COMPLIANCE-MAIL-OPENAPI`: mail delta applies PHI transport requirements.
- `HANDOFF-COMPLIANCE-MESSENGER-OPENAPI`: messenger delta applies clinical channel controls.
- `HANDOFF-COMPLIANCE-PAYMENTS-OPENAPI`: payments delta applies healthcare billing controls.
- `HANDOFF-COMPLIANCE-WORKFLOW-OPENAPI`: workflow-engine delta applies human-review gates.
- `HANDOFF-COMPLIANCE-ONTOLOGY-OPENAPI`: ontology delta applies risk-tier fields.
- `HANDOFF-COMPLIANCE-INTELLIGENCE-OPENAPI`: intelligence delta applies high-risk guardrail profile.
- `HANDOFF-COMPLIANCE-AUDIT-OPENAPI`: audit-chain delta registers event classes before activation.
- `HANDOFF-GOVERNANCE-ALL`: every service reports same active Cedar bundle id.
- `HANDOFF-AUDIT-PROTO`: pack cascade, PHI access, and AI review events conform to audit proto.
- `HANDOFF-TRACE`: trace id spans all affected microservices.
- `HANDOFF-IDEMPOTENCY`: duplicate activation request returns the same activation id.
- `HANDOFF-ERROR`: missing acknowledgement keeps compliance state `BLOCKED_ON_SERVICE_ACK`.
- `HANDOFF-ROLLBACK`: rollback readiness lists reverse dependency order.
- `HANDOFF-RUNTIME-PROBE`: runtime probes prove policy changed after activation.
- `HANDOFF-REPLAY`: acknowledgement stream replay matches compliance activation status.

## SLO Conformance

- `SLO-PREVIEW-P95`: activation preview P95 <= 1000 ms.
- `SLO-COMPATIBILITY-P95`: pack compatibility validation P95 <= 800 ms.
- `SLO-GOV-PUBLISH-P95`: governance overlay publish P95 <= 700 ms.
- `SLO-SERVICE-DELTA-P95`: each service delta apply P95 <= 900 ms.
- `SLO-AUDIT-DELTA-P95`: audit-chain event class registration P95 <= 600 ms.
- `SLO-POLICY-ACTIVATE-P95`: governance bundle activation P95 <= 500 ms after acknowledgements.
- `SLO-RUNTIME-DENY-P95`: PHI and AI Act denials P95 <= 250 ms.
- `SLO-RUNTIME-ALLOW-P95`: treatment-purpose PHI export acceptance P95 <= 600 ms.
- `SLO-CASCADE-END-P95`: full cascade P95 <= 20 seconds excluding soak clock.
- `SLO-ACK-REPLAY-P95`: acknowledgement replay P95 <= 2 seconds.
- `SLO-AVAILABILITY`: all affected service endpoints target 99.95 percent monthly availability.
- `SLO-THROUGHPUT`: one tenant supports 5 pack activation cascades per hour with no policy-bundle collision.
- `SLO-CONSISTENCY`: active policy bundle convergence across services <= 2 seconds after activation.

## Reproducibility

- Named cargo test invocation: `cargo test -p oya-cross-msvc-tests compliance_pack_activation_cascade -- --ignored --exact --test-threads=1`.
- Named deterministic seed: `CMIT-SEED-2026-05-20-PACK-CASCADE-ATLAS`.
- Required fixture bundle: `fixtures/cross-msvc/compliance-pack-cascade-atlas-health.yaml`.
- Required policy baseline: `cedar-bundle-atlas-baseline-2026-05-20`.
- Required policy target: `cedar-bundle-atlas-hipaa-euai-2026-05-20`.
- Required delta bundle: `delta-atlas-health-hipaa-euai-001`.
- Required clock: `2026-05-20T14:00:00Z` with seeded soak jump.
- Required runtime probes: PHI export deny, PHI export allow, high-risk AI deny, high-risk AI reviewed allow.
- Test isolation: pack activation is rolled back only after audit proof export.
- Stop condition: compliance activation is active, all acknowledgements replay, runtime probes pass, and rollback readiness is true.

## References

- `docs/decisions/ADR-0113-vcs-orchestrator-end-to-end.md`.
- `docs/decisions/ADR-0145-inter-microservice-communication-reform.md`.
- `docs/decisions/ADR-0243-cedar-as-universal-gate.md`.
- `docs/decisions/ADR-0244-tenant-as-universal-scoping-primitive.md`.
- `docs/decisions/ADR-0251-compliance-pack-cell-certification-levels.md`.
- `docs/decisions/ADR-0255-intelligence-as-two-layer-ai-substrate.md`.
- `docs/decisions/ADR-0263-observability-emission-contract.md`.
- `docs/standards/documentation-rigor.md` section 3.2.5 rows 5, 17, 18, 23, 28.
- `registry/compliance-packs/HIPAA.yaml`.
- `registry/compliance-packs/EU-AI-Act.yaml`.
- `microservices/compliance/contracts/openapi-v1.yaml`.
- `microservices/governance/contracts/openapi-v1.yaml`.
- `microservices/drive/contracts/openapi-v1.yaml`.
- `microservices/intelligence/contracts/openapi-v1.yaml`.
- `microservices/audit-chain/contracts/audit-event-v1.proto`.
