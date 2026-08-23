---
doc_class: Tutorial
tutorial_id: TUT-OYATIE-MULTIPACK-007
persona: "Omar Haddad, compliance platform administrator"
prerequisite_packs:
  - canonical-base
  - tenant-admin
  - compliance-pack-operations
related_oyatie_adrs:
  - ADR-0240
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0263
  - ADR-0316
status: Draft
date: 2026-05-20
owner: docs-experience
estimated_completion_time: "95 minutes"
---

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

# Activate HIPAA, SOC 2, GDPR, and KR-PIPA on a Single Tenant

## Goal

You will activate four compliance packs on `tenant-healthbridge-global`, resolve their overlay order, test a protected health workflow, test an EU data subject view, test a Korean resident privacy view, and verify that the tenant can run all packs without creating product-specific service forks.

## Prerequisites

- Tenant admin account: `omar.haddad@healthbridge.example`.
- Tenant id: `tenant-healthbridge-global`.
- Tenant display name: `HealthBridge Global`.
- Home cell: `us-east-health-1`.
- EU cell: `eu-frankfurt-1`.
- KR cell: `kr-seoul-1`.
- Packs to activate: `HIPAA`, `SOC2-Type-II`, `GDPR`, `KR-PIPA`.
- Capability tier: `health-operations-core`.
- Capability tier: `content-classification-core`.
- Capability tier: `executive-dashboard-core`.
- Subscribed microservices: `tenancy`, `policy-engine`, `compliance`, `audit-chain`, `ontology`, `workflow-engine`, `drive`, `mail`, `messenger`, `intelligence`, `observability`.
- Required Cedar permit: `tenant.pack.activate`.
- Required Cedar permit: `tenant.pack.precedence.set`.
- Required Cedar permit: `compliance.attestation.generate`.
- Required Cedar permit: `policy.pack.evaluate`.
- Required Cedar permit: `ontology.projection.read`.
- Required Cedar permit: `workflow.run.start`.
- Required Cedar permit: `audit.compliance.read`.
- Test patient id: `patient-riley-chen-001`.
- Test EU subject id: `subject-elena-martin-001`.
- Test KR subject id: `subject-minji-kang-001`.
- Evidence bundle id: `pack-activation-healthbridge-2026-05-20`.
- Named saved query: `tutorial.multi_pack_tenant_activation_status`.

## Step-by-Step

1. Open the tenant compliance console.
   - Sign in as `omar.haddad@healthbridge.example`.
   - Switch to `HealthBridge Global`.
   - Open `Admin -> Compliance packs`.
   - Confirm tenant context: `tenant-healthbridge-global`.
   - Confirm current active pack: `canonical-base`.
   - Confirm admin role: `Compliance Platform Admin`.
   - Screenshot checkpoint: capture the pack overview.
   - If another tenant is active, switch before continuing.
   - This tutorial applies all packs to one tenant, not four tenant clones.
   - Keep the `Pack activation timeline` panel visible.

2. Run pack readiness scan.
   - Click `Run readiness scan`.
   - Select packs: `HIPAA`, `SOC2-Type-II`, `GDPR`, `KR-PIPA`.
   - Scan scope: `tenant-wide`.
   - Include capability tiers: enabled.
   - Include Cedar coverage: enabled.
   - Include ontology projections: enabled.
   - Include audit-chain retention: enabled.
   - Click `Start scan`.
   - Expected state: `Readiness scan running`.
   - Screenshot checkpoint: capture scan configuration.

3. Review readiness output.
   - HIPAA row should show `BAA required: satisfied`.
   - SOC 2 row should show `control evidence pipeline: ready`.
   - GDPR row should show `DSR workflows: ready`.
   - KR-PIPA row should show `resident privacy overlay: ready`.
   - Cedar coverage should show `no missing forbid rules`.
   - Ontology projections should show `schema pins present`.
   - Audit retention should show `pack-aware retention enabled`.
   - Click `Export readiness`.
   - Save as `healthbridge-pack-readiness.json`.
   - Screenshot checkpoint: capture all green readiness rows.

4. Activate HIPAA.
   - Click `Activate pack`.
   - Choose `HIPAA`.
   - Pack id: `HIPAA`.
   - Overlay id: `pack-us-healthcare-hipaa`.
   - Data classes: `PHI`, `PII`, `Confidential`.
   - Required contracts: `BAA`.
   - Minimum necessary: enabled.
   - Audit retention: `P6Y`.
   - Click `Activate`.
   - Expected toast: `HIPAA pack activated`.
   - Screenshot checkpoint: capture HIPAA active state.

5. Activate SOC 2 Type II.
   - Click `Activate pack`.
   - Choose `SOC2-Type-II`.
   - Overlay id: `pack-soc2-type-ii`.
   - Trust services categories: `security`, `availability`, `confidentiality`.
   - Evidence cadence: `daily`.
   - Auditor portal: enabled.
   - Control exceptions: `none`.
   - Click `Activate`.
   - Expected toast: `SOC 2 Type II pack activated`.
   - Screenshot checkpoint: capture SOC 2 active state.
   - SOC 2 supplies evidence discipline but does not weaken HIPAA.

6. Activate GDPR.
   - Click `Activate pack`.
   - Choose `GDPR`.
   - Overlay id: `EU-GDPR-2018-baseline`.
   - Jurisdiction: `EU`.
   - DSR SLA: `P30D`.
   - Lawful basis registry: enabled.
   - Data minimization: enabled.
   - Cross-border transfer check: enabled.
   - Click `Activate`.
   - Expected toast: `GDPR pack activated`.
   - Screenshot checkpoint: capture GDPR active state.
   - Confirm EU cell `eu-frankfurt-1` is available.

7. Activate KR-PIPA.
   - Click `Activate pack`.
   - Choose `KR-PIPA`.
   - Overlay id: `KR-PIPA-2023-amendment`.
   - Jurisdiction: `KR`.
   - Resident notice language: `Korean`.
   - Cross-border transfer notice: enabled.
   - Local evidence export: enabled.
   - KR cell: `kr-seoul-1`.
   - Click `Activate`.
   - Expected toast: `KR-PIPA pack activated`.
   - Screenshot checkpoint: capture KR-PIPA active state.
   - Confirm Korean-language notice template is linked.

8. Set overlay precedence.
   - Open `Pack precedence`.
   - Set base layer: `canonical-base`.
   - Set regulated health layer: `HIPAA`.
   - Set assurance layer: `SOC2-Type-II`.
   - Set regional privacy layer order: `GDPR`, then `KR-PIPA`.
   - Conflict rule: `Most restrictive applicable rule wins`.
   - Evidence rule: `Emit one audit event per pack decision`.
   - Click `Save precedence`.
   - Expected toast: `Pack precedence saved`.
   - Screenshot checkpoint: capture the precedence graph.
   - Do not use alphabetical precedence.

9. Bind packs to capability tiers.
   - Open `Capability tiers`.
   - Select `health-operations-core`.
   - Add pack overlays: `HIPAA`, `GDPR`, `KR-PIPA`.
   - Select `content-classification-core`.
   - Add pack overlays: `SOC2-Type-II`, `GDPR`, `KR-PIPA`.
   - Select `executive-dashboard-core`.
   - Add pack overlays: `SOC2-Type-II`.
   - Click `Save tier bindings`.
   - Expected toast: `Capability tier pack overlays saved`.
   - Screenshot checkpoint: capture tier binding matrix.

10. Validate Cedar pack evaluation.
    - Open `Policy -> Pack evaluation`.
    - Principal: `clinician.rivera@healthbridge.example`.
    - Action: `health.record.read`.
    - Resource: `patient-riley-chen-001`.
    - Context data class: `PHI`.
    - Context jurisdiction: `US`.
    - Active packs: `HIPAA`, `SOC2-Type-II`.
    - Click `Evaluate`.
    - Expected decision: `Permit with HIPAA minimum necessary`.
    - Screenshot checkpoint: capture the Cedar decision.
    - Confirm audit event class `PackPolicyEvaluated`.

11. Test EU subject access behavior.
    - Change principal: `privacy.elena@healthbridge.example`.
    - Action: `privacy.dsr.case.create`.
    - Resource: `subject-elena-martin-001`.
    - Context jurisdiction: `EU`.
    - Active packs: `GDPR`, `SOC2-Type-II`.
    - Click `Evaluate`.
    - Expected decision: `Permit with GDPR DSR SLA`.
    - Expected SLA: `P30D`.
    - Screenshot checkpoint: capture GDPR decision.
    - Confirm HIPAA is not applied when resource is not PHI.
    - Close the policy drawer.

12. Test KR resident behavior.
    - Open `Policy -> Pack evaluation`.
    - Principal: `privacy.minji@healthbridge.example`.
    - Action: `privacy.notice.send`.
    - Resource: `subject-minji-kang-001`.
    - Context jurisdiction: `KR`.
    - Active packs: `KR-PIPA`, `SOC2-Type-II`.
    - Click `Evaluate`.
    - Expected decision: `Permit with Korean resident notice`.
    - Expected language: `ko-KR`.
    - Expected transfer notice: `required`.
    - Screenshot checkpoint: capture KR-PIPA decision.
    - Confirm GDPR does not override KR resident notice language.

13. Run a protected health workflow.
    - Open `Workflow -> Templates`.
    - Select `HIPAA care summary review`.
    - Patient id: `patient-riley-chen-001`.
    - Reviewer: `clinician.rivera@healthbridge.example`.
    - Data class: `PHI`.
    - Active packs shown: `HIPAA`, `SOC2-Type-II`.
    - Click `Start run`.
    - Expected run state: `Started`.
    - Expected notice: `Minimum necessary applies`.
    - Screenshot checkpoint: capture workflow run header.
    - Do not attach non-redacted training data.

14. Verify ontology projection filters.
    - Open `Ontology -> Projection Explorer`.
    - Projection: `projection:care-team-handoff:v1`.
    - Tenant: `tenant-healthbridge-global`.
    - Subject: `patient-riley-chen-001`.
    - Role: `clinician`.
    - Active packs: `HIPAA`, `SOC2-Type-II`.
    - Click `Preview projection`.
    - Expected visible fields: `care_plan`, `allergies`, `next_visit`.
    - Expected redacted fields: `billing_notes`, `unrelated_family_history`.
    - Screenshot checkpoint: capture field visibility.

15. Generate the compliance attestation bundle.
    - Open `Compliance -> Evidence bundles`.
    - Click `New bundle`.
    - Bundle id: `pack-activation-healthbridge-2026-05-20`.
    - Include packs: `HIPAA`, `SOC2-Type-II`, `GDPR`, `KR-PIPA`.
    - Include readiness scan: enabled.
    - Include policy evaluations: enabled.
    - Include workflow run evidence: enabled.
    - Include ontology projection preview: enabled.
    - Click `Generate bundle`.
    - Expected state: `Bundle generated`.

16. Review bundle contents.
    - Open the generated bundle.
    - Confirm `healthbridge-pack-readiness.json` is included.
    - Confirm `PackActivated` events for all four packs.
    - Confirm `PackPrecedenceSaved` event.
    - Confirm `CapabilityTierPackBound` events.
    - Confirm `PackPolicyEvaluated` examples.
    - Confirm `WorkflowRunStarted` example.
    - Confirm `OntologyProjectionPreviewed` example.
    - Screenshot checkpoint: capture the bundle table of contents.
    - Export as `pack-activation-healthbridge-2026-05-20.pdf`.

17. Run final verification query.
    - Open `Compliance -> Saved checks`.
    - Choose `tutorial.multi_pack_tenant_activation_status`.
    - Input `tenant_id=tenant-healthbridge-global`.
    - Input `bundle_id=pack-activation-healthbridge-2026-05-20`.
    - Input `expected_packs=HIPAA,SOC2-Type-II,GDPR,KR-PIPA`.
    - Click `Run`.
    - Expected title: `Multi-pack tenant activation complete`.
    - Expected state: `PASS`.
    - Screenshot checkpoint: capture the query output.
    - Save output to `Compliance Evidence -> Multi-pack Activation`.
    - This is the stop condition for the tutorial.

## Verification

- Named query: `tutorial.multi_pack_tenant_activation_status`.
- Query location: `Compliance -> Saved checks`.
- Query input `tenant_id`: `tenant-healthbridge-global`.
- Query input `bundle_id`: `pack-activation-healthbridge-2026-05-20`.
- Query input `expected_packs`: `HIPAA,SOC2-Type-II,GDPR,KR-PIPA`.
- Expected output field: `active_pack_count`.
- Expected output value: `4`.
- Expected output field: `hipaa_state`.
- Expected output value: `active`.
- Expected output field: `soc2_state`.
- Expected output value: `active`.
- Expected output field: `gdpr_state`.
- Expected output value: `active`.
- Expected output field: `kr_pipa_state`.
- Expected output value: `active`.
- Expected output field: `precedence_rule`.
- Expected output value: `most_restrictive_applicable_rule_wins`.
- Expected output field: `capability_tier_bindings_present`.
- Expected output value: `true`.
- Expected output field: `cedar_pack_evaluations_passed`.
- Expected output value: `3`.
- Expected output field: `ontology_projection_filters_passed`.
- Expected output value: `true`.
- Expected output field: `attestation_bundle_generated`.
- Expected output value: `true`.
- Expected output field: `audit_chain_seals_present`.
- Expected output value: `true`.
- Expected output field: `result_label`.
- Expected output value: `Multi-pack tenant activation complete`.
- CLI equivalent:

```bash
oya compliance verify multi-pack \
  --tenant tenant-healthbridge-global \
  --bundle pack-activation-healthbridge-2026-05-20 \
  --packs HIPAA,SOC2-Type-II,GDPR,KR-PIPA
```

- CLI expected line: `PASS tutorial.multi_pack_tenant_activation_status`.
- CLI expected line: `active_pack_count=4`.
- CLI expected line: `precedence_rule=most_restrictive_applicable_rule_wins`.
- CLI expected line: `attestation_bundle_generated=true`.
- Audit event to inspect: `PackActivated`.
- Audit event to inspect: `PackPrecedenceSaved`.
- Audit event to inspect: `CapabilityTierPackBound`.
- Audit event to inspect: `PackPolicyEvaluated`.
- Audit event to inspect: `ComplianceAttestationBundleGenerated`.
- Evidence artifact: `healthbridge-pack-readiness.json`.
- Evidence artifact: `pack-activation-healthbridge-2026-05-20.pdf`.
- Dashboard: `Compliance -> Pack attestation lag`.
- Expected tile: `HealthBridge Global - no pack drift`.

## Common Pitfalls + Recovery

- Pitfall: packs are activated on separate cloned tenants.
- Recovery: activate all four packs on `tenant-healthbridge-global` and delete unused test clones.
- Pitfall: HIPAA is active but no BAA evidence is attached.
- Recovery: attach BAA evidence before running protected health workflows.
- Pitfall: SOC 2 is treated as a data residency rule.
- Recovery: use SOC 2 for control evidence; keep residency decisions in GDPR and KR-PIPA overlays.
- Pitfall: precedence is alphabetical.
- Recovery: set `Most restrictive applicable rule wins` with explicit regional overlays.
- Pitfall: GDPR applies to KR resident notices.
- Recovery: inspect jurisdiction context and ensure KR-PIPA handles Korean resident notice language.
- Pitfall: capability tiers have no pack bindings.
- Recovery: bind packs to `health-operations-core`, `content-classification-core`, and `executive-dashboard-core`.
- Pitfall: Cedar evaluation permits PHI with no minimum necessary reason.
- Recovery: repair HIPAA policy overlay before running health workflows.
- Pitfall: ontology projection shows billing notes to clinician role.
- Recovery: update projection filters and rerun preview.
- Pitfall: attestation bundle excludes policy evaluations.
- Recovery: regenerate with `Include policy evaluations` enabled.
- Pitfall: audit-chain seal is missing from a pack activation.
- Recovery: rerun pack activation evidence emission or block tenant promotion.
- Pitfall: KR cell is not available.
- Recovery: provision `kr-seoul-1` or delay KR-PIPA activation until ready.
- Pitfall: EU DSR SLA is not `P30D`.
- Recovery: edit GDPR overlay and rerun pack verification.
- Pitfall: a pack is in `preview` state.
- Recovery: promote it to `active` only after readiness scan passes.
- Pitfall: the tenant admin uses a product-specific service fork.
- Recovery: keep behavior in pack overlays and capability-tier bindings per ADR-0316.
- Pitfall: evidence files are stored outside tenant evidence.
- Recovery: move them to `Compliance Evidence -> Multi-pack Activation`.
- Pitfall: tests use real patient PHI.
- Recovery: replace with training fixture `patient-riley-chen-001`.
- Pitfall: policy evaluation uses stale context.
- Recovery: refresh active pack context before running the evaluation.
- Pitfall: the query reports pack drift.
- Recovery: open the drifting pack row and replay the latest `PackActivated` event.

## Multi-Pack Collision Checks

Run these checks before telling a tenant that all four packs are active.

- HIPAA should enforce minimum necessary access for PHI workflows.
- SOC 2 should enforce evidence retention and change-management review.
- GDPR should enforce DSR SLA `P30D`.
- KR-PIPA should enforce Korean residency and privacy notice requirements.
- The active cell for KR data should be `kr-seoul-1`.
- EU subject request routing should remain in the EU policy context.
- Audit-chain should seal one activation event per pack.
- Cedar should evaluate the combined context, not four isolated happy paths.
- Ontology should hide billing notes from clinician roles.
- Workflow should block PHI export without minimum necessary reason.
- Evidence bundle should include policy evaluations, not only pack names.
- Observability should report pack drift as `0`.

The useful proof is a collision-free combined posture.

Do not accept four independent `active` labels if the combined policy context was never evaluated.

Do not resolve conflicts by weakening the stricter pack.

Prefer a narrower workflow exception with named approval and audit evidence.

The tutorial is complete when `tutorial.multi_pack_activation_status` returns `combined_policy_ready`.

## Next Tutorials

- [Handle a GDPR erasure request](data-subject-erasure-request-handling.md).
- [Capture and propagate consent across microservices](consent-cascade-across-microservices.md).
- [Use intelligence to summarize a 200-page contract](ai-assisted-document-summarization.md).
- [Build an employee-onboarding workflow](workflow-studio-build-employee-onboarding.md).

## References

- [HIPAA compliance pack](../../registry/compliance-packs/HIPAA.yaml).
- [SOC 2 Type II compliance pack](../../registry/compliance-packs/SOC2-Type-II.yaml).
- [GDPR compliance pack](../../registry/compliance-packs/GDPR.yaml).
- [KR-PIPA compliance pack](../../registry/compliance-packs/KR-PIPA.yaml).
- [Compliance evidence automation standard](../standards/compliance-evidence-automation.md).
- [Capability Tier Over Product Fragmentation ADR](../decisions/ADR-0316-capability-tier-over-product-fragmentation.md).
- [Capability tier matrix standard](../standards/capability-tier-matrix.md).
- [Pack overlay schema](../../specs/pack-overlay-schema.json).
- [Documentation Rigor](../standards/documentation-rigor.md).
- [Doc Style](../standards/doc-style.md).
