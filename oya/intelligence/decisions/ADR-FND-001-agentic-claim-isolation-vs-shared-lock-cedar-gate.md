---
id: ADR-FND-001
title: Agentic Claim Isolation vs Shared Lock Cedar Gate
status: Proposed
date: 2026-05-20
microservice: foundry
related_oyatie_adrs:
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0702-identity-authz-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
decision_owner: axis-foundry-runtime + ops-security
---

# ADR-FND-001: Agentic Claim Isolation vs Shared Lock Cedar Gate

## Context

- Foundry is the single microservice for agent runtime, supervisor, eval, evidence, guardrails, providers, and capability execution.
- The workspace guidance requires Oya VCS claim, verify, done, and promote state transitions.
- ADR-0116 retires external agent-coordination tooling and keeps Oya VCS as the compatibility policy-ratchet surface.
- Existing Foundry IPs include runtime autonomy tier gates, supervisor lifecycle, evidence builder, guardrails, and provider router lanes.
- Agentic implementation work can run in parallel across many file scopes.
- Named pressure FND-P1: two agents editing the same file or identifier can corrupt a changeset.
- Named pressure FND-P2: global shared locks serialize too much work and kill throughput.
- Named pressure FND-P3: pure optimistic merge can discover conflicts after hours of work.
- Named pressure FND-P4: claims need to be Cedar-authorized, auditable, and tied to delegated authority.
- Named pressure FND-P5: claims must map to file paths and semantic identifiers, not only branch names.
- Named pressure FND-P6: verify and promote must prove the claim's evidence before expanding blast radius.
- Named pressure FND-P7: child agents need bounded ownership without becoming autonomous planners.
- Named pressure FND-P8: stale claims must expire or be recoverable without destructive resets.
- Named pressure FND-P9: claim conflict output must be machine-readable for agent recovery.
- Named pressure FND-P10: Foundry needs honest claim status for dashboards and promotion gates.
- Constraint FND-C1: audit-chain evidence is required for state transitions per ADR-0003.
- Constraint FND-C2: Cedar is the universal authorization gate per ADR-0007 and ADR-0243.
- Constraint FND-C3: capability registry and MCP gateway authority follows ADR-0021.
- Constraint FND-C4: autonomy ceilings are enforced at runtime per ADR-0022.
- Constraint FND-C5: changeset state transitions follow ADR-0110.
- Constraint FND-C6: plan DAG and honest claim gates follow ADR-0129.
- Constraint FND-C7: Foundry remains one microservice per ADR-0136.
- Constraint FND-C8: delegated authority chain follows ADR-0305.
- Constraint FND-C9: retired external coordination tools cannot be reintroduced as the source of truth.
- Constraint FND-C10: Oya VCS compatibility verbs remain the policy surface for claim, verify, done, and promote.
- A shared lock is simple but too coarse.
- Pure claim isolation is fast but needs conflict detection and policy.
- The system needs claim isolation with a Cedar gate and typed conflict semantics.

## Decision

- Adopt agentic claim isolation as the primary concurrency model.
- Use Cedar-gated shared lock only for explicitly declared critical sections.
- Name the model `FoundryClaimIsolation v1`.
- Treat each claim as a lease over path scopes and optional semantic identifiers.
- Require claims before edits to claimed paths.
- Require verify before done.
- Require done before promote.
- Require promote to attach evidence bundle, environment, and changeset id.
- Use Cedar to authorize claim creation, claim extension, verify, done, promote, steal, and release.
- Use lease expiry to recover stale claims.
- Use semantic conflicts when two claims overlap on the same `file::Identifier`.
- Use path conflicts when two claims overlap on exact file paths.
- Use directory conflicts when a recursive directory claim conflicts with a child path claim.
- Use shared critical-section locks only for migrations, generated indices, registry writes, and root control surfaces.
- For ordinary microservice ADR authoring, use isolated path claims rather than global locks.
- Keep claim state in Foundry evidence store and Oya VCS compatibility surface.
- Emit typed events for claim accepted, conflict, verified, done, promoted, expired, and stolen.
- Bind every claim to an agent id, intent, branch, workspace, and monotonic lease version.
- Bind every claim to delegated authority context when issued by a leader to a child agent.
- Deny claim creation when requested scope violates authority or existing exclusive claim.
- Allow non-overlapping claims to proceed in parallel.
- Allow read-only analysis without a write claim, but require a claim before edits.
- Allow claim expansion only after conflict check and Cedar permit.
- Require all claim conflict responses to include blocking claim id, owner, scope, lease expiration, and suggested narrower scope.
- Require all verify responses to include evidence summary and failing gates if any.
- Require all done responses to include changed scopes and residual risk.
- Require all promote responses to include environment and bundle digest.
- Do not use a single repository-wide mutex for normal agent work.
- Do not let an agent silently edit outside its claim scope.
- Do not let child agents widen ownership without leader approval.
- Make this ADR authoritative for Foundry agentic claim isolation and shared lock use.

## Alternatives Considered

### Repository Wide Shared Lock

- Pros: simple to reason about.
- Pros: prevents all concurrent write conflicts.
- Pros: easy to implement with one lease row.
- Cons: prevents useful parallel work across unrelated microservices.
- Cons: creates long queues for large documentation batches.
- Cons: does not teach agents precise ownership boundaries.
- Rejected for normal work; retained only for critical global operations.

### Pure Optimistic Concurrency

- Pros: maximal throughput.
- Pros: no up-front coordination overhead.
- Pros: fits simple branch-per-agent workflows.
- Cons: conflicts emerge late.
- Cons: agents may waste time writing doomed patches.
- Cons: missing policy evidence before editing.
- Rejected because Oyatie requires claim-before-edit discipline.

### File-Only Claims

- Pros: easy to implement.
- Pros: maps to git diff paths.
- Pros: good for many documentation edits.
- Cons: does not catch semantic overlap inside shared files.
- Cons: generated indices and registries need identifier-level ownership.
- Cons: future refactors need symbol-aware conflict checks.
- Rejected as the whole model; accepted as one scope kind.

### Semantic Identifier Claims Only

- Pros: precise for code and registry entities.
- Pros: supports multiple agents in the same file.
- Pros: aligns with `file::Identifier` guidance.
- Cons: documentation and generated files may not have parseable identifiers.
- Cons: requires language-aware index quality.
- Cons: path-level deletes and moves still need file ownership.
- Rejected as the only model; accepted alongside path claims.

### External Coordinator as Lock Source

- Pros: may already offer team coordination UX.
- Pros: decouples claim storage from Foundry.
- Pros: can be fast to adopt.
- Cons: violates ADR-0116 retirement.
- Cons: weakens audit-chain and Oya VCS state transition authority.
- Cons: splits source of truth.
- Rejected because Oya VCS and Foundry evidence are the policy surfaces.

## Consequences

- Positive: unrelated agents can work in parallel.
- Positive: claim conflicts are detected before edits.
- Positive: Cedar policy controls who can claim, verify, done, promote, or steal.
- Positive: state transitions align with Oya VCS guidance.
- Positive: stale claims are recoverable through lease expiry.
- Positive: critical operations can still use shared locks.
- Positive: dashboards can report claim health and conflict density.
- Negative: claim storage and conflict detection become load-bearing.
- Negative: semantic identifier extraction needs quality gates.
- Negative: agents must request claim expansion for legitimate scope growth.
- Negative: lease expiry can surprise a slow worker if progress heartbeats fail.
- Neutral: read-only exploration remains unclaimed.
- Neutral: branch isolation still matters but is not enough.
- Neutral: team mode can layer on top of the same claim model.
- Neutral: Oya VCS remains compatibility spelling until policy verbs split explicitly.
- Follow-up work FND-F1: add claim conflict schema to Foundry contracts.
- Follow-up work FND-F2: add semantic identifier extraction tests.
- Follow-up work FND-F3: add stale claim recovery runbook.
- Follow-up work FND-F4: add claim health dashboard.
- Follow-up work FND-F5: add critical-section lock catalog.

## Implementation Notes

- Data shape `ClaimLease`: `{claim_id, agent_id, intent, branch, workspace_id, lease_version, expires_at, state}`.
- Data shape `ClaimScope`: `{claim_id, scope_kind, path, identifier, recursive, access_mode, exclusivity}`.
- Data shape `ClaimConflict`: `{request_id, blocking_claim_id, conflict_kind, requested_scope, blocking_scope, suggested_scope}`.
- Data shape `ClaimAuthority`: `{agent_id, delegated_by, max_scope, allowed_actions, autonomy_level, expires_at}`.
- Data shape `ClaimEvidence`: `{claim_id, changeset_id, verify_evidence, done_evidence, promote_bundle, audit_event_ids}`.
- Data shape `CriticalSectionLock`: `{lock_id, name, scope, reason, owner_agent_id, expires_at, state}`.
- Postgres table `foundry_claim_lease` stores claim leases.
- Postgres table `foundry_claim_scope` stores path and identifier scopes.
- Postgres table `foundry_claim_conflict` stores denied claim attempts.
- Postgres table `foundry_claim_evidence` stores verify, done, and promote evidence.
- Postgres table `foundry_critical_section_lock` stores shared locks.
- REST endpoint `POST /v1/foundry/claims` creates a claim.
- REST endpoint `POST /v1/foundry/claims/{claim_id}/extend` extends lease or scope.
- REST endpoint `POST /v1/foundry/claims/{claim_id}/verify` records verification.
- REST endpoint `POST /v1/foundry/claims/{claim_id}/done` marks done.
- REST endpoint `POST /v1/foundry/claims/{claim_id}/promote` promotes with bundle.
- REST endpoint `POST /v1/foundry/claims/{claim_id}/release` releases a claim.
- REST endpoint `POST /v1/foundry/claims/{claim_id}/steal` recovers stale or blocked claim.
- REST endpoint `POST /v1/foundry/critical-section-locks` creates shared lock.
- AsyncAPI channel `foundry.claim.accepted.v1` publishes accepted claim.
- AsyncAPI channel `foundry.claim.conflict.v1` publishes conflict.
- AsyncAPI channel `foundry.claim.verified.v1` publishes verify evidence.
- AsyncAPI channel `foundry.claim.done.v1` publishes done evidence.
- AsyncAPI channel `foundry.claim.promoted.v1` publishes promote evidence.
- AsyncAPI channel `foundry.claim.expired.v1` publishes expiry.
- Cedar action `foundry::claim::create` requires agent authority and non-conflicting scope.
- Cedar action `foundry::claim::extend` requires current owner or leader authority.
- Cedar action `foundry::claim::verify` requires claim owner and evidence attachment.
- Cedar action `foundry::claim::done` requires verified claim.
- Cedar action `foundry::claim::promote` requires done claim and environment permission.
- Cedar action `foundry::claim::steal` requires stale lease or security override.
- Cedar action `foundry::lock::critical_section_create` requires critical operation reason.
- SLO target `foundry_claim_conflict_detection_p95_ms` is <=200.
- SLO target `foundry_claim_false_non_conflict_total` is 0.
- SLO target `foundry_claim_stale_recovery_p95_minutes` is <=15.
- SLO target `foundry_claim_transition_audit_lag_p95_seconds` is <=5.
- SLO target `foundry_claim_promote_evidence_completeness_ratio` is 1.0.

## Verification

- Unit test `overlapping_path_claim_is_denied` proves path conflict detection.
- Unit test `overlapping_identifier_claim_is_denied` proves semantic conflict detection.
- Unit test `non_overlapping_microservice_claims_can_parallelize` proves useful concurrency.
- Unit test `claim_extension_requires_cedar_permit` proves policy gate.
- Unit test `verify_requires_claim_owner` proves ownership.
- Unit test `done_requires_verified_claim` proves state machine.
- Unit test `promote_requires_done_and_bundle` proves promotion gate.
- Unit test `critical_section_lock_requires_registered_reason` proves shared lock restraint.
- Contract test `claim_conflict_response_contains_suggested_scope` proves recovery ergonomics.
- Contract test `promote_response_contains_bundle_digest` proves evidence output.
- Property test `claim_scope_intersection_is_symmetric` proves conflict algebra.
- Property test `lease_version_monotonic_under_extend` proves stale write safety.
- Replay test `claim_events_rebuild_current_claim_state` proves event sufficiency.
- Integration test `agent_cannot_edit_outside_claim_scope` proves enforcement hook.
- Integration test `stale_claim_can_be_stolen_after_expiry` proves recovery.
- Failure test `audit_chain_unavailable_blocks_promote` proves evidence-first posture.
- Failure test `semantic_index_unavailable_falls_back_to_path_conflict` proves safe degradation.
- Security test `child_agent_cannot_widen_scope_without_leader` proves delegated authority.
- Metric `foundry_claim_active_total` tracks active claims by service and owner.
- Metric `foundry_claim_conflict_total` tracks conflicts by kind.
- Metric `foundry_claim_transition_duration_ms` tracks claim, verify, done, promote latency.
- Metric `foundry_claim_stale_total` tracks expired and stolen claims.
- Metric `foundry_critical_section_lock_active_total` tracks shared locks.
- Metric `foundry_claim_false_non_conflict_total` tracks post-hoc conflicts.
- Dashboard `foundry-agentic-claim-isolation` shows active claims, conflicts, stale leases, and promotion state.
- Dashboard `foundry-critical-section-locks` shows shared lock usage and duration.
- Dashboard `foundry-claim-evidence` shows verify, done, promote evidence completeness.
- Dashboard `foundry-delegated-authority-chain` shows leader, child, and autonomy tier linkages.
- Alert `FoundryClaimFalseNonConflict` fires on any post-hoc claim collision.
- Alert `FoundryClaimStaleLeaseBurn` fires when stale recovery p95 exceeds 15 minutes.
- Alert `FoundryCriticalSectionLockTooLong` fires when shared lock exceeds catalog budget.
- Alert `FoundryPromoteWithoutCompleteEvidence` fires on any evidence gap.

## References

- Internal: docs/AGENTS.md agent-instructions block
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- Internal: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Internal: docs/decisions/ADR-0709-general-live-apex.md
- Internal: docs/decisions/ADR-0700-ci-admission-live-apex.md
- Internal: microservices/intelligence/IP-012-runtime-autonomy-ceiling-gate.md
- Internal: microservices/intelligence/policy/runtime-tenant-scope.cedar
- Cedar policy language syntax: https://docs.cedarpolicy.com/policies/syntax-policy.html
- Cedar authorization language paper: https://arxiv.org/abs/2403.04651
- OpenTelemetry semantic conventions: https://opentelemetry.io/docs/concepts/semantic-conventions/
- CloudEvents Specification: https://cloudevents.io/
- Kubernetes leases concept: https://kubernetes.io/docs/concepts/architecture/leases/
- Git documentation on index and locking: https://git-scm.com/docs/git-update-index
- Martin Fowler, Event Sourcing: https://www.martinfowler.com/eaaDev/EventSourcing.html
- OpenAPI Specification: https://spec.openapis.org/oas/
