---
id: ADR-SVC-CG-005
title: "B2C users can self-revoke consent with fail-closed propagation"
status: Accepted
date: 2026-05-18
microservice: consent-graph
related_oyatie_adrs:
  - ADR-0003
  - ADR-0214
  - ADR-0243
  - ADR-0244
  - ADR-0251
  - ADR-0258
  - ADR-0263
decision_owner: axis-consent-graph + council-privacy
---

# ADR-SVC-CG-005: B2C users can self-revoke consent with fail-closed propagation

## Context

- The named architectural pressure is `data-subject-revocation-without-tenant-admin`.
- Consent-graph must support B2C and personal-pillar agreements where the data subject is the grantor.
- ADR-0214 makes consent state part of real-time cross-tenant visibility.
- ADR-0243 requires Cedar to gate the revocation path.
- ADR-0003 requires audit-chain evidence for rights-affecting events.
- Prior incident class `tenant-admin-mediated-withdrawal` delayed withdrawal until a business admin acted.
- Prior incident class `revocation-ui-only` showed withdrawal in UI while APIs still served data.
- Prior incident class `notification-without-propagation` told grantees about withdrawal but did not revoke access.
- Prior incident class `ambiguous-subject-identity` let a user revoke the wrong subject binding.
- Legal pressure comes from GDPR Art. 7(3), GDPR Art. 17, CCPA/CPRA §1798.105, KR PIPA Art. 37, LGPD Art. 18, and ePrivacy Directive Art. 5.
- Withdrawal must be as easy as giving consent.
- A data subject must not need a grantee admin to act.
- A data subject must not need the grantor tenant admin when the subject is the grantor.
- Withdrawal must produce evidence immediately.
- Withdrawal must propagate to Cedar cache invalidation.
- Withdrawal must propagate to projection topics.
- Withdrawal must propagate to attested-query replay windows.
- Withdrawal must not delete audit evidence.
- Withdrawal must preserve legal hold when a separate lawful basis exists.
- The user experience is out of scope for this ADR, but API semantics are in scope.
- The API must be idempotent.
- The API must be resistant to account-takeover abuse.
- The API must support cooling-off only where law permits.
- The API must fail closed on uncertainty.
- The implementation must be buildable from this ADR.

## Decision

- We choose `self-revocation as first-class consent lifecycle event`.
- The named pattern is `subject-initiated revocation with synchronous authorization cutoff`.
- B2C self-revocation endpoint is `POST /v1/agreements/{agreement_id}/self-revoke`.
- The endpoint requires an authenticated data subject principal.
- The endpoint requires step-up authentication for sensitive scopes.
- Step-up authentication is WebAuthn L3 or passkey assertion.
- The endpoint accepts optional `reason_code`.
- The endpoint accepts optional `subject_note`.
- The endpoint returns `revocation_event_id`.
- The endpoint returns `effective_at`.
- The endpoint returns `propagation_deadline_at`.
- Self-revocation increments `consent_epoch`.
- Self-revocation writes a sealed audit-chain event before external notifications.
- Self-revocation invalidates Cedar decision cache synchronously.
- Self-revocation publishes projection tombstone synchronously.
- Self-revocation closes attested-query verifier replay windows synchronously.
- Self-revocation target for authorization cutoff is <= 500 ms p99 same-region.
- Self-revocation target for cross-region cutoff is <= 1 second p99.
- User-visible confirmation target is <= 2 seconds p99.
- Grantee notification target is <= 60 seconds p99.
- Same request id is idempotent for 24 hours.
- Repeated revoke after already revoked returns 200 with original event id.
- Revocation cannot be silently undone.
- Re-grant requires a new consent grant ceremony.
- Legal hold does not re-enable consent-based access.
- Legal hold creates separate `lawful_basis_hold` evidence and must be Cedar-gated.
- Cedar action `consent-graph.agreement.self_revoke` gates self-revocation.
- Cedar action `consent-graph.agreement.regrant` gates later re-grant.
- Cedar action `consent-graph.revocation.notify` gates notification fanout.
- Cedar action `consent-graph.revocation.legal_hold_view` gates hold evidence access.
- A tenant admin cannot block a valid self-revocation.
- A grantee cannot delay a valid self-revocation.

## Alternatives Considered

### Tenant-admin mediated revocation

- Pro: familiar enterprise workflow.
- Pro: admin can verify business context.
- Pro: fewer account-takeover concerns.
- Con: violates GDPR Art. 7(3) ease of withdrawal.
- Con: delays data-subject rights.
- Con: creates conflict when tenant incentives oppose revocation.
- Con: weak B2C posture.
- Tradeoff: operational control but rights-hostile.
- Rejected.

### UI-only revocation with async backend reconciliation

- Pro: fast user experience.
- Pro: simple product implementation.
- Pro: backend can converge later.
- Con: APIs continue serving data after user sees revoked.
- Con: audit evidence is misleading.
- Con: creates regulator-visible gap.
- Tradeoff: perceived speed but incorrect semantics.
- Rejected.

### Hard delete all shared data immediately

- Pro: simple mental model for users.
- Pro: strong privacy optics.
- Pro: minimizes residual data.
- Con: may destroy audit evidence.
- Con: may conflict with legal hold or financial records.
- Con: grantee may have independent lawful basis for retained copies.
- Tradeoff: privacy-forward but legally overbroad.
- Rejected.

### Email support ticket revocation

- Pro: no new API.
- Pro: human review can catch fraud.
- Pro: easy early launch.
- Con: not as easy as consent grant.
- Con: unbounded latency.
- Con: no machine-verifiable cutoff evidence.
- Tradeoff: simple support process but fails rights standard.
- Rejected.

### Cooling-off delay before effective revocation

- Pro: mitigates account takeover abuse.
- Pro: gives users time to reverse accidental clicks.
- Pro: useful for high-value data feeds.
- Con: delays withdrawal.
- Con: not allowed as default under GDPR Art. 7(3).
- Con: creates stale access window.
- Tradeoff: abuse mitigation but rights latency.
- Partial accept: delay may exist only for non-sensitive optional feeds where pack law permits and user explicitly chose it at grant time.

## Consequences

- Positive: data subjects can withdraw without tenant or grantee permission.
- Positive: API semantics match legal rights obligations.
- Positive: revocation cutoff is measurable.
- Positive: cache and projection systems share one consent epoch.
- Positive: audit chain proves when access stopped.
- Negative: account takeover can trigger unwanted revocation.
- Negative: step-up authentication adds friction.
- Negative: grantee workflows must handle immediate cutoff.
- Negative: re-grant ceremonies may increase support volume.
- Neutral: legal-hold evidence remains separate from consent access.
- Neutral: deletion rights require separate erasure workflow.
- Follow-up work: add user-facing consent receipt update.
- Follow-up work: add notification templates by pack.
- Follow-up work: add account-takeover revocation reversal runbook.
- Follow-up work: add regulator evidence export for revocation cutoff.

## Implementation Notes

- Data shape `SelfRevocationRequestV1` contains `agreement_id`.
- Data shape `SelfRevocationRequestV1` contains `subject_id`.
- Data shape `SelfRevocationRequestV1` contains `idempotency_key`.
- Data shape `SelfRevocationRequestV1` contains `reason_code`.
- Data shape `SelfRevocationRequestV1` contains `subject_note`.
- Data shape `SelfRevocationRequestV1` contains `step_up_assertion`.
- Data shape `SelfRevocationResponseV1` contains `revocation_event_id`.
- Data shape `SelfRevocationResponseV1` contains `effective_at`.
- Data shape `SelfRevocationResponseV1` contains `propagation_deadline_at`.
- Data shape `SelfRevocationResponseV1` contains `consent_epoch`.
- Data shape `SelfRevocationResponseV1` contains `notification_status_url`.
- Data shape `RevocationLifecycleEventV1` contains `agreement_id`.
- Data shape `RevocationLifecycleEventV1` contains `previous_consent_epoch`.
- Data shape `RevocationLifecycleEventV1` contains `new_consent_epoch`.
- Data shape `RevocationLifecycleEventV1` contains `revoked_by_principal_id`.
- Data shape `RevocationLifecycleEventV1` contains `revoked_by_subject_hash`.
- Data shape `RevocationLifecycleEventV1` contains `step_up_method`.
- Data shape `RevocationLifecycleEventV1` contains `legal_hold_present`.
- Data shape `RevocationLifecycleEventV1` contains `cutoff_completed_at`.
- API endpoint `POST /v1/agreements/{agreement_id}/self-revoke` starts revocation.
- API endpoint `GET /v1/agreements/{agreement_id}/revocations/{event_id}` returns status.
- API endpoint `POST /v1/internal/revocations/{event_id}/notify` sends grantee notification.
- API endpoint `POST /v1/internal/revocations/{event_id}/cutoff` executes cutoff workers.
- API endpoint `GET /v1/internal/revocations/{event_id}/evidence` exports evidence bundle.
- Idempotency key storage uses PostgreSQL 16.6 unique constraint on `(subject_hash, agreement_id, idempotency_key)`.
- Revocation events are appended to audit-chain before notification.
- Cache invalidation uses ADR-SVC-CG-002 events.
- Projection tombstone uses ADR-SVC-CG-004 topics.
- Attested query expiry sets answer state to `revoked`.
- Cedar principal is `Oyatie::Principal::User("subject:{subject_id}")`.
- Cedar resource is `ConsentGraph::Agreement`.
- Cedar action is `consent-graph.agreement.self_revoke`.
- Example permit: principal `User("subject:sub_01HY")`, action `consent-graph.agreement.self_revoke`, resource `ConsentGraph::Agreement::"dsa_01HY"`, context `{subject_hash:"h_sub_a", agreement_subject_hash:"h_sub_a", agreement_state:"accepted", step_up_method:"webauthn_l3"}`.
- Example forbid: same action with context `{subject_hash:"h_sub_b", agreement_subject_hash:"h_sub_a"}`.
- Example forbid: same action with context `{agreement_state:"expired"}`.
- Example permit: principal `consent-graph.notification-worker`, action `consent-graph.revocation.notify`, resource `ConsentGraph::Revocation::"cg_rev_01HY"`, context `{notification_channel:"pulsar", deadline_seconds:60}`.
- SLO `consent-self-revocation-cutoff.openslo.yaml` sets same-region p99 <= 500 ms.
- SLO `consent-self-revocation-cross-region.openslo.yaml` sets cross-region p99 <= 1 second.
- SLO `consent-self-revocation-notification.openslo.yaml` sets grantee notification p99 <= 60 seconds.
- Failure mode `step_up_failed` returns 403 and emits security event.
- Failure mode `subject_mismatch` returns 403 and emits rights-abuse event.
- Failure mode `cutoff_worker_timeout` fails closed and pages Sev-1.
- Failure mode `notification_late` does not re-enable access.
- Failure mode `legal_hold_present` blocks deletion but not consent cutoff.
- Failure mode `account_takeover_suspected` preserves revocation but opens recovery review.

## Verification

- Test `self_revoke_requires_subject_principal` verifies principal binding.
- Test `self_revoke_requires_step_up_for_sensitive_scope` verifies WebAuthn gate.
- Test `self_revoke_subject_mismatch_forbidden` verifies subject matching.
- Test `self_revoke_idempotent_24h` verifies repeated request behavior.
- Test `self_revoke_already_revoked_returns_original_event` verifies idempotence.
- Test `self_revoke_increments_consent_epoch` verifies epoch.
- Test `self_revoke_invalidates_cache_before_response` verifies cutoff ordering.
- Test `self_revoke_publishes_projection_tombstone` verifies projection cutoff.
- Test `self_revoke_closes_attested_query_replay` verifies proof expiry.
- Test `legal_hold_does_not_reenable_consent_access` verifies lawful-basis separation.
- Test `tenant_admin_cannot_block_self_revoke` verifies rights posture.
- Test `grantee_cannot_delay_self_revoke` verifies downstream cutoff.
- Metric `oya_consent_graph_self_revocation_cutoff_ms` must meet p99 <= 500 ms same-region.
- Metric `oya_consent_graph_self_revocation_cross_region_ms` must meet p99 <= 1 second.
- Metric `oya_consent_graph_self_revocation_notification_ms` must meet p99 <= 60 seconds.
- Metric `oya_consent_graph_self_revocation_fail_closed_total` tracks cutoff failures.
- Dashboard `consent-graph-self-revocation.json` shows cutoff, notification, and fail-closed counts.
- Dashboard `consent-graph-data-subject-rights.json` shows revocation volume by pack.
- Dashboard `consent-graph-regrant.json` shows new grants after revocation.
- CI check `consent-self-revocation-openapi` validates endpoints.
- CI check `consent-self-revocation-cedar` validates policies.
- CI check `consent-self-revocation-audit-events` validates evidence.
- CI check `consent-self-revocation-no-admin-block` rejects admin veto paths.
- Chaos test stalls notification worker and verifies cutoff remains effective.
- Chaos test drops projection tombstone and expects fail-closed agreement.
- Rights drill executes quarterly with GDPR and KR PIPA packs.

## References

- ADR-0003: Audit-chain and evidence emission.
- ADR-0214: Cross-tenant real-time visibility.
- ADR-0243: Cedar as Universal Gate.
- ADR-0244: Tenant as universal scoping primitive.
- ADR-0251: Compliance pack cell certification levels.
- ADR-0258: API versioning model.
- ADR-0263: Observability emission contract.
- GDPR Art. 7(3) and Art. 17.
- CCPA/CPRA Cal. Civ. Code §1798.105.
- KR PIPA Art. 37.
- LGPD Art. 18.
- ePrivacy Directive 2002/58/EC Art. 5.
- HIPAA 45 CFR §164.312(a)(1).
- W3C WebAuthn Level 3.
- FIDO2 Client to Authenticator Protocol 2.1.
- RFC 8032: Ed25519 signatures.
- RFC 8785: JSON Canonicalization Scheme.
- NIST SP 800-63B authentication guidance.
