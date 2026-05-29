---
doc_class: BackfillReplay
microservice: forms
status: Accepted
date: 2026-05-17
owner_team: axis-forms + ops-sre-reliability + axis-foundry-evidence
doc_status: published
---

# Forms — Backfill & Replay Strategy

## Backfill scenarios

| Scenario | Trigger | Strategy |
|---|---|---|
| New analytic field added to Response | Schema evolution | Lazy-compute on next read; eager-compute via background worker |
| Per-tenant DEK rotation | Quarterly | Rolling re-encryption with dual-key window (read both; write new) |
| Pack onboarding (new pack activated) | New region | Empty start; no backfill (no migration cross-pack) |
| Form-version migration | Tenant updates form schema (non-breaking) | Old responses queryable against old schema; new responses against new |
| Audit-chain re-seal | Key rotation | Replay chain; new seals appended; old seals retained |

## Replay scenarios

| Scenario | Trigger | Strategy |
|---|---|---|
| Webhook DLQ drain | Tenant fixes endpoint | Replay from DLQ; idempotency key honoured |
| Workflow-trigger replay | workflow-engine downtime | Submission events buffered in Kafka; replay when engine healthy |
| Audit-chain integrity check | Quarterly | Full chain replay; verify Ed25519 seal continuity |
| Bulk-distribute partial failure | Mail outage mid-blast | Resume from last-acked recipient; idempotency cap |
| DSR cascade replay | Cascade failed mid-execution | Resume from last-erased entity; ledger ensures no double-erase / no skip |

## Idempotency

Every backfill + replay is idempotent. Idempotency keys:
- Submission: `(tenant_id, form_id, response_id)` — server-assigned at submit; client-replayed safe.
- Webhook: `(tenant_id, response_id, webhook_target_id)` — per-delivery nonce.
- Bulk-distribute: `(tenant_id, distribute_job_id, recipient_id)`.
- DSR: `(tenant_id, subject_hash, response_id)`.
- AI-form-build: `(tenant_id, prompt_hash, model_id)` — same prompt → cached completion unless explicit re-roll.

## Replay throttling

| Replay class | Throttle |
|---|---|
| Webhook | per-tenant token bucket (default 100 RPS); manual elevate via ops |
| Workflow-trigger | per-tenant token bucket (default 50 RPS) |
| Bulk-distribute | mail µservice ingress rate (cf. `microservices/mail/capacity-model.md`) |
| DSR cascade | per-pack queue (default 10 RPS); SLA per pack |

## Verification

- Quarterly chaos drill: induce webhook DLQ; verify replay drains.
- Audit-chain replay daily (last-7d window); SLI `oya_forms_audit_chain_replay_success` = 100%.
- DSR replay tested per pack onboarding.

## References

- `failure-modes.md`.
- `capacity-model.md`.
- `compliance.md`.
- ADR-0028 audit-chain (Ed25519 seal continuity).
- ADR-0110 ChangeSet state machine.
- ADR-0131 per-microservice flat layout.
