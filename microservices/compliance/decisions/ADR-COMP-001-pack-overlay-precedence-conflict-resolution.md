---
id: ADR-COMP-001
title: Pack Overlay Precedence and Multi-Pack Conflict Resolution
status: Accepted
date: 2026-05-20
microservice: compliance
related_oyatie_adrs:
  - docs/decisions/ADR-0003-audit-chain-and-evidence-emission.md
  - docs/decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md
  - docs/decisions/ADR-0008-data-use-boundary.md
  - docs/decisions/ADR-0010-regional-pack-architecture.md
  - docs/decisions/ADR-0304-cross-jurisdiction-conflict-resolution.md
decision_owner: axis-compliance
---

# ADR-COMP-001: Pack Overlay Precedence and Multi-Pack Conflict Resolution

## Context

- Compliance owns evidence collection, SOC 2 mapping, GDPR DSR, HIPAA minimum necessary logging, audit seal coverage, retention tiers, regulator audit evidence, DPIA orchestration, breach notification, pack registry, and control mapping.
- Existing policies include `pack-overlay-authorization.cedar`, `action-authorization.cedar`, `auditor-scope.cedar`, `data-residency.cedar`, and `data-residency.md`.
- Existing scorecards include SOC 2 Type 2, GDPR, HIPAA, PCI DSS, and override records.
- Existing dashboards include pack-overlay coverage, evidence coverage, audit-chain seal health, DSAR pipeline, and breach notification SLA.
- ADR-0304 establishes cross-jurisdiction conflict doctrine; this ADR binds compliance-local pack overlay precedence and conflict resolution.
- Named precedent: AWS Artifact and Audit Manager model controls as evidence packs but defer enforcement to account and region policy.
- Named precedent: Google Assured Workloads applies regional and compliance controls as overlays on tenant resources.
- Named precedent: Microsoft Purview compliance boundaries and retention labels apply stricter controls when multiple policies target content.
- Constraint COMP-C1: audit evidence and pack decisions must write to audit-chain per ADR-0003.
- Constraint COMP-C2: Cedar must authorize pack publication, overlay subscription, regulator engagement, and conflict override per ADR-0007.
- Constraint COMP-C3: data-use decisions must respect ADR-0008 data classes and purpose permissions.
- Constraint COMP-C4: regional pack semantics follow ADR-0010.
- Constraint COMP-C5: conflicts follow ADR-0304 higher-restriction, residency hard-stop, and transparency-report doctrine.
- Constraint COMP-C6: one tenant can have many packs: SOC 2, ISO 27001, GDPR, KR PIPA, HIPAA, PCI DSS, FedRAMP, SOX, DORA, EU AI Act, and local labor rules.
- Constraint COMP-C7: pack overlays can conflict on retention, residency, breach clock, regulator export, appeal, notice, consent, and automated decisioning.
- Constraint COMP-C8: the service must produce an explainable winning rule for every conflict.
- Constraint COMP-C9: a conflict cannot be silently resolved by array order or file load order.
- Constraint COMP-C10: pack overlays must be versioned and immutable after publish.
- Constraint COMP-C11: pack hotfix must be possible without rewriting historical decisions.
- Constraint COMP-C12: tenant opt-in must be explicit and auditable.
- Constraint COMP-C13: regulator-requested exports must obey both request authority and tenant pack floors.
- Constraint COMP-C14: cross-service consumers must get a single effective policy projection, not the whole legal reasoning graph.
- Constraint COMP-C15: compliance cannot become a global data lake of regulated payloads.
- The architecture must handle pack overlay precedence for data residency, retention, breach notification, DSR, DPIA, control mapping, and AI-risk flags.
- The architecture must make conflict reports usable by governance, tenancy, identity, drive, mail, messenger, and calendar.
- The architecture must keep legal explanation and machine projection in sync.

## Decision

- Define compliance pack overlays as immutable, versioned policy bundles.
- Store pack overlay metadata, rule fragments, Cedar policy refs, effective dates, jurisdiction, data classes, and evidence requirements.
- Compute an `EffectivePackPolicy` per tenant, data class, action, jurisdiction, and service.
- Resolve conflicts through a deterministic precedence order.
- Precedence order step 1: absolute legal hard-stop wins over all other rules.
- Precedence order step 2: data residency restriction wins over availability or convenience.
- Precedence order step 3: higher restriction wins for retention, breach clock, consent, export, and automated decisioning.
- Precedence order step 4: more specific jurisdiction wins when it is stricter than broader jurisdiction.
- Precedence order step 5: tenant explicit stricter policy can raise the floor.
- Precedence order step 6: product recommendation applies only when no pack rule covers the primitive.
- Never allow a less restrictive pack to weaken a stricter pack.
- Never allow tenant override to weaken a regulator floor.
- Publish one effective policy projection per tenant and pack set.
- Include a transparency report explaining every winning and losing rule.
- Include machine-readable conflict facts for governance rollups.
- Emit conflict decisions as audit events.
- Require Cedar approval for pack activation, pack deactivation, pack hotfix, and override request.
- Use pack overlay soak before activation unless an emergency regulator update flags immediate activation.
- Keep old pack versions immutable and queryable for historical evidence.
- Bind every pack overlay to scorecards, evidence requirements, control mappings, and retention classes.
- Make effective policy available to product services through read APIs and async events.
- Keep regulated payloads out of compliance projection; store pointers and evidence summaries.
- Use ADR-0304 as authority for cross-jurisdiction conflicts and cite conflict reason codes.
- Treat pack conflict resolution as part of tenant onboarding and every relevant pack change.

## Alternatives Considered

### Last-Writer-Wins Pack Overlay

- Pros: simple implementation.
- Pros: easy for tenant admins to reason about recent changes.
- Pros: cheap to compute.
- Cons: arbitrary and unsafe for legal conflicts.
- Cons: less restrictive late pack could weaken a stricter earlier pack.
- Cons: audit explanation is poor.
- Rejected because compliance policy must be deterministic and legally defensible.

### Manual Legal Review for Every Conflict

- Pros: highest human oversight.
- Pros: allows nuanced legal interpretation.
- Pros: useful for novel jurisdictions.
- Cons: does not scale to every request.
- Cons: blocks runtime policy evaluation.
- Cons: produces inconsistent operational decisions if not codified.
- Rejected as the default; manual review remains an escalation for unknown conflicts.

### Product-Service Local Conflict Rules

- Pros: product teams can tune for their workflows.
- Pros: fewer central compliance dependencies.
- Pros: lower initial implementation cost.
- Cons: drift across services.
- Cons: auditors cannot prove one conflict doctrine.
- Cons: ADR-0304 would be inconsistently applied.
- Rejected because pack conflict resolution is a compliance substrate.

### One Monolithic Global Compliance Policy

- Pros: single artifact to inspect.
- Pros: fewer merge joins.
- Pros: simple for small deployments.
- Cons: pack updates become risky and huge.
- Cons: tenant-specific projections are hard to explain.
- Cons: sovereign overlays and local laws change at different cadences.
- Rejected in favor of immutable pack overlays plus effective projections.

## Consequences

- Positive: every conflict has a deterministic winning rule and explanation.
- Positive: product services receive simple effective policy while compliance preserves reasoning.
- Positive: less restrictive packs cannot weaken stronger obligations.
- Positive: old decisions can be replayed against historical pack versions.
- Positive: governance can aggregate conflict counts and pack coverage.
- Positive: tenant onboarding can preview effective obligations before activation.
- Positive: emergency regulator updates can activate without mutating old pack versions.
- Positive: ADR-0304 doctrine becomes executable inside compliance.
- Negative: effective policy computation is more complex than static pack lists.
- Negative: pack authoring requires precise rule metadata.
- Negative: tenant admins may need explanation for why stricter pack wins.
- Negative: hotfix activation can create operational churn in product services.
- Negative: conflict reports may be large for multi-national tenants.
- Neutral: manual legal escalation still exists for unknown conflict types.
- Neutral: product services can suggest policy but cannot weaken pack floors.
- Neutral: governance stores minimized conflict rollups, while compliance stores detailed explanation.
- Neutral: policy changes are projected asynchronously but must be consumed before high-risk actions proceed.
- Neutral: tenant stricter policy is treated as a valid raising of the floor.

## Implementation Notes

- Data shape `CompliancePackOverlay`: `{pack_id, version, jurisdiction, effective_from, rule_refs[], cedar_policy_refs[], scorecard_refs[], status}`.
- Data shape `PackRule`: `{rule_id, pack_id, version, primitive, action, data_class, constraint_type, restriction_level, legal_basis, citation}`.
- Data shape `EffectivePackPolicy`: `{tenant_id, pack_set_hash, primitive, action, data_class, jurisdiction, winning_rule_id, decision, projection_version}`.
- Data shape `PackConflict`: `{conflict_id, tenant_id, candidate_rules[], winning_rule_id, losing_rule_ids[], reason_code, transparency_report_ref}`.
- Data shape `PackActivation`: `{tenant_id, pack_id, version, requested_by, approved_by, soak_until, state, audit_event_id}`.
- REST endpoint `POST /v1/compliance/packs` publishes immutable pack overlay.
- REST endpoint `POST /v1/compliance/tenants/{tenant_id}/packs/{pack_id}/activate` activates a pack.
- REST endpoint `POST /v1/compliance/tenants/{tenant_id}/effective-policy/evaluate` dry-runs effective policy.
- REST endpoint `GET /v1/compliance/tenants/{tenant_id}/effective-policy` returns current projection.
- REST endpoint `GET /v1/compliance/tenants/{tenant_id}/pack-conflicts` returns conflict reports.
- REST endpoint `POST /v1/compliance/packs/{pack_id}/hotfixes` publishes emergency hotfix version.
- REST endpoint `POST /v1/compliance/regulator-requests/{request_id}/evaluate` evaluates authority and pack floors.
- AsyncAPI channel `compliance.pack.published.v1` publishes pack version.
- AsyncAPI channel `compliance.pack.activated.v1` publishes tenant activation.
- AsyncAPI channel `compliance.effective-policy.changed.v1` publishes new projection.
- AsyncAPI channel `compliance.pack-conflict.detected.v1` publishes conflict facts.
- AsyncAPI channel `compliance.regulator-request.evaluated.v1` publishes authority decision.
- Cedar permit `compliance::pack::publish` requires compliance owner and signed pack schema.
- Cedar permit `compliance::pack::activate` requires tenant admin plus pack eligibility.
- Cedar permit `compliance::pack::hotfix` requires compliance owner and emergency reason or soak.
- Cedar forbid `compliance::pack::weaken_regulator_floor` is unconditional.
- Cedar permit `compliance::regulator_request::export` requires valid authority and effective policy allow.
- Audit event `EVT-COMP-PACK-PUBLISHED` includes pack id, version, hash, and author.
- Audit event `EVT-COMP-PACK-ACTIVATED` includes tenant, pack, version, and soak.
- Audit event `EVT-COMP-PACK-CONFLICT-RESOLVED` includes winning rule and reason code.
- Audit event `EVT-COMP-EFFECTIVE-POLICY-CHANGED` includes projection diff hash.
- Metric `compliance_effective_policy_compute_latency_ms` tracks projection compute time.
- Metric `compliance_pack_conflict_total` counts conflict by type and pack set.
- Metric `compliance_pack_projection_staleness_seconds` tracks product-service consumption lag.
- Metric `compliance_regulator_request_denied_total` tracks authority denials.
- Capacity math: a tenant with 20 packs, 500 rules per pack, and 200 primitives has 10k candidate rules; pre-index by primitive and data class to keep evaluation below 100 ms.
- Capacity math: 100k tenants with daily pack recalculation cannot full-scan all rules; recalc only affected pack sets and cache `pack_set_hash`.
- Rollback path: bad pack version is superseded by hotfix version; historical decisions keep original version.
- Rollback path: effective projection rollback reactivates previous projection hash and publishes a changed event.
- Multi-region path: policy computation can happen centrally only for metadata; regulated pack payloads remain in home jurisdiction.
- Sovereign path: data-residency hard-stop prevents central projection from exposing restricted data details.
- Versioning: pack overlay schema `compliance-pack-overlay-v1`.
- Deprecation: pack rules are superseded, never edited in place or deleted.

## Verification

- Unit test `hard_stop_beats_all_other_rules` verifies precedence step 1.
- Unit test `higher_restriction_wins_retention` verifies retention conflict.
- Unit test `tenant_policy_cannot_weaken_regulator_floor` verifies override limits.
- Unit test `effective_policy_includes_transparency_report_ref` verifies explainability.
- Unit test `pack_versions_immutable_after_publish` verifies immutable overlay.
- Property test `conflict_resolution_independent_of_rule_order` generates pack permutations.
- Property test `less_restrictive_pack_never_weakens_result` checks monotonicity.
- Fuzz test `pack_rule_parser_rejects_missing_legal_basis` covers malformed overlays.
- Integration test `activate_pack_publishes_effective_policy_changed` verifies async handoff.
- Integration test `regulator_export_denied_without_authority` verifies request gate.
- Integration test `hotfix_supersedes_without_rewriting_history` verifies pack immutability.
- Integration test `product_service_receives_single_projection` verifies consumer contract.
- Load test `evaluate_20_packs_10k_rules_under_100ms` validates computation target.
- Load test `recalculate_affected_pack_set_100k_tenants` validates incremental cache.
- Chaos test `audit_chain_unavailable_blocks_pack_activation` proves evidence-first behavior.
- Chaos test `projection_publish_failure_reverts_to_previous_hash` proves rollback.
- Metric SLO: `compliance_effective_policy_compute_latency_ms` p95 below 100 ms.
- Metric SLO: `compliance_pack_projection_staleness_seconds` p95 below 60 seconds.
- Metric SLO: conflict reports generated within 5 minutes of pack activation.
- Audit check: every conflict decision emits `EVT-COMP-PACK-CONFLICT-RESOLVED`.
- Audit check: every pack activation has tenant approval and soak or emergency reason.
- Static check: no resolver uses array order as precedence.
- Static check: pack rule records require legal citation and restriction level.
- Contract check: OpenAPI documents precedence reason codes from ADR-0304.

